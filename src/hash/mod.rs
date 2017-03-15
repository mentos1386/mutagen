//! Provides a uniform interface to a variety of hash functions.

#[cfg(test)]
mod tests;

use std::fmt;
use std::io;
use sha1::Sha1 as Sha1Impl;
use blake2_rfc::blake2b::Blake2b as Blake2bImpl;

/// Enumerates the supported hashing algorithms.
pub enum Algorithm {
    /// Represents the SHA1 algorithm.
    Sha1,
    /// Represents the BLAKE2b algorithm with variable length (in bytes).
    Blake2b(usize)
}

/// Represents the digest of a hash algorithm. The raw underlying vector can be
/// easily extracted if necessary.
pub struct Digest(pub Vec<u8>);

impl fmt::Display for Digest {
    /// Formats the digest into a lower-case hexidecimal string with no prefix,
    /// spaces, or other punctuation. All bytes are rendered as two character
    /// string components, with 0-padding as necessary.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &b in self.0.iter() {
            try!(write!(f, "{:02x}", b));
        }
        Ok(())
    }
}

/// Encapsulates the underlying hash algorithm state.
enum HasherState {
    /// Encapsulates a SHA1 state.
    Sha1(Sha1Impl),
    /// Encapsulates a BLAKE2b state.
    Blake2b(Blake2bImpl),
}

pub struct Hasher {
    /// The underlying hash algorithm state.
    state: HasherState,
}

impl Hasher {
    /// Constructs a new hasher.
    pub fn new(algorithm: &Algorithm) -> Hasher {
        Hasher{
            state: match *algorithm {
                Algorithm::Sha1 => HasherState::Sha1(Sha1Impl::new()),
                Algorithm::Blake2b(length) =>
                    HasherState::Blake2b(Blake2bImpl::new(length)),
            },
        }
    }

    /// Consumes the hasher and computes the resulting digest.
    pub fn digest(self) -> Digest {
        match self.state {
            HasherState::Sha1(state) => Digest(state.digest().bytes().to_vec()),
            HasherState::Blake2b(state) =>
                Digest(state.finalize().as_bytes().to_vec()),
        }
    }
}

impl io::Write for Hasher {
    /// Feeds data into the hasher's state.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.state {
            HasherState::Sha1(ref mut state) => state.update(buf),
            HasherState::Blake2b(ref mut state) => state.update(buf),
        }
        Ok(buf.len())
    }

    /// No-op that always succeeds.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
