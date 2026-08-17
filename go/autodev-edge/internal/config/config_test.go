package config

import (
	"reflect"
	"testing"
)

func env(values map[string]string) func(string) string {
	return func(key string) string { return values[key] }
}

func validEnv() map[string]string {
	return map[string]string{
		"AUTODEV_EDGE_UPSTREAM_NAME":        "filesystem",
		"AUTODEV_EDGE_UPSTREAM_KIND":        "streamable_http",
		"AUTODEV_EDGE_UPSTREAM_ENDPOINT":    "http://127.0.0.1:9000/mcp",
		"AUTODEV_EDGE_OBSERVATION_CAPACITY": "256",
		"AUTODEV_EDGE_MAX_SESSIONS":         "16",
	}
}

func TestLoadUsesLoopbackDefaults(t *testing.T) {
	cfg, err := Load(env(validEnv()))
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}
	if cfg.BindAddress != "127.0.0.1:8791" {
		t.Fatalf("BindAddress = %q", cfg.BindAddress)
	}
	if cfg.ObservationCapacity != 256 {
		t.Fatalf("ObservationCapacity = %d", cfg.ObservationCapacity)
	}
	if cfg.MaxSessions != 16 {
		t.Fatalf("MaxSessions = %d", cfg.MaxSessions)
	}
	if cfg.UpstreamName != "filesystem" || cfg.UpstreamKind != "streamable_http" {
		t.Fatalf("upstream = %q/%q", cfg.UpstreamName, cfg.UpstreamKind)
	}
	if cfg.UpstreamEndpoint != "http://127.0.0.1:9000/mcp" {
		t.Fatalf("UpstreamEndpoint = %q", cfg.UpstreamEndpoint)
	}
	if cfg.HTTPControl {
		t.Fatal("HTTPControl must default to false")
	}
}

func TestLoadRejectsNonLoopbackBind(t *testing.T) {
	values := validEnv()
	values["AUTODEV_EDGE_BIND"] = "0.0.0.0:8791"
	if _, err := Load(env(values)); err == nil {
		t.Fatal("Load() accepted non-loopback bind")
	}
}

func TestLoadRejectsNonPositiveCapacities(t *testing.T) {
	for _, tc := range []struct {
		name string
		key  string
	}{
		{name: "observation capacity", key: "AUTODEV_EDGE_OBSERVATION_CAPACITY"},
		{name: "session capacity", key: "AUTODEV_EDGE_MAX_SESSIONS"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			values := validEnv()
			values[tc.key] = "0"
			if _, err := Load(env(values)); err == nil {
				t.Fatalf("Load() accepted %s=0", tc.key)
			}
		})
	}
}

func TestLoadRequiresUpstreamName(t *testing.T) {
	values := validEnv()
	delete(values, "AUTODEV_EDGE_UPSTREAM_NAME")
	if _, err := Load(env(values)); err == nil {
		t.Fatal("Load() accepted empty upstream name")
	}
}

func TestLoadRequiresStreamableHTTPEndpoint(t *testing.T) {
	values := validEnv()
	delete(values, "AUTODEV_EDGE_UPSTREAM_ENDPOINT")
	if _, err := Load(env(values)); err == nil {
		t.Fatal("Load() accepted streamable_http without endpoint")
	}
}

func TestLoadAcceptsCommandUpstreamWithJSONArguments(t *testing.T) {
	values := validEnv()
	values["AUTODEV_EDGE_UPSTREAM_KIND"] = "command"
	delete(values, "AUTODEV_EDGE_UPSTREAM_ENDPOINT")
	values["AUTODEV_EDGE_UPSTREAM_COMMAND"] = "tool-server"
	values["AUTODEV_EDGE_UPSTREAM_ARGS"] = `["--stdio","--root","/workspace"]`

	cfg, err := Load(env(values))
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}
	if cfg.UpstreamCommand != "tool-server" {
		t.Fatalf("UpstreamCommand = %q", cfg.UpstreamCommand)
	}
	if !reflect.DeepEqual(cfg.UpstreamArgs, []string{"--stdio", "--root", "/workspace"}) {
		t.Fatalf("UpstreamArgs = %#v", cfg.UpstreamArgs)
	}
}

func TestLoadRejectsMalformedCommandArguments(t *testing.T) {
	values := validEnv()
	values["AUTODEV_EDGE_UPSTREAM_KIND"] = "command"
	delete(values, "AUTODEV_EDGE_UPSTREAM_ENDPOINT")
	values["AUTODEV_EDGE_UPSTREAM_COMMAND"] = "tool-server"
	values["AUTODEV_EDGE_UPSTREAM_ARGS"] = `--stdio`
	if _, err := Load(env(values)); err == nil {
		t.Fatal("Load() accepted non-JSON command arguments")
	}
}

func TestLoadRequiresTokenWhenHTTPControlEnabled(t *testing.T) {
	values := validEnv()
	values["AUTODEV_EDGE_HTTP_CONTROL"] = "1"
	if _, err := Load(env(values)); err == nil {
		t.Fatal("Load() accepted HTTP control without token")
	}
}
