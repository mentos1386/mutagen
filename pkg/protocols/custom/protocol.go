package custom

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"time"
	"unicode/utf8"

	"github.com/pkg/errors"

	"github.com/havoc-io/mutagen/pkg/process"
	"github.com/havoc-io/mutagen/pkg/remote"
	"github.com/havoc-io/mutagen/pkg/session"
	urlpkg "github.com/havoc-io/mutagen/pkg/url"
)

const (
	// transportExecutableNameTemplate is the formatting template to use to
	// identify the appropriate transport executable for a custom protocol
	// scheme. The scheme should be converted to lower case before formatting.
	transportExecutableNameTemplate = "mutagen-transport-%s"
)

// protocolHandler implements the session.ProtocolHandler interface for
// connecting to custom endpoints using protocol handler executables.
type protocolHandler struct{}

// Dial connects to a Docker endpoint.
func (h *protocolHandler) Dial(
	url *urlpkg.URL,
	prompter,
	session string,
	version session.Version,
	configuration *session.Configuration,
	alpha bool,
) (session.Endpoint, error) {
	// Verify that the URL is of the correct protocol.
	if url.Protocol != urlpkg.Protocol_Custom {
		panic("non-custom URL dispatched to custom protocol handler")
	}

	// Extract the scheme from the protocol.
	scheme, err := urlpkg.ParseCustomScheme(url.Path)
	if err != nil {
		return nil, errors.Wrap(err, "unable to parse custom URL scheme")
	} else if scheme == "" {
		return nil, errors.New("empty scheme parsed from custom URL")
	}

	// Convert the scheme to lower case, since URL schemes don't distinguish
	// between case. We treat this as the canonical form of the scheme for
	// protocol handler executable identification.
	scheme = strings.ToLower(scheme)

	// Compute the expected name of the transport executable.
	transportExecutableName := fmt.Sprintf(transportExecutableNameTemplate, scheme)

	// Attempt to find the appropriate protocol handler executable.
	transportExecutablePath, err := exec.LookPath(transportExecutableName)
	if err != nil {
		return nil, errors.Wrap(err, "unable to find transport executable")
	}

	// Create a command to run the transport executable.
	transport := exec.Command(transportExecutablePath, url.Path)

	// Create a copy of the current environment.
	environment := os.Environ()

	// Set prompting environment variables. We do this even if there isn't a
	// prompter available, so that (a) no existing prompter environment variable
	// is propagated and (b) the transport executable can see that no prompter
	// is available.
	environment, err = setPrompterVariables(environment, prompter)
	if err != nil {
		return nil, errors.Wrap(err, "unable to create prompter environment")
	}

	// Set the environment.
	transport.Env = environment

	// Create a connection over the transport.
	connection, err := process.NewConnection(transport, time.Second*6)
	if err != nil {
		return nil, errors.Wrap(err, "unable to create connection to transport process")
	}

	// Redirect the transport's standard error output to a buffer so that we can
	// give better feedback in errors. This might be a bit dangerous since this
	// buffer will be attached for the lifetime of the process and we don't know
	// exactly how much output will be received (and thus we could buffer a
	// large amount of it in memory).
	// TODO: If we do start seeing large allocations in these buffers, a simple
	// size-limited buffer might suffice, at least to get some of the error
	// message.
	// TODO: Since this problem is shared with the agent package, it would be
	// good to implement a shared solution.
	errorBuffer := bytes.NewBuffer(nil)
	transport.Stderr = errorBuffer

	// Start the process.
	if err := transport.Start(); err != nil {
		connection.Close()
		return nil, errors.Wrap(err, "unable to start transport process")
	}

	// Create a client over the transport. We set the root path to be empty
	// since it's the job of the remote to determine what it should be. When
	// checking for errors, we look specifically for transport errors that occur
	// during handshake, because that's an indication that our transport process
	// is not functioning correctly. If that's the case, we wait for the
	// transport process to exit (which we know it will because the
	// NewEndpointClient method will close the connection (hence terminating the
	// process) on failure), and see if we can return a more detailed error
	// message about its failure.
	client, err := remote.NewEndpointClient(connection, "/mnt/app", session, version, configuration, alpha)
	if remote.IsHandshakeTransportError(err) {
		// On error, NewEndpointClient will close the connection, but doing so
		// won't wait for the underlying process to terminate, so we need to do
		// that before trying to access the error buffer.
		transport.Wait()

		// Extract error output and ensure it's UTF-8.
		errorOutput := errorBuffer.String()
		if !utf8.ValidString(errorOutput) {
			return nil, errors.New("transport failed with unknown error")
		}

		// Return an error based on the error output.
		return nil, errors.Errorf(
			"transport process failed with error output:\n%s",
			strings.TrimSpace(errorOutput),
		)
	} else if err != nil {
		return nil, errors.Wrap(err, "unable to create endpoint client")
	}

	// Now that we've successfully connected, disable the kill delay on the
	// process connection.
	connection.SetKillDelay(time.Duration(0))

	// Success.
	return client, nil
}

func init() {
	// Register the custom protocol handler with the session package.
	session.ProtocolHandlers[urlpkg.Protocol_Custom] = &protocolHandler{}
}
