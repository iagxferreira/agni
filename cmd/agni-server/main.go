package main

import (
	"bufio"
	"errors"
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"time"

	"github.com/iagxferreira/agni/config"
	"github.com/iagxferreira/agni/internal/wire"
	"github.com/iagxferreira/agni/protocol"
	"github.com/iagxferreira/agni/store"
)

const version = "0.1.0"

func main() {
	configPath := flag.String("config", "", "path to the YAML configuration file")
	flag.Parse()

	cfg := config.Default()
	if *configPath != "" {
		loaded, err := config.FromFile(*configPath)
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		cfg = loaded
	}

	ln, err := net.Listen("tcp", cfg.Addr())
	if err != nil {
		log.Fatalf("listen: %v", err)
	}
	defer ln.Close()

	st := store.New()
	startedAt := time.Now()
	log.Printf("server started service=agni host=%s port=%d version=%s", cfg.Host, cfg.Port, version)

	serve(ln, st, cfg.Host, cfg.Port, startedAt)
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
