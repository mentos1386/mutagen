package main

import (
	"log"
	"net/http"
	"strings"

	"golang.org/x/net/websocket"

	"github.com/havoc-io/mutagen/pkg/remote"
)

const (
	// serverAddress is the address at which our server will serve requests.
	serverAddress = "localhost:52787"

	// secretPassword is the secret password that clients must provide to
	// connect.
	secretPassword = "mutagen"
)

func handleEndpointConnection(connection *websocket.Conn) {
	// Log the connection.
	log.Println("Accepted endpoint connection")

	// Set the websocket frame payload type to binary.
	connection.PayloadType = websocket.BinaryFrame

	// Extract the HTTP request.
	request := connection.Request()

	// Here you can perform any validation, authentication, database lookup,
	// etc. necessary to manage the request. For demonstration purposes, we'll
	// require a secret password ("mutagen"), which will be provided via an HTTP
	// header.
	passwordHeader := request.Header["X-Mutagen-Password"]
	if len(passwordHeader) == 0 {
		log.Println("Request received without password")
		return
	} else if passwordHeader[0] != secretPassword {
		log.Println("Incorrect password received")
		return
	} else {
		log.Println("Client successfully authenticated")
	}

	// For demo purposes, we'll just treat the requested path as if it's a
	// filesystem path. If it starts with "/~", then we'll assume that it's
	// going to be a path requiring home directory lookup.
	path := request.URL.Path
	if strings.HasPrefix(path, "/~") {
		path = path[1:]
	}
	log.Println("Serving endpoint for", path)

	// Serve the connection. For custom protocols, the Mutagen client doesn't
	// send synchronization root information, so it's the responsibility of the
	// endpoint server to determine the appropriate path, and the WithRoot
	// endpoint server option must be specified. Other endpoint server options
	// are also available.
	err := remote.ServeEndpoint(
		connection,
		remote.WithRoot(path),
		// Additional options can be added here, including:
		//
		// remote.WithConnectionValidator(...),
		// remote.WithEndpointOption(local.WithCachePathCallback(...)),
		// remote.WithEndpointOption(local.WithStagingRootCallback(...)),
		// remote.WithEndpointOption(local.WithWatchingMechanism(...)),
	)
	if err != nil {
		log.Println("Endpoint serving failed with error:", err)
	}
}

func main() {
	// Listen for and serve requests. To keep things simple, our HTTP server
	// will just have one endpoint which is a websocket server, but you can just
	// as easily integrate the websocket server into a more complex HTTP setup.
	http.ListenAndServe(serverAddress, &websocket.Server{
		Handler: handleEndpointConnection,
	})
}
