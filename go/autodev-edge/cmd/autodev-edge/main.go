package main

import (
	"context"
	"log/slog"
	"os"
	"os/signal"
	"syscall"

	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/config"
	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/control"
	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/edge"
	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/mcpclient"
)

func main() {
	if err := run(); err != nil {
		slog.Error("AutoDev Edge stopped with error", "error", err)
		os.Exit(1)
	}
}

func run() error {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	return runWithContext(ctx, os.Getenv, mcpclient.NewSDKClient())
}

func runWithContext(
	ctx context.Context,
	getenv func(string) string,
	client mcpclient.Client,
) error {
	cfg, err := config.Load(getenv)
	if err != nil {
		return err
	}

	upstream := upstreamFromConfig(cfg)
	manager := edge.NewManager(
		edge.WithClient(client),
		edge.WithLimits(cfg.MaxSessions, cfg.ObservationCapacity),
	)
	defer manager.Close()

	slog.Info(
		"AutoDev Edge starting",
		"upstream", cfg.UpstreamName,
		"transport", cfg.UpstreamKind,
		"http_control", cfg.HTTPControl,
		"max_sessions", cfg.MaxSessions,
	)

	if !cfg.HTTPControl {
		return manager.RunUpstreams(ctx, []mcpclient.Upstream{upstream})
	}

	server, err := control.NewServer(cfg.BindAddress, cfg.LocalToken, manager)
	if err != nil {
		return err
	}
	runCtx, cancel := context.WithCancel(ctx)
	defer cancel()

	results := make(chan error, 2)
	go func() {
		results <- manager.RunUpstreams(runCtx, []mcpclient.Upstream{upstream})
	}()
	go func() {
		results <- server.Run(runCtx)
	}()

	first := <-results
	cancel()
	second := <-results
	if first != nil {
		return first
	}
	return second
}

func upstreamFromConfig(cfg config.Config) mcpclient.Upstream {
	kind := mcpclient.UpstreamStreamableHTTP
	if cfg.UpstreamKind == "command" {
		kind = mcpclient.UpstreamCommand
	}
	return mcpclient.Upstream{
		Name:     cfg.UpstreamName,
		Kind:     kind,
		Endpoint: cfg.UpstreamEndpoint,
		Command:  cfg.UpstreamCommand,
		Args:     append([]string(nil), cfg.UpstreamArgs...),
	}
}
