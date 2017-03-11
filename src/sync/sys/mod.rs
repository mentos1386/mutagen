//! Provides platform-specific functionality for the sync module.

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use self::macos::{decomposes_unicode, recompose_unicode_name};
