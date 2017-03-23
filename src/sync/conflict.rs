//! Provides the `Conflict` type.

use super::change::Changes;

/// Represents a conflict that occurs during reconciliation.
///
/// At least one change on one side of the conflict will occur at the conflict
/// root.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Conflict {
    /// Conflicting changes from alpha.
    pub alpha_changes: Changes,
    /// Conflicting changes from beta.
    pub beta_changes: Changes,
}

/// A simple type alias for a list of `Conflict` objects.
pub type Conflicts = Vec<Conflict>;
