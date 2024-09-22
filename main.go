package main

import (
	"github.com/bwaklog/bee/cmd/conn"
)

func main() {
	conn, err := conn.NewListiner("localhost", 8080)
	if err != nil {
		conn.Logger.Fatalln(err)
	}

	conn.Logger.Println(conn)
}
