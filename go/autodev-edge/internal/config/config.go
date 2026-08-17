package config

import (
	"encoding/json"
	"fmt"
	"net"
	"net/url"
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
	UpstreamKind        string
	UpstreamEndpoint    string
	UpstreamCommand     string
	UpstreamArgs        []string
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
	upstreamKind := strings.TrimSpace(getenv("AUTODEV_EDGE_UPSTREAM_KIND"))
	if upstreamKind == "" {
		upstreamKind = "streamable_http"
	}
	upstreamEndpoint := strings.TrimSpace(getenv("AUTODEV_EDGE_UPSTREAM_ENDPOINT"))
	upstreamCommand := strings.TrimSpace(getenv("AUTODEV_EDGE_UPSTREAM_COMMAND"))
	upstreamArgs, err := parseArguments(getenv("AUTODEV_EDGE_UPSTREAM_ARGS"))
	if err != nil {
		return Config{}, err
	}
	if err := validateUpstream(upstreamKind, upstreamEndpoint, upstreamCommand); err != nil {
		return Config{}, err
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
		UpstreamKind:        upstreamKind,
		UpstreamEndpoint:    upstreamEndpoint,
		UpstreamCommand:     upstreamCommand,
		UpstreamArgs:        upstreamArgs,
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

func validateUpstream(kind, endpoint, command string) error {
	switch kind {
	case "streamable_http":
		if endpoint == "" {
			return fmt.Errorf("AUTODEV_EDGE_UPSTREAM_ENDPOINT is required for streamable_http")
		}
		parsed, err := url.Parse(endpoint)
		if err != nil || parsed.Host == "" || (parsed.Scheme != "http" && parsed.Scheme != "https") {
			return fmt.Errorf("AUTODEV_EDGE_UPSTREAM_ENDPOINT must be an http or https URL")
		}
		if command != "" {
			return fmt.Errorf("AUTODEV_EDGE_UPSTREAM_COMMAND must be empty for streamable_http")
		}
	case "command":
		if command == "" {
			return fmt.Errorf("AUTODEV_EDGE_UPSTREAM_COMMAND is required for command transport")
		}
		if endpoint != "" {
			return fmt.Errorf("AUTODEV_EDGE_UPSTREAM_ENDPOINT must be empty for command transport")
		}
	default:
		return fmt.Errorf("AUTODEV_EDGE_UPSTREAM_KIND must be streamable_http or command")
	}
	return nil
}

func parseArguments(raw string) ([]string, error) {
	value := strings.TrimSpace(raw)
	if value == "" {
		return nil, nil
	}
	var args []string
	if err := json.Unmarshal([]byte(value), &args); err != nil {
		return nil, fmt.Errorf("AUTODEV_EDGE_UPSTREAM_ARGS must be a JSON string array: %w", err)
	}
	return args, nil
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
