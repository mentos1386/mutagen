package url

import (
	"github.com/pkg/errors"
)

// EnsureValid ensures that URL's invariants are respected.
func (u *URL) EnsureValid() error {
	// Ensure that the URL is non-nil.
	if u == nil {
		return errors.New("nil URL")
	}

	// Handle validation based on protocol.
	if u.Protocol == Protocol_Local {
		if u.Username != "" {
			return errors.New("local URL with non-empty username")
		} else if u.Hostname != "" {
			return errors.New("local URL with non-empty hostname")
		} else if u.Port != 0 {
			return errors.New("local URL with non-zero port")
		} else if u.Path == "" {
			return errors.New("local URL with empty path")
		}
	} else if u.Protocol == Protocol_SSH {
		if u.Hostname == "" {
			return errors.New("SSH URL with empty hostname")
		} else if u.Path == "" {
			return errors.New("SSH URL with empty path")
		}
	} else if u.Protocol == Protocol_Custom {
		if u.Username != "" {
			return errors.New("custom URL with non-empty username")
		} else if u.Hostname != "" {
			return errors.New("custom URL with non-empty hostname")
		} else if u.Port != 0 {
			return errors.New("custom URL with non-zero port")
		} else if u.Path == "" {
			return errors.New("custom URL with empty raw URL")
		} else if scheme, err := ParseScheme(u.Path); err != nil {
			return errors.Wrap(err, "unable to parse custom URL scheme")
		} else if scheme == "" {
			return errors.New("custom URL with invalid scheme")
		}
	} else {
		return errors.New("unknown or unsupported protocol")
	}

	// Success.
	return nil
}
