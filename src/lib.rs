//! Provides version and legal information for Mutagen.

#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate blake2_rfc;
extern crate protobuf;
extern crate sha1;

#[cfg(target_os = "macos")]
extern crate libc;
#[cfg(target_os = "macos")]
extern crate unicode_normalization;

pub mod errors {
    //! Provides error handling infrastructure for Mutagen via the error-chain
    //! crate.
    error_chain! { }
}
pub mod hash;
pub mod sync;
pub mod prompt;
pub mod proto;
pub mod time;
pub mod url;
#[cfg(test)]
mod tests;

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

/// Contains all legal information for Mutagen and its dependencies.
// TODO: This needs to be updated with *all* transitive dependencies. There are
// a few tools that are supposed to be able to show these.
pub const LEGAL_NOTICE: &'static str = "Mutagen (https://mutagen.io)

Copyright (c) 2016 - 2017 Jacob Howard. All rights reserved.

Released under the terms of the MIT License.

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the \"Software\"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


Mutagen makes use of the following third-party software:


clap (https://github.com/kbknapp/clap-rs)

The MIT License (MIT)

Copyright (c) 2015-2016 Kevin B. Knapp

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the \"Software\"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.";
