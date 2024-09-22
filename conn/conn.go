package conn

import (
	"log"
	"net"
	"os"
)

type Conn struct {
	conn *net.Conn

	// Connection error logger
	Logger *log.Logger
}

func NewListiner(addr string, port uint) (Conn, error) {
	return Conn{
		conn:   nil,
		Logger: log.New(os.Stderr, "[Conn] ERROR: ", log.LstdFlags),
	}, nil
}
