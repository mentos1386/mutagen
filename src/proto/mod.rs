//! Provides Protocol Buffers message definitions for Mutagen. Ideally these
//! would be put into their related packages, but the Rust Protocol Buffers code
//! generator is a bit nascent and doesn't seem to support the full import
//! infrastructure provided by some of the other language generators. But that's
//! fine, because dealing with import paths is very complicated in general and
//! this is the simple and stupid solution.

pub mod sync;
pub mod time;
pub mod url;
