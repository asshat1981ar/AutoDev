package edge

import (
	"context"
	"testing"
	"time"
)

func TestManagerRunStopsOnContextCancellation(t *testing.T) {
	manager := NewManager()
	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan error, 1)
	go func() { done <- manager.Run(ctx) }()

	cancel()
	select {
	case err := <-done:
		if err != nil {
			t.Fatalf("Run() error = %v", err)
		}
	case <-time.After(time.Second):
		t.Fatal("Run() did not stop after context cancellation")
	}
}

func TestManagerCloseIsIdempotentAndStopsRun(t *testing.T) {
	manager := NewManager()
	done := make(chan error, 1)
	go func() { done <- manager.Run(context.Background()) }()

	if err := manager.Close(); err != nil {
		t.Fatalf("first Close() error = %v", err)
	}
	if err := manager.Close(); err != nil {
		t.Fatalf("second Close() error = %v", err)
	}

	select {
	case err := <-done:
		if err != nil {
			t.Fatalf("Run() error = %v", err)
		}
	case <-time.After(time.Second):
		t.Fatal("Run() did not stop after Close()")
	}
}
