//! Provides a uniform interface to a variety of hash functions.

#[cfg(test)]
mod tests;

use std::fmt;
use std::io;
use sha1::Sha1 as Sha1Impl;
use blake2_rfc::blake2b::Blake2b as Blake2bImpl;

/// Represents a hashing algorithm.
pub enum Algorithm {
    /// Represents the SHA1 algorithm.
    Sha1,
    /// Represents the BLAKE2b algorithm with variable length (in bytes).
    Blake2b(usize)
}

impl Algorithm {
    /// Constructs a new hasher for the algorithm.
    pub fn hasher(&self) -> Box<Hasher> {
        match *self {
            Algorithm::Sha1 => Box::new(Sha1::new()),
            Algorithm::Blake2b(s) => Box::new(Blake2b::new(s)),
        }
    }
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

/// Provides the common interface for hashers. Data can be fed into the hasher's
/// state using the `Write` interface and the final hash computed using the
/// `digest` method. The `write` method will always be successful for hashers.
pub trait Hasher : io::Write {
    /// Consumes the hasher and computes the resulting digest.
    fn digest(self: Box<Self>) -> Digest;
}

/// Implements the `Hasher` trait for the SHA1 algorithm.
struct Sha1 {
    /// The underlying SHA1 hasher state.
    state: Sha1Impl,
}

impl Sha1 {
    /// Constructs a new SHA1 hasher.
    fn new() -> Sha1 {
        Sha1 {
            state: Sha1Impl::new(),
        }
    }
}

impl io::Write for Sha1 {
    /// Feeds data into the hasher's state.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.state.update(buf);
        Ok(buf.len())
    }

    /// No-op that always succeeds.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Hasher for Sha1 {
    /// Computes the final SHA1 digest. It will have a length of 20 bytes.
    fn digest(self: Box<Self>) -> Digest {
        Digest(self.state.digest().bytes().to_vec())
    }
}

/// Implements the `Hasher` trait for the BLAKE2b algorithm.
struct Blake2b {
    state: Blake2bImpl,
}

impl Blake2b {
    /// Constructs a new BLAKE2b hasher with the specified byte width.
    fn new(size: usize) -> Blake2b {
        Blake2b {
            state: Blake2bImpl::new(size),
        }
    }
}

impl io::Write for Blake2b {
    /// Feeds data into the hasher's state.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.state.update(buf);
        Ok(buf.len())
    }

    /// No-op that always succeeds.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Hasher for Blake2b {
    /// Computes the final BLAKE2b digest. It will have a length equal to the
    /// specified width of the hasher.
    fn digest(self: Box<Self>) -> Digest {
        Digest(self.state.finalize().as_bytes().to_vec())
    }
}
