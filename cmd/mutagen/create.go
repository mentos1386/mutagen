package main

import (
	"fmt"
	"strconv"

	"github.com/pkg/errors"

	"github.com/havoc-io/mutagen/cmd"
	"github.com/havoc-io/mutagen/daemon"
	"github.com/havoc-io/mutagen/filesystem"
	"github.com/havoc-io/mutagen/rpc"
	sessionpkg "github.com/havoc-io/mutagen/session"
	"github.com/havoc-io/mutagen/url"
)

var createUsage = `usage: mutagen create [-h|--help] [-i|--ignore=<pattern>]
                      [--ignore-size=<size>] <alpha> <beta>

Creates and starts a new synchronization session.
`

type ignorePatterns []string

func (p *ignorePatterns) String() string {
	return "ignore patterns"
}

func (p *ignorePatterns) Set(value string) error {
	*p = append(*p, value)
	return nil
}

func createMain(arguments []string) error {
	// Parse command line arguments.
	var ignores ignorePatterns
	var ignoreSizeString string
	var ignoreSize uint64
	flagSet := cmd.NewFlagSet("create", createUsage, []int{2})
	flagSet.VarP(&ignores, "ignore", "i", "specify ignore paths")
	flagSet.StringVar(&ignoreSizeString, "ignore-size", "", "ignore files above a certain size")
	urls := flagSet.ParseOrDie(arguments)
	if length := len(ignoreSizeString); length > 0 {
		multiplier := uint64(1)
		lastByte := ignoreSizeString[length-1]
		if lastByte == 'K' {
			multiplier = 1024
			ignoreSizeString = ignoreSizeString[:length-1]
		} else if lastByte == 'M' {
			multiplier = 1024 * 1024
			ignoreSizeString = ignoreSizeString[:length-1]
		} else if lastByte == 'G' {
			multiplier = 1024 * 1024 * 1024
			ignoreSizeString = ignoreSizeString[:length-1]
		}
		if parsed, err := strconv.ParseUint(ignoreSizeString, 10, 64); err != nil {
			return errors.Wrap(err, "unable to parse ignore size")
		} else {
			ignoreSize = parsed * multiplier
		}
	}

	// Extract and parse URLs.
	alpha, err := url.Parse(urls[0])
	if err != nil {
		return errors.Wrap(err, "unable to parse alpha URL")
	}
	beta, err := url.Parse(urls[1])
	if err != nil {
		return errors.Wrap(err, "unable to parse beta URL")
	}

	// If either URL is a local path, make sure it's normalized.
	if alpha.Protocol == url.Protocol_Local {
		if alphaPath, err := filesystem.Normalize(alpha.Path); err != nil {
			return errors.Wrap(err, "unable to normalize alpha path")
		} else {
			alpha.Path = alphaPath
		}
	}
	if beta.Protocol == url.Protocol_Local {
		if betaPath, err := filesystem.Normalize(beta.Path); err != nil {
			return errors.Wrap(err, "unable to normalize beta path")
		} else {
			beta.Path = betaPath
		}
	}

	// Create a daemon client.
	daemonClient := rpc.NewClient(daemon.NewOpener())

	// Invoke the session creation method and ensure the resulting stream is
	// closed when we're done.
	stream, err := daemonClient.Invoke(sessionpkg.MethodCreate)
	if err != nil {
		return errors.Wrap(err, "unable to invoke session creation")
	}
	defer stream.Close()

	// Send the initial request.
	request := sessionpkg.CreateRequest{
		Alpha: alpha,
		Beta:  beta,
		Ignores: sessionpkg.IgnoreSpecification{
			Paths: []string(ignores),
			Size:  ignoreSize,
		},
	}
	if err := stream.Send(request); err != nil {
		return errors.Wrap(err, "unable to send creation request")
	}

	// Handle authentication challenges.
	if err := handlePromptRequests(stream); err != nil {
		return errors.Wrap(err, "unable to handle prompt requests")
	}

	// Receive the create response.
	var response sessionpkg.CreateResponse
	if err := stream.Receive(&response); err != nil {
		return errors.Wrap(err, "unable to receive create response")
	}

	// Print the session identifier.
	fmt.Println("Created session", response.Session)

	// Success.
	return nil
}
