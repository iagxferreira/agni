package store

import (
	"crypto/rand"
	"encoding/json"
	"fmt"
)

// Entry mirrors agni-core's Entry: an id, key, and value. encoding/json
// base64-encodes []byte fields by default, so ToJSON needs no custom
// serializer to match the Kotlin/Rust wire format for getAsJson.
type Entry struct {
	ID    string `json:"id"`
	Key   string `json:"key"`
	Value []byte `json:"value"`
}

func NewEntry(key string, value []byte) Entry {
	return Entry{ID: newUUID(), Key: key, Value: value}
}

func (e Entry) ToJSON() (string, error) {
	b, err := json.Marshal(e)
	if err != nil {
		return "", err
	}
	return string(b), nil
}

func newUUID() string {
	b := make([]byte, 16)
	_, _ = rand.Read(b)
	b[6] = (b[6] & 0x0f) | 0x40
	b[8] = (b[8] & 0x3f) | 0x80
	return fmt.Sprintf("%x-%x-%x-%x-%x", b[0:4], b[4:6], b[6:8], b[8:10], b[10:16])
}
