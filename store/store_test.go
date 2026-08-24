package store

import (
	"fmt"
	"strings"
	"sync"
	"testing"
)

func TestSetThenGetReturnsValue(t *testing.T) {
	s := New()
	s.Set("key", []byte("value"))

	got, ok := s.Get("key")
	if !ok {
		t.Fatal("expected key to be present")
	}
	if string(got) != "value" {
		t.Fatalf("got %q, want %q", got, "value")
	}
}

func TestGetMissingKeyReturnsFalse(t *testing.T) {
	s := New()

	_, ok := s.Get("missing")
	if ok {
		t.Fatal("expected missing key to return false")
	}
}

func TestSetOverwritesExistingValue(t *testing.T) {
	s := New()
	s.Set("key", []byte("first"))
	s.Set("key", []byte("second"))

	got, _ := s.Get("key")
	if string(got) != "second" {
		t.Fatalf("got %q, want %q", got, "second")
	}
}

func TestDeleteRemovesKey(t *testing.T) {
	s := New()
	s.Set("key", []byte("value"))

	if !s.Delete("key") {
		t.Fatal("expected delete to report existing key")
	}
	if _, ok := s.Get("key"); ok {
		t.Fatal("expected key to be gone after delete")
	}
}

func TestDeleteMissingKeyReturnsFalse(t *testing.T) {
	s := New()

	if s.Delete("missing") {
		t.Fatal("expected delete of missing key to return false")
	}
}

func TestGetAsJSONIncludesBase64Value(t *testing.T) {
	s := New()
	s.Set("key", []byte("value"))

	j, ok := s.GetAsJSON("key")
	if !ok {
		t.Fatal("expected key to be present")
	}
	if want := `"key":"key"`; !strings.Contains(j, want) {
		t.Fatalf("json %q missing %q", j, want)
	}
	if want := `"value":"dmFsdWU="`; !strings.Contains(j, want) {
		t.Fatalf("json %q missing base64 value %q", j, want)
	}
}

// TestConcurrentSetAndGet exercises Store under contention from many
// goroutines at once — run with -race to catch data races on the map.
func TestConcurrentSetAndGet(t *testing.T) {
	s := New()
	const goroutines = 50
	const opsEach = 200

	var wg sync.WaitGroup
	for g := 0; g < goroutines; g++ {
		wg.Add(1)
		go func(g int) {
			defer wg.Done()
			for i := 0; i < opsEach; i++ {
				key := fmt.Sprintf("key:%d", i%20)
				s.Set(key, []byte(fmt.Sprintf("g%d-i%d", g, i)))
				s.Get(key)
			}
		}(g)
	}
	wg.Wait()
}
