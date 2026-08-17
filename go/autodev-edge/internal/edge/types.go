package edge

import (
	"fmt"
	"strings"
	"time"
)

const PublicSchemaVersion = "1"

type ConnectionState string

const (
	ConnectionDisconnected ConnectionState = "disconnected"
	ConnectionConnecting   ConnectionState = "connecting"
	ConnectionReady        ConnectionState = "ready"
	ConnectionDegraded     ConnectionState = "degraded"
)

type ConnectivityStatus struct {
	SchemaVersion string          `json:"schema_version"`
	SourceID      string          `json:"source_id"`
	Kind          string          `json:"kind"`
	State         ConnectionState `json:"state"`
	Protocol      string          `json:"protocol"`
	LatencyMS     *int64          `json:"latency_ms"`
	ObservedAt    string          `json:"observed_at"`
	Detail        string          `json:"detail"`
}

func (s ConnectivityStatus) Validate() error {
	if s.SchemaVersion != PublicSchemaVersion {
		return fmt.Errorf("schema_version must be %q", PublicSchemaVersion)
	}
	if strings.TrimSpace(s.SourceID) == "" {
		return fmt.Errorf("source_id is required")
	}
	if s.Kind != "mcp" && s.Kind != "provider" {
		return fmt.Errorf("kind must be mcp or provider")
	}
	switch s.State {
	case ConnectionDisconnected, ConnectionConnecting, ConnectionReady, ConnectionDegraded:
	default:
		return fmt.Errorf("invalid connection state %q", s.State)
	}
	if strings.TrimSpace(s.Protocol) == "" {
		return fmt.Errorf("protocol is required")
	}
	if s.LatencyMS != nil && *s.LatencyMS < 0 {
		return fmt.Errorf("latency_ms must be non-negative")
	}
	if _, err := time.Parse(time.RFC3339, s.ObservedAt); err != nil {
		return fmt.Errorf("observed_at must be RFC3339: %w", err)
	}
	return nil
}
