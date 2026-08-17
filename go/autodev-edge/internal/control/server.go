package control

import (
	"context"
	"crypto/sha256"
	"crypto/subtle"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"net/http"
	"strings"
	"time"

	"github.com/asshat1981ar/AutoDev/go/autodev-edge/internal/edge"
)

type StatusSource interface {
	ConnectivitySnapshot() []edge.ConnectivityStatus
}

type Server struct {
	bind        string
	tokenDigest [sha256.Size]byte
	source      StatusSource
	handler     http.Handler
}

func NewServer(bind, token string, source StatusSource) (*Server, error) {
	if err := validateLoopbackBind(bind); err != nil {
		return nil, err
	}
	if strings.TrimSpace(token) == "" {
		return nil, fmt.Errorf("local bearer token is required")
	}
	if source == nil {
		return nil, fmt.Errorf("connectivity status source is required")
	}

	server := &Server{
		bind:        bind,
		tokenDigest: sha256.Sum256([]byte(token)),
		source:      source,
	}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", server.health)
	mux.Handle("GET /api/v1/connectivity", server.requireBearer(http.HandlerFunc(server.connectivity)))
	server.handler = mux
	return server, nil
}

func (s *Server) Handler() http.Handler {
	return s.handler
}

func (s *Server) Run(ctx context.Context) error {
	listener, err := net.Listen("tcp", s.bind)
	if err != nil {
		return err
	}
	defer listener.Close()

	httpServer := &http.Server{
		Handler:           s.handler,
		ReadHeaderTimeout: 2 * time.Second,
		WriteTimeout:      10 * time.Second,
		IdleTimeout:       30 * time.Second,
	}
	serveDone := make(chan error, 1)
	go func() {
		err := httpServer.Serve(listener)
		if errors.Is(err, http.ErrServerClosed) {
			err = nil
		}
		serveDone <- err
	}()

	select {
	case err := <-serveDone:
		return err
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := httpServer.Shutdown(shutdownCtx); err != nil {
			return err
		}
		return <-serveDone
	}
}

func (s *Server) health(response http.ResponseWriter, request *http.Request) {
	writeJSON(response, http.StatusOK, map[string]string{"status": "ok"})
}

func (s *Server) connectivity(response http.ResponseWriter, request *http.Request) {
	writeJSON(response, http.StatusOK, s.source.ConnectivitySnapshot())
}

func (s *Server) requireBearer(next http.Handler) http.Handler {
	return http.HandlerFunc(func(response http.ResponseWriter, request *http.Request) {
		presented, ok := strings.CutPrefix(request.Header.Get("Authorization"), "Bearer ")
		if !ok || strings.TrimSpace(presented) == "" || !s.matchesToken(presented) {
			writeJSON(response, http.StatusUnauthorized, map[string]string{"error": "unauthorized"})
			return
		}
		next.ServeHTTP(response, request)
	})
}

func (s *Server) matchesToken(token string) bool {
	presented := sha256.Sum256([]byte(token))
	return subtle.ConstantTimeCompare(s.tokenDigest[:], presented[:]) == 1
}

func validateLoopbackBind(address string) error {
	host, port, err := net.SplitHostPort(address)
	if err != nil {
		return fmt.Errorf("bind must be host:port: %w", err)
	}
	if port == "" {
		return fmt.Errorf("bind port is required")
	}
	if host == "localhost" {
		return nil
	}
	ip := net.ParseIP(host)
	if ip == nil || !ip.IsLoopback() {
		return fmt.Errorf("bind must use a loopback address")
	}
	return nil
}

func writeJSON(response http.ResponseWriter, status int, value any) {
	response.Header().Set("Content-Type", "application/json")
	response.WriteHeader(status)
	_ = json.NewEncoder(response).Encode(value)
}
