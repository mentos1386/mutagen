package custom

import (
	"fmt"
	"os"

	"github.com/pkg/errors"

	"github.com/havoc-io/mutagen/pkg/prompt"
)

const (
	// AskpassEnvironmentVariable is the environment variable set to the
	// executable responsible for handling prompting requests.
	// TODO: Should we give this a better name?
	AskpassEnvironmentVariable = "MUTAGEN_ASKPASS"
)

// setPrompterVariables sets up environment variables for prompting based on the
// provided prompter identifier.
func setPrompterVariables(environment []string, prompter string) ([]string, error) {
	// Compute the path to the current (mutagen) executable and set it in the
	// MUTAGEN_ASKPASS variable.
	if mutagenPath, err := os.Executable(); err != nil {
		return nil, errors.Wrap(err, "unable to determine executable path")
	} else {
		environment = append(environment,
			fmt.Sprintf("%s=%s", AskpassEnvironmentVariable, mutagenPath),
		)
	}

	// Add the prompter environment variable to make Mutagen recognize a
	// prompting invocation.
	environment = append(environment,
		fmt.Sprintf("%s=%s", prompt.PrompterEnvironmentVariable, prompter),
	)

	// Done.
	return environment, nil
}
