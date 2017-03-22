//! Provides the `Change` type.

use super::entry::Entry;

/// Represents a change to a filesystem hierarchy.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Change {
    /// The path to the change within the filesystem hierarchy.
    pub path: String,
    /// The old entry, if any.
    pub old: Option<Entry>,
    /// The new entry, if any.
    pub new: Option<Entry>,
}
