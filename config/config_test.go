package config

import (
	"errors"
	"os"
	"path/filepath"
	"testing"
)

func TestDefaultHostAndPort(t *testing.T) {
	c := Default()
	if c.Host != "127.0.0.1" || c.Port != 6379 {
		t.Fatalf("got %+v, want 127.0.0.1:6379", c)
	}
}

func TestAddrFormatsHostAndPort(t *testing.T) {
	c := Config{Host: "0.0.0.0", Port: 7000}
	if got := c.Addr(); got != "0.0.0.0:7000" {
		t.Fatalf("got %q, want 0.0.0.0:7000", got)
	}
}

func TestFromFileParsesValidYAML(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "config.yml")
	writeFile(t, path, "host: 0.0.0.0\nport: 7000\n")

	c, err := FromFile(path)
	if err != nil {
		t.Fatalf("FromFile: %v", err)
	}
	if c.Host != "0.0.0.0" || c.Port != 7000 {
		t.Fatalf("got %+v, want 0.0.0.0:7000", c)
	}
}

func TestFromFileReturnsIOErrorWhenMissing(t *testing.T) {
	dir := t.TempDir()
	missing := filepath.Join(dir, "missing.yml")

	_, err := FromFile(missing)
	var ioErr *IOError
	if !errors.As(err, &ioErr) {
		t.Fatalf("got %v (%T), want *IOError", err, err)
	}
}

func TestFromFileReturnsParseErrorOnInvalidYAML(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "config.yml")
	writeFile(t, path, "not: [valid")

	_, err := FromFile(path)
	var parseErr *ParseError
	if !errors.As(err, &parseErr) {
		t.Fatalf("got %v (%T), want *ParseError", err, err)
	}
}

func writeFile(t *testing.T, path, contents string) {
	t.Helper()
	if err := os.WriteFile(path, []byte(contents), 0o600); err != nil {
		t.Fatalf("writing test fixture: %v", err)
	}
}
