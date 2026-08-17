package config

import (
	"fmt"
	"net"
	"strconv"
	"strings"
)

const (
	defaultBindAddress    = "127.0.0.1:8791"
	defaultObservationCap = 256
	defaultMaxSessions    = 16
)

type Config struct {
	BindAddress         string
	ObservationCapacity int
	MaxSessions         int
	UpstreamName        string
	HTTPControl         bool
	LocalToken          string
}

func Load(getenv func(string) string) (Config, error) {
	if getenv == nil {
		return Config{}, fmt.Errorf("getenv function is required")
	}

	bindAddress := strings.TrimSpace(getenv("AUTODEV_EDGE_BIND"))
	if bindAddress == "" {
		bindAddress = defaultBindAddress
	}
	if err := validateLoopbackBind(bindAddress); err != nil {
		return Config{}, err
	}

	observationCapacity, err := positiveInt(getenv("AUTODEV_EDGE_OBSERVATION_CAPACITY"), defaultObservationCap, "AUTODEV_EDGE_OBSERVATION_CAPACITY")
	if err != nil {
		return Config{}, err
	}
	maxSessions, err := positiveInt(getenv("AUTODEV_EDGE_MAX_SESSIONS"), defaultMaxSessions, "AUTODEV_EDGE_MAX_SESSIONS")
	if err != nil {
		return Config{}, err
	}

	upstreamName := strings.TrimSpace(getenv("AUTODEV_EDGE_UPSTREAM_NAME"))
	if upstreamName == "" {
		return Config{}, fmt.Errorf("AUTODEV_EDGE_UPSTREAM_NAME is required")
	}

	httpControl, err := boolean(getenv("AUTODEV_EDGE_HTTP_CONTROL"))
	if err != nil {
		return Config{}, fmt.Errorf("AUTODEV_EDGE_HTTP_CONTROL: %w", err)
	}
	localToken := strings.TrimSpace(getenv("AUTODEV_EDGE_LOCAL_TOKEN"))
	if httpControl && localToken == "" {
		return Config{}, fmt.Errorf("AUTODEV_EDGE_LOCAL_TOKEN is required when HTTP control is enabled")
	}

	return Config{
		BindAddress:         bindAddress,
		ObservationCapacity: observationCapacity,
		MaxSessions:         maxSessions,
		UpstreamName:        upstreamName,
		HTTPControl:         httpControl,
		LocalToken:          localToken,
	}, nil
}

func validateLoopbackBind(address string) error {
	host, port, err := net.SplitHostPort(address)
	if err != nil {
		return fmt.Errorf("AUTODEV_EDGE_BIND must be host:port: %w", err)
	}
	if port == "" {
		return fmt.Errorf("AUTODEV_EDGE_BIND port is required")
	}
	if host == "localhost" {
		return nil
	}
	ip := net.ParseIP(host)
	if ip == nil || !ip.IsLoopback() {
		return fmt.Errorf("AUTODEV_EDGE_BIND must use a loopback address")
	}
	return nil
}

func positiveInt(raw string, fallback int, name string) (int, error) {
	value := strings.TrimSpace(raw)
	if value == "" {
		return fallback, nil
	}
	parsed, err := strconv.Atoi(value)
	if err != nil || parsed <= 0 {
		return 0, fmt.Errorf("%s must be a positive integer", name)
	}
	return parsed, nil
}

func boolean(raw string) (bool, error) {
	switch strings.ToLower(strings.TrimSpace(raw)) {
	case "", "0", "false", "no", "off":
		return false, nil
	case "1", "true", "yes", "on":
		return true, nil
	default:
		return false, fmt.Errorf("must be a boolean")
	}
}
