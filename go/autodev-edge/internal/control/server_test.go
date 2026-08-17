package control

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/edge"
)

type fakeStatusSource struct {
	statuses []edge.ConnectivityStatus
}

func (f fakeStatusSource) ConnectivitySnapshot() []edge.ConnectivityStatus {
	return append([]edge.ConnectivityStatus(nil), f.statuses...)
}

func readyStatus() edge.ConnectivityStatus {
	latency := int64(12)
	return edge.ConnectivityStatus{
		SchemaVersion: edge.PublicSchemaVersion,
		SourceID:      "mcp-filesystem",
		Kind:          "mcp",
		State:         edge.ConnectionReady,
		Protocol:      "2026-07-28",
		LatencyMS:     &latency,
		ObservedAt:    "2026-08-17T12:06:00Z",
		Detail:        "connected",
	}
}

func TestNewServerRejectsNonLoopbackBind(t *testing.T) {
	if _, err := NewServer("0.0.0.0:8791", "secret", fakeStatusSource{}); err == nil {
		t.Fatal("NewServer() accepted non-loopback bind")
	}
}

func TestConnectivityRequiresBearerAndReturnsPublicFieldsOnly(t *testing.T) {
	server, err := NewServer(
		"127.0.0.1:8791",
		"secret-token",
		fakeStatusSource{statuses: []edge.ConnectivityStatus{readyStatus()}},
	)
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	for _, tc := range []struct {
		name   string
		header string
		status int
	}{
		{name: "missing", status: http.StatusUnauthorized},
		{name: "wrong", header: "Bearer wrong-token", status: http.StatusUnauthorized},
		{name: "correct", header: "Bearer secret-token", status: http.StatusOK},
	} {
		t.Run(tc.name, func(t *testing.T) {
			request := httptest.NewRequest(http.MethodGet, "/api/v1/connectivity", nil)
			if tc.header != "" {
				request.Header.Set("Authorization", tc.header)
			}
			response := httptest.NewRecorder()
			server.Handler().ServeHTTP(response, request)
			if response.Code != tc.status {
				t.Fatalf("status = %d, want %d", response.Code, tc.status)
			}
			if tc.status != http.StatusOK {
				return
			}

			var payload []map[string]any
			if err := json.Unmarshal(response.Body.Bytes(), &payload); err != nil {
				t.Fatalf("decode connectivity response: %v", err)
			}
			if len(payload) != 1 {
				t.Fatalf("connectivity count = %d, want 1", len(payload))
			}
			item := payload[0]
			if item["source_id"] != "mcp-filesystem" || item["state"] != "ready" {
				t.Fatalf("unexpected payload = %#v", item)
			}
			for _, forbidden := range []string{"approval_ref", "authorization", "capabilities", "policy", "task_graph"} {
				if _, exists := item[forbidden]; exists {
					t.Fatalf("response exposed forbidden field %q", forbidden)
				}
			}
		})
	}
}

func TestHealthIsProcessOnlyAndNoMutationRoutesExist(t *testing.T) {
	server, err := NewServer("127.0.0.1:8791", "secret-token", fakeStatusSource{})
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	health := httptest.NewRecorder()
	server.Handler().ServeHTTP(health, httptest.NewRequest(http.MethodGet, "/health", nil))
	if health.Code != http.StatusOK || health.Body.String() != "{\"status\":\"ok\"}\n" {
		t.Fatalf("health response = %d %q", health.Code, health.Body.String())
	}

	request := httptest.NewRequest(http.MethodPost, "/api/v1/objectives", nil)
	request.Header.Set("Authorization", "Bearer secret-token")
	response := httptest.NewRecorder()
	server.Handler().ServeHTTP(response, request)
	if response.Code != http.StatusNotFound {
		t.Fatalf("unexpected mutation route status = %d, want 404", response.Code)
	}
}
