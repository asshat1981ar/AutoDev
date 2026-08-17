package mcpclient

import (
	"context"
	"errors"
	"testing"
)

func TestUpstreamValidation(t *testing.T) {
	for _, tc := range []struct {
		name     string
		upstream Upstream
		wantErr  bool
	}{
		{
			name: "streamable http",
			upstream: Upstream{
				Name:     "filesystem",
				Kind:     UpstreamStreamableHTTP,
				Endpoint: "http://127.0.0.1:9000/mcp",
			},
		},
		{
			name: "command",
			upstream: Upstream{
				Name:    "local-tool",
				Kind:    UpstreamCommand,
				Command: "tool-server",
				Args:    []string{"--stdio"},
			},
		},
		{
			name:     "missing name",
			upstream: Upstream{Kind: UpstreamStreamableHTTP, Endpoint: "http://127.0.0.1:9000/mcp"},
			wantErr:  true,
		},
		{
			name:     "remote non-http scheme",
			upstream: Upstream{Name: "bad", Kind: UpstreamStreamableHTTP, Endpoint: "file:///tmp/mcp"},
			wantErr:  true,
		},
		{
			name:     "missing command",
			upstream: Upstream{Name: "bad", Kind: UpstreamCommand},
			wantErr:  true,
		},
		{
			name:     "unknown kind",
			upstream: Upstream{Name: "bad", Kind: UpstreamKind("unknown")},
			wantErr:  true,
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			err := tc.upstream.Validate()
			if tc.wantErr && err == nil {
				t.Fatal("Validate() unexpectedly succeeded")
			}
			if !tc.wantErr && err != nil {
				t.Fatalf("Validate() error = %v", err)
			}
		})
	}
}

func TestSDKClientRejectsCanceledContextBeforeDial(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	client := NewSDKClient()
	_, err := client.Connect(ctx, Upstream{
		Name:     "filesystem",
		Kind:     UpstreamStreamableHTTP,
		Endpoint: "http://127.0.0.1:9000/mcp",
	})
	if !errors.Is(err, context.Canceled) {
		t.Fatalf("Connect() error = %v, want context.Canceled", err)
	}
}
