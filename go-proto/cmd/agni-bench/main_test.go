package main

import (
	"testing"
	"time"
)

// TestPercentileEmptyLatencies guards against a real panic: idx computed
// as -1 on an empty slice used to index straight into sorted[-1].
func TestPercentileEmptyLatencies(t *testing.T) {
	r := benchResult{}
	if got := r.percentile(50); got != 0 {
		t.Fatalf("got %v, want 0", got)
	}
}

func TestPercentileSingleLatency(t *testing.T) {
	r := benchResult{latencies: []time.Duration{5 * time.Millisecond}}
	if got := r.percentile(99); got != 5*time.Millisecond {
		t.Fatalf("got %v, want 5ms", got)
	}
}

func TestValidateFlagsRejectsZeroConcurrency(t *testing.T) {
	if err := validateFlags(0, 100); err == nil {
		t.Fatal("expected an error for concurrency=0")
	}
}

func TestValidateFlagsRejectsNegativeConcurrency(t *testing.T) {
	if err := validateFlags(-1, 100); err == nil {
		t.Fatal("expected an error for negative concurrency")
	}
}

func TestValidateFlagsRejectsOpsBelowConcurrency(t *testing.T) {
	if err := validateFlags(50, 10); err == nil {
		t.Fatal("expected an error when ops < concurrency")
	}
}

func TestValidateFlagsAcceptsValidInput(t *testing.T) {
	if err := validateFlags(50, 10000); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}
