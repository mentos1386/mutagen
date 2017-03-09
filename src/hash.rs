use std::fmt;
use std::io;
use sha1::Sha1 as Sha1Impl;
use blake2_rfc::blake2b::Blake2b as Blake2bImpl;

pub enum Algorithm {
    Sha1,
    Blake2b(usize)
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Algorithm::Sha1 => write!(f, "SHA1"),
            Algorithm::Blake2b(s) => write!(f, "BLAKE2b ({})", s),
        }
    }
}

impl Algorithm {
    pub fn hasher(&self) -> Box<Hasher> {
        match *self {
            Algorithm::Sha1 => Box::new(Sha1::new()),
            Algorithm::Blake2b(s) => Box::new(Blake2b::new(s)),
        }
    }
}

pub trait Hasher : io::Write {
    fn digest(self: Box<Self>) -> Vec<u8>;
}

struct Sha1 {
    state: Sha1Impl,
}

impl Sha1 {
    fn new() -> Sha1 {
        Sha1 {
            state: Sha1Impl::new(),
        }
    }
}

impl io::Write for Sha1 {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.state.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Hasher for Sha1 {
    fn digest(self: Box<Self>) -> Vec<u8> {
        self.state.digest().bytes().to_vec()
    }
}

struct Blake2b {
    state: Blake2bImpl,
}

impl Blake2b {
    fn new(size: usize) -> Blake2b {
        Blake2b {
            state: Blake2bImpl::new(size),
        }
    }
}

impl io::Write for Blake2b {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.state.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Hasher for Blake2b {
    fn digest(self: Box<Self>) -> Vec<u8> {
        self.state.finalize().as_bytes().to_vec()
    }
}
