package protocol

import "fmt"

type ResponseKind int

const (
	Pong ResponseKind = iota
	Ok
	Value
	Null
	Error
)

type Response struct {
	Kind    ResponseKind
	Value   []byte
	Message string
}

func (r Response) Bytes() []byte {
	switch r.Kind {
	case Pong:
		return []byte("PONG")
	case Ok:
		return []byte("OK")
	case Value:
		return r.Value
	case Null:
		return []byte("NULL")
	case Error:
		return []byte(fmt.Sprintf("ERR %s", r.Message))
	default:
		return nil
	}
}
