package main

import (
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"
	"time"

	"github.com/pkg/errors"

	shell "github.com/mattn/go-shellwords"

	"github.com/havoc-io/mutagen/cmd"
	"github.com/havoc-io/mutagen/daemon"
	"github.com/havoc-io/mutagen/environment"
	"github.com/havoc-io/mutagen/filesystem"
	"github.com/havoc-io/mutagen/rpc"
	sessionpkg "github.com/havoc-io/mutagen/session"
	"github.com/havoc-io/mutagen/url"
)

const (
	// The environment variable from which to grab the user's editor command.
	editorEnvironmentVariable = "EDITOR"

	// editsDirectoryName is the Mutagen subdirectory in which to store edits.
	editsDirectoryName = "edits"

	// editLeafName is the leaf name to use inside the unique edit directory.
	editLeafName = "mutagen_edit"

	// initializationPollInterval is the interval that we poll at until we
	// see the synchronized file appear locally.
	initializationPollInterval = 500 * time.Millisecond
)

var editUsage = `usage: mutagen edit [-h|--help] [-i|--ignore=<pattern>]
                    <target>

Creates and starts a new ephemeral synchronization session between the target
and a local temporary path, starts the editor specified in the EDITOR
environment variable and provides it the temporary path, terminates the session
when the editor exits, and finally cleans up local temporary resources. If a
local path is specified, the editor is simply launched on that path directly.
`

func launchEditorAndWait(command string, arguments []string) error {
	process := exec.Command(command, arguments...)
	process.Stdin = os.Stdin
	process.Stdout = os.Stdout
	process.Stderr = os.Stderr
	return process.Run()
}

func editMain(arguments []string) error {
	// Parse command line arguments.
	var ignores ignorePatterns
	flagSet := cmd.NewFlagSet("create", createUsage, []int{1})
	flagSet.VarP(&ignores, "ignore", "i", "specify ignore paths")
	urls := flagSet.ParseOrDie(arguments)

	// Extract and parse target URL.
	target, err := url.Parse(urls[0])
	if err != nil {
		return errors.Wrap(err, "unable to parse target URL")
	}

	// Grab the editor specification.
	editor := environment.Current[editorEnvironmentVariable]
	if editor == "" {
		return errors.New("editor environment variable not set")
	}

	// Parse the editor command.
	var editorArguments []string
	if components, err := shell.Parse(editor); err != nil {
		return errors.Wrap(err, "unable to parse editor environment variable")
	} else if len(components) == 0 {
		return errors.New("invalid editor command")
	} else {
		editor = components[0]
		editorArguments = components[1:]
	}

	// If the target is a local path, then just execute the editor on it
	// directly and bail.
	if target.Protocol == url.Protocol_Local {
		editorArguments = append(editorArguments, target.Path)
		return errors.Wrap(
			launchEditorAndWait(editor, editorArguments),
			"error executing editor on local path",
		)
	}

	// Compute the path to the edits directory and ensure it exists.
	editsDirectory, err := filesystem.Mutagen(true, editsDirectoryName)
	if err != nil {
		return errors.Wrap(err, "unable to compute/create edits directory")
	}

	// Create a temporary edit directory.
	temporaryParent, err := ioutil.TempDir(editsDirectory, "edit")
	if err != nil {
		return errors.Wrap(err, "unable to create temporary edit directory")
	}

	// Compute the local target. It will already be normalized, so we don't need
	// to worry about that.
	local, err := url.Parse(filepath.Join(temporaryParent, editLeafName))
	if err != nil {
		os.RemoveAll(temporaryParent)
		return errors.Wrap(err, "unable to compute local synchronization URL")
	}

	// Create a daemon client.
	daemonClient := rpc.NewClient(daemon.NewOpener())

	// Invoke the session creation method.
	stream, err := daemonClient.Invoke(sessionpkg.MethodCreate)
	if err != nil {
		os.RemoveAll(temporaryParent)
		return errors.Wrap(err, "unable to invoke session creation")
	}

	// At this point, we only remove the temporary directory when we're sure the
	// session has been terminated and isn't running. At most points in this
	// logic, it's possible for the session to be successfully created and
	// start. If that happens, removing the temporary directory would wipe the
	// remote. It's best to just let manual cleanup happen at that point.

	// Send the initial request.
	createRequest := sessionpkg.CreateRequest{
		Alpha:   target,
		Beta:    local,
		Ignores: []string(ignores),
	}
	if err := stream.Send(createRequest); err != nil {
		stream.Close()
		return errors.Wrap(err, "unable to send creation request")
	}

	// Handle authentication challenges.
	if err := handlePromptRequests(stream); err != nil {
		stream.Close()
		return errors.Wrap(err, "unable to handle prompt requests")
	}

	// Receive and validate the create response.
	var createResponse sessionpkg.CreateResponse
	if err := stream.Receive(&createResponse); err != nil {
		stream.Close()
		return errors.Wrap(err, "unable to receive create response")
	} else if createResponse.Session == "" {
		stream.Close()
		return errors.New("empty session identifier returned")
	}

	// Close the create stream.
	if err := stream.Close(); err != nil {
		return errors.Wrap(err, "unable to close create stream")
	}

	// TODO: We need to wait for a full initial synchronizatoin to occur. We'll
	// need to extend session state with this information and poll for it.

	// Run the editor and wait for it to complete.
	editorArguments = append(editorArguments, local.Path)
	if err := launchEditorAndWait(editor, editorArguments); err != nil {
		cmd.Warning("editor exited abnormally, avoiding session termination")
		return errors.Wrap(err, "abnormal editor termination")
	}

	// TODO: We need a wait for force a final synchronization and verify that it
	// completes in case the user saves the file and exits the editor right
	// after.

	// Invoke the session terminate method and ensure the resulting stream is
	// closed when we're done.
	stream, err = daemonClient.Invoke(sessionpkg.MethodTerminate)
	if err != nil {
		return errors.Wrap(err, "unable to invoke session terminate")
	}
	defer stream.Close()

	// Send the terminate request.
	terminateRequest := sessionpkg.TerminateRequest{
		Session: createResponse.Session,
	}
	if err := stream.Send(terminateRequest); err != nil {
		return errors.Wrap(err, "unable to send terminate request")
	}

	// Receive the terminate response.
	var terminateResponse sessionpkg.TerminateResponse
	if err := stream.Receive(&terminateResponse); err != nil {
		return errors.Wrap(err, "unable to receive terminate response")
	}

	// Clean up the temporary directory.
	if err := os.RemoveAll(temporaryParent); err != nil {
		return errors.Wrap(err, "unable to clean up temporary edit directory")
	}

	// Success.
	return nil
}
