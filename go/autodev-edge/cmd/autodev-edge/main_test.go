package main

import (
	"context"
	"sync/atomic"
	"testing"
	"time"

	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/mcpclient"
)

type runtimeFakeClient struct {
	connected chan mcpclient.Upstream
	closed    atomic.Int64
}

func (c *runtimeFakeClient) Connect(ctx context.Context, upstream mcpclient.Upstream) (mcpclient.Session, error) {
	select {
	case c.connected <- upstream:
	case <-ctx.Done():
		return nil, ctx.Err()
	}
	return &runtimeFakeSession{closed: &c.closed}, nil
}

type runtimeFakeSession struct {
	closed *atomic.Int64
}

func (s *runtimeFakeSession) ProtocolVersion() string { return "2026-07-28" }
func (s *runtimeFakeSession) ListTools(context.Context) ([]mcpclient.ToolSummary, error) {
	return nil, nil
}
func (s *runtimeFakeSession) CallTool(context.Context, string, map[string]any) (mcpclient.ToolResult, error) {
	return mcpclient.ToolResult{}, nil
}
func (s *runtimeFakeSession) Close() error {
	s.closed.Add(1)
	return nil
}

func runtimeEnv(key string) string {
	values := map[string]string{
		"AUTODEV_EDGE_UPSTREAM_NAME":        "filesystem",
		"AUTODEV_EDGE_UPSTREAM_KIND":        "streamable_http",
		"AUTODEV_EDGE_UPSTREAM_ENDPOINT":    "http://127.0.0.1:9000/mcp",
		"AUTODEV_EDGE_OBSERVATION_CAPACITY": "8",
		"AUTODEV_EDGE_MAX_SESSIONS":         "1",
	}
	return values[key]
}

func TestRunWithContextConnectsConfiguredUpstreamAndClosesOnCancellation(t *testing.T) {
	client := &runtimeFakeClient{connected: make(chan mcpclient.Upstream, 1)}
	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan error, 1)
	go func() {
		done <- runWithContext(ctx, runtimeEnv, client)
	}()

	select {
	case upstream := <-client.connected:
		if upstream.Name != "filesystem" || upstream.Kind != mcpclient.UpstreamStreamableHTTP {
			t.Fatalf("upstream = %#v", upstream)
		}
		if upstream.Endpoint != "http://127.0.0.1:9000/mcp" {
			t.Fatalf("upstream endpoint = %q", upstream.Endpoint)
		}
	case <-time.After(time.Second):
		t.Fatal("runtime did not connect configured upstream")
	}

	cancel()
	select {
	case err := <-done:
		if err != nil {
			t.Fatalf("runWithContext() error = %v", err)
		}
	case <-time.After(time.Second):
		t.Fatal("runtime did not stop after context cancellation")
	}
	if got := client.closed.Load(); got != 1 {
		t.Fatalf("session close count = %d, want 1", got)
	}
}
