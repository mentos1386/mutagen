//! Provides the data structures and algorithms underlying the synchronization
//! process.

pub mod change;
mod diff;
pub mod entry;
mod path;
pub mod scan;
#[cfg(test)]
mod tests;
