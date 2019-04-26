package url

import (
	"regexp"
	"github.com/pkg/errors"
)

// schemeMatcher is a regular expression that matches a URL scheme prefix.
var schemeMatcher *regexp.Regexp

func init() {
	// Compile our URL scheme matching regular expression. We allow the scheme
	// to be empty and allow non-alphabetical characters for matching on the
	// first character of the scheme, but we enforce on parsing that the scheme
	// length is non-zero and that the first character is an alphabetical
	// character. This allows us to catch protocols that are probably supposed
	// to be custom URL schemes but which have been mistyped. This is a bit of a
	// heuristic, but certainly a reasonable one.
	schemeMatcher = regexp.MustCompile(`^([a-zA-Z0-9+.\-]*)://`)
}

// errNoScheme is a sentinel error for ParseScheme to indicate that no URL
// scheme was detected.
var errNoScheme = errors.New("no scheme detected")

// ParseCustomScheme parses the scheme from a custom URL. It returns an empty string
// if the URL is not a custom-scheme URL, and it returns an error if the string
// is not a properly formatted custom-scheme URL.
func ParseCustomScheme(raw string) (string, error) {
	// Compute up to a single match.
	matches := schemeMatcher.FindAllStringSubmatch(raw, 1)

	// If there weren't any matches, then we're done.
	if len(matches) == 0 {
		return "", errNoScheme
	}

	// Extract the proprosed scheme.
	scheme := matches[0][1]

	// Ensure that scheme is non-empty.
	if scheme == "" {
		return "", errors.New("empty scheme")
	}

	// Ensure that it starts with an alphabetical character.
	inLower := 'a' <= scheme[0] && scheme[0] <= 'z'
	inUpper := 'A' <= scheme[0] && scheme[0] <= 'Z'
	if !(inLower || inUpper) {
		return "", errors.New("custom scheme must start with alphabetical character")
	}

	// Success.
	return scheme, nil
}


func isCustomURL(raw string) bool {
	scheme, err := ParseCustomScheme(raw)
	return err == nil && scheme != ""
}

func parseCustom(raw string) (*URL, error) {
	return &URL{
		Protocol: Protocol_Custom,
		Path:     raw,
	}, nil
}
