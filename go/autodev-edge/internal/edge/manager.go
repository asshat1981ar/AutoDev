package edge

import (
	"context"
	"sync"
)

type Manager struct {
	closeOnce sync.Once
	closed    chan struct{}
}

func NewManager() *Manager {
	return &Manager{closed: make(chan struct{})}
}

func (m *Manager) Run(ctx context.Context) error {
	select {
	case <-ctx.Done():
		return nil
	case <-m.closed:
		return nil
	}
}

func (m *Manager) Close() error {
	m.closeOnce.Do(func() {
		close(m.closed)
	})
	return nil
}
