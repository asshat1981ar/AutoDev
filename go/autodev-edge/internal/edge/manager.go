package edge

import (
	"context"
	"fmt"
	"sort"
	"sync"
	"time"

	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/mcpclient"
)

const (
	defaultMaxSessions         = 16
	defaultObservationCapacity = 256
)

var retryDelays = [...]time.Duration{
	250 * time.Millisecond,
	500 * time.Millisecond,
	time.Second,
	2 * time.Second,
	5 * time.Second,
}

type DelayFunc func(context.Context, time.Duration) error

type ManagerOption func(*Manager)

type Manager struct {
	closeOnce sync.Once
	closed    chan struct{}

	client              mcpclient.Client
	maxSessions         int
	observationCapacity int
	delay               DelayFunc
	observations        *statusQueue
	latest              sync.Map
}

func NewManager(options ...ManagerOption) *Manager {
	manager := &Manager{
		closed:              make(chan struct{}),
		maxSessions:         defaultMaxSessions,
		observationCapacity: defaultObservationCapacity,
		delay:               waitContext,
	}
	for _, option := range options {
		if option != nil {
			option(manager)
		}
	}
	if manager.maxSessions <= 0 {
		manager.maxSessions = defaultMaxSessions
	}
	if manager.observationCapacity <= 0 {
		manager.observationCapacity = defaultObservationCapacity
	}
	if manager.delay == nil {
		manager.delay = waitContext
	}
	manager.observations = newStatusQueue(manager.observationCapacity)
	return manager
}

func WithClient(client mcpclient.Client) ManagerOption {
	return func(manager *Manager) {
		manager.client = client
	}
}

func WithLimits(maxSessions, observationCapacity int) ManagerOption {
	return func(manager *Manager) {
		manager.maxSessions = maxSessions
		manager.observationCapacity = observationCapacity
	}
}

func WithDelay(delay DelayFunc) ManagerOption {
	return func(manager *Manager) {
		manager.delay = delay
	}
}

func (m *Manager) Run(ctx context.Context) error {
	select {
	case <-ctx.Done():
		return nil
	case <-m.closed:
		return nil
	}
}

func (m *Manager) RunUpstreams(ctx context.Context, upstreams []mcpclient.Upstream) error {
	if len(upstreams) == 0 {
		return nil
	}
	if m.client == nil {
		return fmt.Errorf("MCP client is required")
	}
	for _, upstream := range upstreams {
		if err := upstream.Validate(); err != nil {
			return fmt.Errorf("upstream %q: %w", upstream.Name, err)
		}
	}

	runCtx, cancel := context.WithCancel(ctx)
	defer cancel()
	go func() {
		select {
		case <-m.closed:
			cancel()
		case <-runCtx.Done():
		}
	}()

	workerCount := min(m.maxSessions, len(upstreams))
	jobs := make(chan mcpclient.Upstream, workerCount)
	var workers sync.WaitGroup
	workers.Add(workerCount)
	for range workerCount {
		go func() {
			defer workers.Done()
			for upstream := range jobs {
				m.runUpstream(runCtx, upstream)
				if runCtx.Err() != nil {
					return
				}
			}
		}()
	}

	feedDone := make(chan struct{})
	go func() {
		defer close(feedDone)
		defer close(jobs)
		for _, upstream := range upstreams {
			select {
			case jobs <- upstream:
			case <-runCtx.Done():
				return
			}
		}
	}()

	workers.Wait()
	<-feedDone
	return nil
}

func (m *Manager) runUpstream(ctx context.Context, upstream mcpclient.Upstream) {
	attempt := 0
	for ctx.Err() == nil {
		m.publishObservation(ConnectivityStatus{
			SchemaVersion: PublicSchemaVersion,
			SourceID:      upstream.Name,
			Kind:          "mcp",
			State:         ConnectionConnecting,
			Protocol:      protocolLabel(upstream),
			ObservedAt:    nowRFC3339(),
			Detail:        "connecting",
		})

		started := time.Now()
		session, err := m.client.Connect(ctx, upstream)
		if err == nil {
			latency := time.Since(started).Milliseconds()
			protocol := session.ProtocolVersion()
			if protocol == "" {
				protocol = protocolLabel(upstream)
			}
			m.publishObservation(ConnectivityStatus{
				SchemaVersion: PublicSchemaVersion,
				SourceID:      upstream.Name,
				Kind:          "mcp",
				State:         ConnectionReady,
				Protocol:      protocol,
				LatencyMS:     &latency,
				ObservedAt:    nowRFC3339(),
				Detail:        "connected",
			})
			<-ctx.Done()
			_ = session.Close()
			m.publishObservation(ConnectivityStatus{
				SchemaVersion: PublicSchemaVersion,
				SourceID:      upstream.Name,
				Kind:          "mcp",
				State:         ConnectionDisconnected,
				Protocol:      protocol,
				ObservedAt:    nowRFC3339(),
				Detail:        "disconnected",
			})
			return
		}
		if ctx.Err() != nil {
			return
		}

		m.publishObservation(ConnectivityStatus{
			SchemaVersion: PublicSchemaVersion,
			SourceID:      upstream.Name,
			Kind:          "mcp",
			State:         ConnectionDegraded,
			Protocol:      protocolLabel(upstream),
			ObservedAt:    nowRFC3339(),
			Detail:        err.Error(),
		})
		delay := retryDelays[min(attempt, len(retryDelays)-1)]
		attempt++
		if err := m.delay(ctx, delay); err != nil && ctx.Err() != nil {
			return
		}
	}
}

func (m *Manager) NextObservation(ctx context.Context) (ConnectivityStatus, error) {
	for {
		if status, ok := m.observations.pop(); ok {
			return status, nil
		}
		select {
		case <-ctx.Done():
			return ConnectivityStatus{}, ctx.Err()
		case <-m.closed:
			return ConnectivityStatus{}, context.Canceled
		case <-m.observations.wake:
		}
	}
}

func (m *Manager) ConnectivitySnapshot() []ConnectivityStatus {
	statuses := make([]ConnectivityStatus, 0)
	m.latest.Range(func(_, value any) bool {
		status, ok := value.(ConnectivityStatus)
		if ok {
			statuses = append(statuses, status)
		}
		return true
	})
	sort.Slice(statuses, func(left, right int) bool {
		return statuses[left].SourceID < statuses[right].SourceID
	})
	return statuses
}

func (m *Manager) publishObservation(status ConnectivityStatus) {
	if status.SourceID != "" {
		m.latest.Store(status.SourceID, status)
	}
	m.observations.publish(status)
}

func (m *Manager) observationQueueLen() int {
	return m.observations.len()
}

func (m *Manager) Close() error {
	m.closeOnce.Do(func() {
		close(m.closed)
	})
	return nil
}

func waitContext(ctx context.Context, duration time.Duration) error {
	timer := time.NewTimer(duration)
	defer timer.Stop()
	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-timer.C:
		return nil
	}
}

func protocolLabel(upstream mcpclient.Upstream) string {
	if upstream.Kind == mcpclient.UpstreamCommand {
		return "stdio"
	}
	return "streamable_http"
}

func nowRFC3339() string {
	return time.Now().UTC().Format(time.RFC3339Nano)
}

type statusQueue struct {
	mu       sync.Mutex
	capacity int
	order    []string
	values   map[string]ConnectivityStatus
	wake     chan struct{}
}

func newStatusQueue(capacity int) *statusQueue {
	return &statusQueue{
		capacity: capacity,
		values:   make(map[string]ConnectivityStatus, capacity),
		wake:     make(chan struct{}, 1),
	}
}

func (q *statusQueue) publish(status ConnectivityStatus) {
	if status.SourceID == "" {
		return
	}
	q.mu.Lock()
	if _, exists := q.values[status.SourceID]; exists {
		q.values[status.SourceID] = status
		q.mu.Unlock()
		q.signal()
		return
	}
	if len(q.order) >= q.capacity {
		oldest := q.order[0]
		q.order = q.order[1:]
		delete(q.values, oldest)
	}
	q.order = append(q.order, status.SourceID)
	q.values[status.SourceID] = status
	q.mu.Unlock()
	q.signal()
}

func (q *statusQueue) pop() (ConnectivityStatus, bool) {
	q.mu.Lock()
	defer q.mu.Unlock()
	if len(q.order) == 0 {
		return ConnectivityStatus{}, false
	}
	sourceID := q.order[0]
	q.order = q.order[1:]
	status := q.values[sourceID]
	delete(q.values, sourceID)
	if len(q.order) > 0 {
		q.signalLocked()
	}
	return status, true
}

func (q *statusQueue) len() int {
	q.mu.Lock()
	defer q.mu.Unlock()
	return len(q.order)
}

func (q *statusQueue) signal() {
	select {
	case q.wake <- struct{}{}:
	default:
	}
}

func (q *statusQueue) signalLocked() {
	select {
	case q.wake <- struct{}{}:
	default:
	}
}
