package main

import (
	"io"
	"net"
	"os"
	"time"

	"github.com/pkg/errors"
)

type stdioAddress struct{}

func (stdioAddress) Network() string {
	return "standard input/output"
}

func (stdioAddress) String() string {
	return "standard input/output"
}

type stdioConnection struct {
	io.Reader
	io.Writer
}

func newStdioConnection() *stdioConnection {
	return &stdioConnection{os.Stdin, os.Stdout}
}

func (c *stdioConnection) Close() error {
	// We can't really close standard input/output, because on many platforms
	// these can't be unblocked on reads and writes, and they'll actually block
	// the call to close. In the case of the agent, where we're just running an
	// endpoint (which will have terminated by the time this connection has
	// closed), this is fine.
	//
	// Since we're not going to close the input/output streams, it doesn't make
	// sense to close the compressor and decompressor. Since these don't leak
	// any resources, this should be fine.
	return errors.New("closing standard input/output connection not allowed")
}

// LocalAddr returns the local address for the connection.
func (c *stdioConnection) LocalAddr() net.Addr {
	return stdioAddress{}
}

// RemoteAddr returns the remote address for the connection.
func (c *stdioConnection) RemoteAddr() net.Addr {
	return stdioAddress{}
}

// SetDeadline sets the read and write deadlines for the connection.
func (c *stdioConnection) SetDeadline(_ time.Time) error {
	return errors.New("deadlines not supported by standard input/output connections")
}

// SetReadDeadline sets the read deadline for the connection.
func (c *stdioConnection) SetReadDeadline(_ time.Time) error {
	return errors.New("read deadlines not supported by standard input/output connections")
}

// SetWriteDeadline sets the write deadline for the connection.
func (c *stdioConnection) SetWriteDeadline(_ time.Time) error {
	return errors.New("write deadlines not supported by standard input/output connections")
}
