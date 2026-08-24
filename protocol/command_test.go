package protocol

import (
	"bytes"
	"testing"
)

func TestParsePing(t *testing.T) {
	cmd := ParseCommand([]byte("PING"))
	if cmd.Kind != Ping {
		t.Fatalf("got kind %v, want Ping", cmd.Kind)
	}
}

func TestParseGetWithKey(t *testing.T) {
	cmd := ParseCommand([]byte("GET mykey"))
	if cmd.Kind != Get || cmd.Key != "mykey" {
		t.Fatalf("got %+v, want Get{Key: mykey}", cmd)
	}
}

func TestParseGetWithoutKeyIsUnknown(t *testing.T) {
	cmd := ParseCommand([]byte("GET"))
	if cmd.Kind != Unknown {
		t.Fatalf("got kind %v, want Unknown", cmd.Kind)
	}
}

func TestParseSetWithSpacesInValue(t *testing.T) {
	cmd := ParseCommand([]byte("SET mykey hello world"))
	if cmd.Kind != Set || cmd.Key != "mykey" || !bytes.Equal(cmd.Value, []byte("hello world")) {
		t.Fatalf("got %+v, want Set{Key: mykey, Value: hello world}", cmd)
	}
}

func TestParseUnknownCommand(t *testing.T) {
	cmd := ParseCommand([]byte("FOO bar"))
	if cmd.Kind != Unknown || cmd.Message != "FOO" {
		t.Fatalf("got %+v, want Unknown{Message: FOO}", cmd)
	}
}

func TestParseCaseInsensitive(t *testing.T) {
	cmd := ParseCommand([]byte("ping"))
	if cmd.Kind != Ping {
		t.Fatalf("got kind %v, want Ping", cmd.Kind)
	}
}
