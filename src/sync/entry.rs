//! Provides the `Entry` type and implements its serialization methods.

use std::collections::BTreeMap;

// TODO: Can we enforce that keys are non-empty for directory maps? Maybe with a
// custom type. We'd also want to enforce that they don't contain invalid
// characters though. I think trying to enforce these constraints is a losing
// battle - the main win of moving to Rust is that we don't have mutations.
/// Represents an entry in a filesystem hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entry {
    /// A directory entry.
    Directory(BTreeMap<String, Entry>),
    /// A file entry.
    File{executable: bool, digest: Vec<u8>},
}
