use std::fmt;

/// Represents a semver-style version with major, minor, and patch components.
pub struct Version {
    /// Represents the major component of the version.
    pub major: u32,
    /// Represents the minor component of the version.
    pub minor: u32,
    /// Represents the patch component of the version.
    pub patch: u32,
}

impl Version {
    /// Returns the current version of Mutagen as specified in the `Cargo.toml`
    /// file. This method will panic if the version cannot be parsed.
    ///
    /// TODO: Should we have this return a `Result` on failure? It's too bad
    /// there's no way to do compile-time parsing and that the Cargo.toml values
    /// aren't typed as integers.
    pub fn get() -> Version {
        Version{
            major: env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap(),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap(),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap(),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
