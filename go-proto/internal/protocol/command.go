package protocol

import "strings"

type CommandKind int

const (
	Ping CommandKind = iota
	Healthcheck
	Get
	Set
	Unknown
)

// Command mirrors agni-core's sealed Command class as a tagged struct:
// Key/Value/Message are only populated for the variants that use them.
type Command struct {
	Kind    CommandKind
	Key     string
	Value   []byte
	Message string
}

// ParseCommand mirrors the Rust/Kotlin parser: split on the first two
// spaces only, so a SET value can contain spaces of its own.
func ParseCommand(data []byte) Command {
	parts := strings.SplitN(string(data), " ", 3)
	name := strings.ToUpper(parts[0])

	switch name {
	case "PING":
		return Command{Kind: Ping}
	case "HEALTHCHECK":
		return Command{Kind: Healthcheck}
	case "GET":
		if len(parts) < 2 {
			return Command{Kind: Unknown, Message: "GET requires a key"}
		}
		return Command{Kind: Get, Key: strings.TrimSpace(parts[1])}
	case "SET":
		if len(parts) < 3 {
			return Command{Kind: Unknown, Message: "SET requires a key and value"}
		}
		return Command{
			Kind:  Set,
			Key:   strings.TrimSpace(parts[1]),
			Value: []byte(strings.TrimSpace(parts[2])),
		}
	default:
		return Command{Kind: Unknown, Message: name}
	}
}
