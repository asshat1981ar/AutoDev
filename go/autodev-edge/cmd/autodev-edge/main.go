package main

import (
	"context"
	"log/slog"
	"os"
	"os/signal"
	"syscall"

	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/config"
	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/edge"
)

func main() {
	if err := run(); err != nil {
		slog.Error("AutoDev Edge stopped with error", "error", err)
		os.Exit(1)
	}
}

func run() error {
	cfg, err := config.Load(os.Getenv)
	if err != nil {
		return err
	}

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	manager := edge.NewManager()
	defer manager.Close()

	slog.Info(
		"AutoDev Edge starting",
		"upstream", cfg.UpstreamName,
		"bind", cfg.BindAddress,
		"http_control", cfg.HTTPControl,
		"max_sessions", cfg.MaxSessions,
	)
	return manager.Run(ctx)
}
