package edge

import (
	"context"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/mcpclient"
)

type fakeClient struct {
	active     atomic.Int64
	maxActive  atomic.Int64
	failures   atomic.Int64
	connected  chan struct{}
}

func (c *fakeClient) Connect(ctx context.Context, upstream mcpclient.Upstream) (mcpclient.Session, error) {
	if remaining := c.failures.Load(); remaining > 0 {
		if c.failures.CompareAndSwap(remaining, remaining-1) {
			return nil, context.DeadlineExceeded
		}
		return c.Connect(ctx, upstream)
	}
	active := c.active.Add(1)
	for {
		current := c.maxActive.Load()
		if active <= current || c.maxActive.CompareAndSwap(current, active) {
			break
		}
	}
	if c.connected != nil {
		select {
		case c.connected <- struct{}{}:
		default:
		}
	}
	return &fakeSession{client: c}, nil
}

type fakeSession struct {
	client *fakeClient
	once   sync.Once
}

func (s *fakeSession) ProtocolVersion() string { return "2026-07-28" }
func (s *fakeSession) ListTools(context.Context) ([]mcpclient.ToolSummary, error) {
	return nil, nil
}
func (s *fakeSession) CallTool(context.Context, string, map[string]any) (mcpclient.ToolResult, error) {
	return mcpclient.ToolResult{}, nil
}
func (s *fakeSession) Close() error {
	s.once.Do(func() { s.client.active.Add(-1) })
	return nil
}

func upstream(index int) mcpclient.Upstream {
	return mcpclient.Upstream{
		Name:     "upstream-" + string(rune('a'+index%26)),
		Kind:     mcpclient.UpstreamStreamableHTTP,
		Endpoint: "http://127.0.0.1:9000/mcp",
	}
}

func TestRunUpstreamsNeverExceedsConfiguredSessionLimit(t *testing.T) {
	client := &fakeClient{connected: make(chan struct{}, 64)}
	manager := NewManager(
		WithClient(client),
		WithLimits(16, 256),
	)
	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan error, 1)
	upstreams := make([]mcpclient.Upstream, 64)
	for i := range upstreams {
		upstreams[i] = upstream(i)
	}

	go func() { done <- manager.RunUpstreams(ctx, upstreams) }()
	for i := 0; i < 16; i++ {
		select {
		case <-client.connected:
		case <-time.After(time.Second):
			t.Fatal("timed out waiting for bounded sessions")
		}
	}
	if got := client.maxActive.Load(); got > 16 {
		t.Fatalf("max active sessions = %d, want <= 16", got)
	}

	cancel()
	if err := <-done; err != nil {
		t.Fatalf("RunUpstreams() error = %v", err)
	}
	if got := client.active.Load(); got != 0 {
		t.Fatalf("active sessions after cancellation = %d", got)
	}
}

func TestRunUpstreamsUsesBoundedRetryDelays(t *testing.T) {
	client := &fakeClient{connected: make(chan struct{}, 1)}
	client.failures.Store(2)
	var mu sync.Mutex
	var delays []time.Duration
	manager := NewManager(
		WithClient(client),
		WithLimits(1, 8),
		WithDelay(func(ctx context.Context, duration time.Duration) error {
			mu.Lock()
			delays = append(delays, duration)
			mu.Unlock()
			return ctx.Err()
		}),
	)
	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan error, 1)
	go func() { done <- manager.RunUpstreams(ctx, []mcpclient.Upstream{upstream(0)}) }()

	select {
	case <-client.connected:
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for retried connection")
	}
	cancel()
	if err := <-done; err != nil {
		t.Fatalf("RunUpstreams() error = %v", err)
	}

	mu.Lock()
	defer mu.Unlock()
	if len(delays) < 2 || delays[0] != 250*time.Millisecond || delays[1] != 500*time.Millisecond {
		t.Fatalf("retry delays = %v", delays)
	}
}

func TestObservationQueueCoalescesBySourceAndStaysBounded(t *testing.T) {
	manager := NewManager(WithLimits(1, 2))
	manager.publishObservation(ConnectivityStatus{SchemaVersion: "1", SourceID: "a", Kind: "mcp", State: ConnectionConnecting, Protocol: "2026-07-28", ObservedAt: "2026-08-17T12:00:00Z"})
	manager.publishObservation(ConnectivityStatus{SchemaVersion: "1", SourceID: "a", Kind: "mcp", State: ConnectionDegraded, Protocol: "2026-07-28", ObservedAt: "2026-08-17T12:00:01Z"})
	manager.publishObservation(ConnectivityStatus{SchemaVersion: "1", SourceID: "a", Kind: "mcp", State: ConnectionReady, Protocol: "2026-07-28", ObservedAt: "2026-08-17T12:00:02Z"})

	if got := manager.observationQueueLen(); got != 1 {
		t.Fatalf("coalesced queue length = %d, want 1", got)
	}
	status, err := manager.NextObservation(context.Background())
	if err != nil {
		t.Fatalf("NextObservation() error = %v", err)
	}
	if status.State != ConnectionReady {
		t.Fatalf("coalesced state = %q, want ready", status.State)
	}

	manager.publishObservation(ConnectivityStatus{SchemaVersion: "1", SourceID: "a", Kind: "mcp", State: ConnectionReady, Protocol: "2026-07-28", ObservedAt: "2026-08-17T12:00:03Z"})
	manager.publishObservation(ConnectivityStatus{SchemaVersion: "1", SourceID: "b", Kind: "provider", State: ConnectionReady, Protocol: "https", ObservedAt: "2026-08-17T12:00:03Z"})
	manager.publishObservation(ConnectivityStatus{SchemaVersion: "1", SourceID: "c", Kind: "mcp", State: ConnectionReady, Protocol: "2026-07-28", ObservedAt: "2026-08-17T12:00:03Z"})
	if got := manager.observationQueueLen(); got != 2 {
		t.Fatalf("bounded queue length = %d, want 2", got)
	}
}
