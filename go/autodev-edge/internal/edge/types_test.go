package edge

import (
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

func canonicalFixture(t *testing.T, name string) []byte {
	t.Helper()
	_, file, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("runtime.Caller failed")
	}
	path := filepath.Join(filepath.Dir(file), "../../../..", "protocols", "public", "v1", "fixtures", name)
	content, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read canonical fixture: %v", err)
	}
	return content
}

func TestConnectivityStatusMatchesCanonicalFixture(t *testing.T) {
	var status ConnectivityStatus
	if err := json.Unmarshal(canonicalFixture(t, "connectivity-status.ready.json"), &status); err != nil {
		t.Fatalf("unmarshal fixture: %v", err)
	}
	if err := status.Validate(); err != nil {
		t.Fatalf("Validate() error = %v", err)
	}
	if status.SchemaVersion != "1" {
		t.Fatalf("SchemaVersion = %q", status.SchemaVersion)
	}
	if status.SourceID != "mcp-filesystem" {
		t.Fatalf("SourceID = %q", status.SourceID)
	}
	if status.State != ConnectionReady {
		t.Fatalf("State = %q", status.State)
	}
	if status.Protocol != "2026-07-28" {
		t.Fatalf("Protocol = %q", status.Protocol)
	}
	if status.LatencyMS == nil || *status.LatencyMS != 12 {
		t.Fatalf("LatencyMS = %v", status.LatencyMS)
	}
}

func TestConnectivityStatusRejectsUnknownState(t *testing.T) {
	status := ConnectivityStatus{
		SchemaVersion: "1",
		SourceID:      "mcp-filesystem",
		Kind:          "mcp",
		State:         ConnectionState("unknown"),
		Protocol:      "2026-07-28",
		ObservedAt:    "2026-08-17T12:06:00Z",
		Detail:        "invalid state",
	}
	if err := status.Validate(); err == nil {
		t.Fatal("Validate() accepted unknown connection state")
	}
}

func TestConnectivityStatusRejectsAuthorityBearingKind(t *testing.T) {
	status := ConnectivityStatus{
		SchemaVersion: "1",
		SourceID:      "forgecore",
		Kind:          "authorization",
		State:         ConnectionReady,
		Protocol:      "internal",
		ObservedAt:    "2026-08-17T12:06:00Z",
		Detail:        "must not cross the edge boundary",
	}
	if err := status.Validate(); err == nil {
		t.Fatal("Validate() accepted non-public connectivity kind")
	}
}
