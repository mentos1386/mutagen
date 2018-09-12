package main

import (
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"

	"golang.org/x/net/websocket"

	"github.com/havoc-io/mutagen/pkg/custom"
	promptpkg "github.com/havoc-io/mutagen/pkg/prompt"
)

const (
	urlPrefix          = "example-custom-protocol://localhost:52787/"
	websocketURLPrefix = "ws://localhost:52787/"
	websocketOrigin    = "http://localhost/"
)

// prompt is an example of how to perform promping for credentials from the user
// who is creating or resume the synchronization session.
func prompt(prompt string) (string, error) {
	// Extract the prompting executable path.
	askpass := os.Getenv(custom.AskpassEnvironmentVariable)
	if askpass == "" {
		return "", errors.New("unable to find prompting executable")
	}

	// Ensure that the prompter environment variable is set. We'll need it set
	// to forward it to the prompting executable.
	if os.Getenv(promptpkg.PrompterEnvironmentVariable) == "" {
		return "", errors.New("prompter environment variable not set")
	}

	// Invoke prompting.
	response, err := exec.Command(askpass, prompt).Output()
	if err != nil {
		return "", fmt.Errorf("prompting command failed: %v", err)
	}

	// Trim the newline from the response and return. Additional validation on
	// the response may be warranted depending on your needs.
	return strings.TrimSuffix(string(response), "\n"), nil
}

func main() {
	// Validate arguments and extract the custom URL. The transport executable
	// will be invoked with a single argument: the custom URL.
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "error: invalid number of arguments received")
		os.Exit(1)
	}
	url := os.Args[1]

	// Validate the URL and translate it to the websocket URL that it
	// represents.
	if !strings.HasPrefix(url, urlPrefix) {
		fmt.Fprintln(os.Stderr, "error: invalid URL scheme")
		os.Exit(1)
	} else {
		url = strings.Replace(url, urlPrefix, websocketURLPrefix, 1)
	}

	// Set up the websocket configuration.
	configuration, err := websocket.NewConfig(url, websocketOrigin)
	if err != nil {
		fmt.Fprintln(os.Stderr, "error: unable to create websocket configuration:", err)
		os.Exit(1)
	}

	// Prompt the user for credentials.
	password, err := prompt("Enter the secret password: ")
	if err != nil {
		fmt.Fprintln(os.Stderr, "error: unable to perform prompting:", err)
		os.Exit(1)
	}

	// Set the password as a header on the connection.
	configuration.Header["X-Mutagen-Password"] = []string{password}

	// Create a websocket connection to the endpoint.
	connection, err := websocket.DialConfig(configuration)
	if err != nil {
		fmt.Fprintln(os.Stderr, "error: unable to create websocket connection:", err)
		os.Exit(1)
	}

	// Set the websocket frame payload type to binary.
	connection.PayloadType = websocket.BinaryFrame

	// Forward standard input to the connection and the connection's output to
	// standard output, watching for forwarding errors.
	forwardingErrors := make(chan error, 2)
	go func() {
		_, err := io.Copy(connection, os.Stdin)
		forwardingErrors <- err
	}()
	go func() {
		_, err := io.Copy(os.Stdout, connection)
		forwardingErrors <- err
	}()

	// Wait for a forwarding error. We don't explicitly watch for signals, but
	// one could do that as well.
	<-forwardingErrors
	fmt.Fprintln(os.Stderr, "error: connection forwarding failed")
	os.Exit(1)
}
