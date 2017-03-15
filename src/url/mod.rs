//! Provides URL parsing facilities.

#[cfg(test)]
mod tests;

use super::errors::Result;
use super::proto::url::{Protocol, URL};

#[cfg(unix)]
/// No-op on POSIX systems, where local paths won't be mistaken for SSH URLs.
fn is_windows_path(_: &str) -> bool {
    false
}

#[cfg(windows)]
/// Checks (using simple heuristics) if a path that looks like an SSH URL might
/// actually be a local path. This check is only performed on Windows.
fn is_windows_path(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    // HACK: This relies on the string being valid UTF-8 (which it will be by
    // definition) and all of the characters we're dealing with being 1-byte,
    // which fortunately they are. We could use a regular expression here, but
    // that's a bit heavy. In any case, our length check should make this safe.
    bytes.len() >= 3 &&
        ((bytes[0] >= ('a' as u8) && bytes[0] <= ('z' as u8)) ||
            (bytes[0] >= ('A' as u8) && bytes[0] <= ('Z' as u8))) &&
        bytes[1] == (':' as u8) &&
        (bytes[2] == ('\\' as u8) || bytes[2] == ('/' as u8))
}

/// Attempts to parse a raw string into an SSH URL. This function will fail if
/// the URL is not a valid SSH URL.
fn parse_ssh(raw: &str) -> Result<URL> {
    // Track what remains of the original string.
    let mut remaining = raw;

    // Create an empty SSH URL.
    let mut url = URL::new();
    url.protocol = Protocol::SSH;

    // Parse off the username, if any.
    // HACK: This relies on find returning a byte index (which it does) and '@'
    // being a single byte in UTF-8.
    if let Some(index) = remaining.find('@') {
        url.username = remaining[..index].to_owned();
        remaining = &remaining[index+1..];
    }

    // Parse off the hostname.
    // HACK: This relies on find returning a byte index (which it does) and ':'
    // being a single byte in UTF-8.
    if let Some(index) = remaining.find(':') {
        if index == 0 {
            bail!("empty hostname");
        }
        url.hostname = remaining[..index].to_owned();
        remaining = &remaining[index+1..];
    } else {
        // All SSH URLs should contain at least one colon.
        bail!("SSH URL missing hostname or path");
    }

    // Parse off the port. This is not a standard SCP URL syntax (and even Git
    // makes you use full SSH URLs if you want to specify a port), so we invent
    // our own rules here, but essentially we just scan until the next colon,
    // and if there is one and all characters before it are 0-9, we try to parse
    // them as a port. We only accept non-empty strings, because technically a
    // file could start with ':' on some systems.
    // HACK: This relies on find returning a byte index (which it does) and ':'
    // being a single byte in UTF-8.
    if let Some(index) = remaining.find(':') {
        // We could technically use str's parse method here, but it doesn't
        // specify if it supports other radices, so it's best to enforce base 10
        // numbers.
        if let Ok(value) = u16::from_str_radix(&remaining[..index], 10) {
            url.port = value as u32;
            remaining = &remaining[index+1..];
        }
    }

    // Whatever's left is the path.
    url.path = remaining.to_owned();

    // Success.
    Ok(url)
}

/// Attempts to parse a raw URL. This method supports both local and SCP-style
/// SSH URLs.
pub fn parse(raw: &str) -> Result<URL> {
    // We don't allow empty URLs.
    if raw.is_empty() {
        bail!("empty raw URL");
    }

    // Check if this is an SCP-style URL. A URL is classified as such if it
    // contains a colon with no forward slashes before it. On Windows, paths
    // beginning with x:\ or x:/ (where x is a-z or A-Z) are almost certainly
    // referring to local paths, but will trigger the SCP URL detection, so we
    // have to check those first. This is, of course, something of a heuristic,
    // but we're unlikely to encounter 1-character hostnames and very likely to
    // encounter Windows paths (except on POSIX, where this check always returns
    // false). If Windows users do have a 1-character hostname, they should just
    // use some other addressing scheme for it (e.g. an IP address or alternate
    // hostname).
    if !is_windows_path(raw) {
        for c in raw.chars() {
            if c == ':' {
                return parse_ssh(raw);
            } else if c == '/' {
                break;
            }
        }
    }

    // Otherwise, just treat this as a raw path.
    let mut url = URL::new();
    url.protocol = Protocol::Local;
    url.path = raw.to_owned();
    Ok(url)
}
