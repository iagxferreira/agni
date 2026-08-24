package protocol

import (
	"bytes"
	"testing"
)

func TestPongBytes(t *testing.T) {
	if got := (Response{Kind: Pong}).Bytes(); !bytes.Equal(got, []byte("PONG")) {
		t.Fatalf("got %q, want PONG", got)
	}
}

func TestValueBytesReturnsRawValue(t *testing.T) {
	r := Response{Kind: Value, Value: []byte("hello")}
	if got := r.Bytes(); !bytes.Equal(got, []byte("hello")) {
		t.Fatalf("got %q, want hello", got)
	}
}

func TestErrorBytesIncludesMessage(t *testing.T) {
	r := Response{Kind: Error, Message: "unknown command 'FOO'"}
	if got := string(r.Bytes()); got != "ERR unknown command 'FOO'" {
		t.Fatalf("got %q", got)
	}
}
