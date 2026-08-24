package wire

import (
	"encoding/binary"
	"fmt"
	"io"
)

// MaxFrameLength matches tokio_util's LengthDelimitedCodec default (and the
// Kotlin server's MAX_FRAME_LENGTH), so a corrupt/hostile length prefix
// can't force an unbounded allocation.
const MaxFrameLength = 8 * 1024 * 1024

// ReadFrame reads a 4-byte big-endian length prefix followed by that many
// bytes. A clean disconnect between frames surfaces as io.EOF; a disconnect
// mid-frame surfaces as io.ErrUnexpectedEOF — callers should only treat the
// former as a silent close.
func ReadFrame(r io.Reader) ([]byte, error) {
	var lenBuf [4]byte
	if _, err := io.ReadFull(r, lenBuf[:]); err != nil {
		return nil, err
	}

	length := binary.BigEndian.Uint32(lenBuf[:])
	if length > MaxFrameLength {
		return nil, fmt.Errorf("frame too large: %d bytes", length)
	}

	frame := make([]byte, length)
	if _, err := io.ReadFull(r, frame); err != nil {
		return nil, err
	}
	return frame, nil
}

func WriteFrame(w io.Writer, payload []byte) error {
	var lenBuf [4]byte
	binary.BigEndian.PutUint32(lenBuf[:], uint32(len(payload)))
	if _, err := w.Write(lenBuf[:]); err != nil {
		return err
	}
	_, err := w.Write(payload)
	return err
}
