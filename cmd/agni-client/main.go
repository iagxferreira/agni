package main

import (
	"bufio"
	"flag"
	"fmt"
	"net"
	"os"
	"strings"

	"github.com/iagxferreira/agni/internal/wire"
)

func main() {
	host := flag.String("host", "127.0.0.1", "server host")
	port := flag.Int("port", 6379, "server port")
	flag.Parse()

	args := flag.Args()
	if len(args) == 0 {
		fmt.Println("usage: agni-client [--host HOST] [--port PORT] COMMAND [args...]")
		os.Exit(1)
	}
	frame := strings.Join(args, " ")

	conn, err := net.Dial("tcp", net.JoinHostPort(*host, fmt.Sprint(*port)))
	if err != nil {
		fmt.Printf("could not connect to %s:%d: %v\n", *host, *port, err)
		os.Exit(1)
	}
	defer conn.Close()

	w := bufio.NewWriter(conn)
	if err := wire.WriteFrame(w, []byte(frame)); err != nil {
		fmt.Printf("failed to send command: %v\n", err)
		os.Exit(1)
	}
	if err := w.Flush(); err != nil {
		fmt.Printf("failed to send command: %v\n", err)
		os.Exit(1)
	}

	response, err := wire.ReadFrame(bufio.NewReader(conn))
	if err != nil {
		fmt.Printf("connection closed with no response: %v\n", err)
		os.Exit(1)
	}
	fmt.Println(string(response))
}
