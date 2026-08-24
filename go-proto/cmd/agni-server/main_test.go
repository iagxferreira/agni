package main

import (
	"bufio"
	"net"
	"testing"
	"time"

	"agni-go/internal/protocol"
	"agni-go/internal/store"
	"agni-go/internal/wire"
)

func TestDispatch(t *testing.T) {
	st := store.New()
	st.Set("existing", []byte("value"))
	startedAt := time.Now()

	cases := []struct {
		name string
		cmd  string
		want string
	}{
		{"ping", "PING", "PONG"},
		{"healthcheck", "HEALTHCHECK", "OK"},
		{"get hit", "GET existing", "value"},
		{"get miss", "GET missing", "NULL"},
		{"set", "SET newkey newvalue", "OK"},
		{"unknown", "BOGUS", "ERR unknown command 'BOGUS'"},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			resp := dispatch(protocol.ParseCommand([]byte(tc.cmd)), st, "127.0.0.1", 0, startedAt)
			if got := string(resp.Bytes()); got != tc.want {
				t.Fatalf("got %q, want %q", got, tc.want)
			}
		})
	}
}

// TestServerEndToEnd binds a real listener and drives it over an actual TCP
// connection, covering the wire-level path (framing + dispatch) that the
// benchmark tool exercises only for PING/SET/GET — HEALTHCHECK and the
// unknown-command error path go untouched by agni-bench.
func TestServerEndToEnd(t *testing.T) {
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("listen: %v", err)
	}
	defer ln.Close()

	st := store.New()
	go serve(ln, st, "127.0.0.1", 0, time.Now())

	conn, err := net.Dial("tcp", ln.Addr().String())
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	defer conn.Close()

	r := bufio.NewReader(conn)
	w := bufio.NewWriter(conn)

	send := func(t *testing.T, cmd, want string) {
		t.Helper()
		if err := wire.WriteFrame(w, []byte(cmd)); err != nil {
			t.Fatalf("WriteFrame(%q): %v", cmd, err)
		}
		if err := w.Flush(); err != nil {
			t.Fatalf("flush: %v", err)
		}
		frame, err := wire.ReadFrame(r)
		if err != nil {
			t.Fatalf("ReadFrame after %q: %v", cmd, err)
		}
		if got := string(frame); got != want {
			t.Fatalf("%q => %q, want %q", cmd, got, want)
		}
	}

	send(t, "PING", "PONG")
	send(t, "HEALTHCHECK", "OK")
	send(t, "SET foo bar", "OK")
	send(t, "GET foo", "bar")
	send(t, "GET missing", "NULL")
	send(t, "BOGUS", "ERR unknown command 'BOGUS'")
}
