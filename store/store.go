package store

import "sync"

// Store is a mutex-guarded map with five operations: set, get, delete, and
// getAsJson. This mirrors the original HashMap+RwLock design (see
// BENCHMARK.md) rather than the sharded DashMap it was later replaced with
// — a starting point, not a final answer on contention.
type Store struct {
	mu   sync.RWMutex
	data map[string]Entry
}

func New() *Store {
	return &Store{data: make(map[string]Entry)}
}

func (s *Store) Set(key string, value []byte) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.data[key] = NewEntry(key, value)
}

func (s *Store) Get(key string) ([]byte, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	e, ok := s.data[key]
	if !ok {
		return nil, false
	}
	return e.Value, true
}

func (s *Store) Delete(key string) bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, ok := s.data[key]; !ok {
		return false
	}
	delete(s.data, key)
	return true
}

func (s *Store) GetAsJSON(key string) (string, bool) {
	s.mu.RLock()
	e, ok := s.data[key]
	s.mu.RUnlock()
	if !ok {
		return "", false
	}
	j, err := e.ToJSON()
	if err != nil {
		return "", false
	}
	return j, true
}
