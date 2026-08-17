package mcpclient

import (
	"context"
	"fmt"
	"net/url"
	"os/exec"
	"strings"
	"sync"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

type UpstreamKind string

const (
	UpstreamStreamableHTTP UpstreamKind = "streamable_http"
	UpstreamCommand        UpstreamKind = "command"
)

type Upstream struct {
	Name     string
	Kind     UpstreamKind
	Endpoint string
	Command  string
	Args     []string
}

func (u Upstream) Validate() error {
	if strings.TrimSpace(u.Name) == "" {
		return fmt.Errorf("upstream name is required")
	}
	switch u.Kind {
	case UpstreamStreamableHTTP:
		parsed, err := url.Parse(u.Endpoint)
		if err != nil || parsed.Host == "" || (parsed.Scheme != "http" && parsed.Scheme != "https") {
			return fmt.Errorf("streamable HTTP upstream requires an http or https endpoint")
		}
		if strings.TrimSpace(u.Command) != "" {
			return fmt.Errorf("streamable HTTP upstream must not define a command")
		}
	case UpstreamCommand:
		if strings.TrimSpace(u.Command) == "" {
			return fmt.Errorf("command upstream requires a command")
		}
		if strings.TrimSpace(u.Endpoint) != "" {
			return fmt.Errorf("command upstream must not define an endpoint")
		}
	default:
		return fmt.Errorf("unsupported upstream kind %q", u.Kind)
	}
	return nil
}

type ToolSummary struct {
	Name        string
	Description string
}

type ToolResult struct {
	IsError           bool
	Text              []string
	StructuredContent any
}

type Client interface {
	Connect(ctx context.Context, upstream Upstream) (Session, error)
}

type Session interface {
	ProtocolVersion() string
	ListTools(ctx context.Context) ([]ToolSummary, error)
	CallTool(ctx context.Context, name string, arguments map[string]any) (ToolResult, error)
	Close() error
}

type SDKClient struct {
	client *mcp.Client
}

func NewSDKClient() *SDKClient {
	return &SDKClient{
		client: mcp.NewClient(
			&mcp.Implementation{Name: "autodev-edge", Version: "0.1.0"},
			&mcp.ClientOptions{Capabilities: &mcp.ClientCapabilities{}},
		),
	}
}

func (c *SDKClient) Connect(ctx context.Context, upstream Upstream) (Session, error) {
	if err := upstream.Validate(); err != nil {
		return nil, err
	}
	if err := ctx.Err(); err != nil {
		return nil, err
	}

	var transport mcp.Transport
	switch upstream.Kind {
	case UpstreamStreamableHTTP:
		transport = &mcp.StreamableClientTransport{Endpoint: upstream.Endpoint}
	case UpstreamCommand:
		transport = &mcp.CommandTransport{
			Command: exec.Command(upstream.Command, upstream.Args...),
		}
	default:
		return nil, fmt.Errorf("unsupported upstream kind %q", upstream.Kind)
	}

	session, err := c.client.Connect(ctx, transport, nil)
	if err != nil {
		return nil, err
	}
	return &sdkSession{session: session}, nil
}

type sdkSession struct {
	session   *mcp.ClientSession
	closeOnce sync.Once
	closeErr  error
}

func (s *sdkSession) ProtocolVersion() string {
	result := s.session.InitializeResult()
	if result == nil {
		return ""
	}
	return result.ProtocolVersion
}

func (s *sdkSession) ListTools(ctx context.Context) ([]ToolSummary, error) {
	result, err := s.session.ListTools(ctx, nil)
	if err != nil {
		return nil, err
	}
	tools := make([]ToolSummary, 0, len(result.Tools))
	for _, tool := range result.Tools {
		if tool == nil {
			continue
		}
		tools = append(tools, ToolSummary{Name: tool.Name, Description: tool.Description})
	}
	return tools, nil
}

func (s *sdkSession) CallTool(
	ctx context.Context,
	name string,
	arguments map[string]any,
) (ToolResult, error) {
	if strings.TrimSpace(name) == "" {
		return ToolResult{}, fmt.Errorf("tool name is required")
	}
	result, err := s.session.CallTool(ctx, &mcp.CallToolParams{Name: name, Arguments: arguments})
	if err != nil {
		return ToolResult{}, err
	}
	texts := make([]string, 0, len(result.Content))
	for _, content := range result.Content {
		if text, ok := content.(*mcp.TextContent); ok {
			texts = append(texts, text.Text)
		}
	}
	return ToolResult{
		IsError:           result.IsError,
		Text:              texts,
		StructuredContent: result.StructuredContent,
	}, nil
}

func (s *sdkSession) Close() error {
	s.closeOnce.Do(func() {
		s.closeErr = s.session.Close()
	})
	return s.closeErr
}
