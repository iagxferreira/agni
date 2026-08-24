package wire

import (
	"bytes"
	"errors"
	"io"
	"testing"
)

func TestWriteThenReadFrameRoundTrips(t *testing.T) {
	var buf bytes.Buffer
	if err := WriteFrame(&buf, []byte("hello")); err != nil {
		t.Fatalf("WriteFrame: %v", err)
	}

	got, err := ReadFrame(&buf)
	if err != nil {
		t.Fatalf("ReadFrame: %v", err)
	}
	if !bytes.Equal(got, []byte("hello")) {
		t.Fatalf("got %q, want %q", got, "hello")
	}
}

func TestReadFrameEmptyPayload(t *testing.T) {
	var buf bytes.Buffer
	if err := WriteFrame(&buf, []byte{}); err != nil {
		t.Fatalf("WriteFrame: %v", err)
	}

	got, err := ReadFrame(&buf)
	if err != nil {
		t.Fatalf("ReadFrame: %v", err)
	}
	if len(got) != 0 {
		t.Fatalf("got %q, want empty", got)
	}
}

func TestReadFrameCleanCloseReturnsEOF(t *testing.T) {
	_, err := ReadFrame(bytes.NewReader(nil))
	if !errors.Is(err, io.EOF) {
		t.Fatalf("got %v, want io.EOF", err)
	}
}

func TestReadFrameTruncatedMidFrameReturnsUnexpectedEOF(t *testing.T) {
	var buf bytes.Buffer
	if err := WriteFrame(&buf, []byte("hello")); err != nil {
		t.Fatalf("WriteFrame: %v", err)
	}
	truncated := buf.Bytes()[:len(buf.Bytes())-2] // drop last 2 payload bytes

	_, err := ReadFrame(bytes.NewReader(truncated))
	if !errors.Is(err, io.ErrUnexpectedEOF) {
		t.Fatalf("got %v, want io.ErrUnexpectedEOF", err)
	}
}

func TestReadFrameRejectsOversizedLength(t *testing.T) {
	var buf bytes.Buffer
	oversized := uint32(MaxFrameLength + 1)
	lenBuf := []byte{byte(oversized >> 24), byte(oversized >> 16), byte(oversized >> 8), byte(oversized)}
	buf.Write(lenBuf)

	_, err := ReadFrame(&buf)
	if err == nil {
		t.Fatal("expected an error for an oversized frame length")
	}
}
