package main

import (
	"bufio"
	"errors"
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"time"

	"agni-go/internal/protocol"
	"agni-go/internal/store"
	"agni-go/internal/wire"
)

const version = "0.1.0"

func main() {
	host := flag.String("host", "127.0.0.1", "listen host")
	port := flag.Int("port", 6379, "listen port")
	flag.Parse()

	addr := net.JoinHostPort(*host, fmt.Sprint(*port))
	ln, err := net.Listen("tcp", addr)
	if err != nil {
		log.Fatalf("listen: %v", err)
	}
	defer ln.Close()

	st := store.New()
	startedAt := time.Now()
	log.Printf("server started service=agni host=%s port=%d version=%s", *host, *port, version)

	serve(ln, st, *host, *port, startedAt)
}

// serve runs the accept loop until the listener is closed.
func serve(ln net.Listener, st *store.Store, host string, port int, startedAt time.Time) {
	for {
		conn, err := ln.Accept()
		if err != nil {
			if errors.Is(err, net.ErrClosed) {
				return
			}
			log.Printf("accept error: %v", err)
			continue
		}
		go handleConnection(conn, st, host, port, startedAt)
	}
}

func handleConnection(conn net.Conn, st *store.Store, host string, port int, startedAt time.Time) {
	defer conn.Close()

	r := bufio.NewReader(conn)
	w := bufio.NewWriter(conn)

	for {
		frame, err := wire.ReadFrame(r)
		if err != nil {
			if !errors.Is(err, io.EOF) {
				log.Printf("connection error: %v", err)
			}
			return
		}

		resp := dispatch(protocol.ParseCommand(frame), st, host, port, startedAt)
		if err := wire.WriteFrame(w, resp.Bytes()); err != nil {
			log.Printf("connection error: %v", err)
			return
		}
		if err := w.Flush(); err != nil {
			log.Printf("connection error: %v", err)
			return
		}
	}
}

func dispatch(cmd protocol.Command, st *store.Store, host string, port int, startedAt time.Time) protocol.Response {
	switch cmd.Kind {
	case protocol.Ping:
		return protocol.Response{Kind: protocol.Pong}
	case protocol.Healthcheck:
		uptimeSecs := int(time.Since(startedAt).Seconds())
		log.Printf("healthcheck ok service=agni host=%s port=%d uptime_secs=%d", host, port, uptimeSecs)
		return protocol.Response{Kind: protocol.Ok}
	case protocol.Get:
		if v, ok := st.Get(cmd.Key); ok {
			return protocol.Response{Kind: protocol.Value, Value: v}
		}
		return protocol.Response{Kind: protocol.Null}
	case protocol.Set:
		st.Set(cmd.Key, cmd.Value)
		return protocol.Response{Kind: protocol.Ok}
	default:
		return protocol.Response{Kind: protocol.Error, Message: fmt.Sprintf("unknown command '%s'", cmd.Message)}
	}
}
