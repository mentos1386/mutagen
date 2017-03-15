//! Provides platform-specific functionality for the sync module.

#[cfg(target_os = "macos")]
pub mod macos;
