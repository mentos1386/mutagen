package url

import (
	"fmt"
)

// Format formats a URL into a human-readable (and reparsable) format.
func (u *URL) Format(environmentPrefix string) string {
	if u.Protocol == Protocol_Local {
		return u.formatLocal()
	} else if u.Protocol == Protocol_SSH {
		return u.formatSSH()
	} else if u.Protocol == Protocol_Custom {
		return u.formatCustom()
	} else if u.Protocol == Protocol_Docker {
		return u.formatDocker(environmentPrefix)
	}
	panic("unknown URL protocol")
}

// formatLocal formats a local URL.
func (u *URL) formatLocal() string {
	return u.Path
}

// formatSSH formats an SSH URL into an SCP-style URL.
func (u *URL) formatSSH() string {
	// Create the base result.
	result := u.Hostname

	// Add username if present.
	if u.Username != "" {
		result = fmt.Sprintf("%s@%s", u.Username, result)
	}

	// Add port if present.
	if u.Port != 0 {
		result = fmt.Sprintf("%s:%d", result, u.Port)
	}

	// Add path.
	result = fmt.Sprintf("%s:%s", result, u.Path)

	// Done.
	return result
}

// formatCustom formats a custom URL. We treat custom URLs as opaque and simply
// return their value.
func (u *URL) formatCustom() string {
	return u.Path
}

// invalidDockerURLFormat is the value returned by formatDocker when a URL is
// provided that breaks invariants.
const invalidDockerURLFormat = "<invalid-docker-url>"

// formatDocker formats a Docker URL.
func (u *URL) formatDocker(environmentPrefix string) string {
	// Start with the container name.
	result := u.Hostname

	// Append the path. If this is a home-directory-relative path or a Windows
	// path, then we need to prepend a slash.
	if u.Path == "" {
		return invalidDockerURLFormat
	} else if u.Path[0] == '/' {
		result += u.Path
	} else if u.Path[0] == '~' || isWindowsPath(u.Path) {
		result += fmt.Sprintf("/%s", u.Path)
	} else {
		return invalidDockerURLFormat
	}

	// Add username if present.
	if u.Username != "" {
		result = fmt.Sprintf("%s@%s", u.Username, result)
	}

	// Add the scheme.
	result = dockerURLPrefix + result

	// Add environment variable information if requested.
	if environmentPrefix != "" {
		for _, variable := range DockerEnvironmentVariables {
			result += fmt.Sprintf("%s%s=%s", environmentPrefix, variable, u.Environment[variable])
		}
	}

	// Done.
	return result
}
