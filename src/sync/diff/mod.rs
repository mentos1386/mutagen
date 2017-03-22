//! Provides diff functionality for filesystem hierarchies.

#[cfg(test)]
mod tests;

use ordered_iter::OrderedMapIterator;
use super::change::Change;
use super::entry::Entry::{self, File, Directory};
use super::path::join;

/// Implements the recursive diff functionality needed to support `diff`.
fn diff_recursive(changes: &mut Vec<Change>,
                  path: String,
                  base: &Entry,
                  target: &Entry) {
    // Switch based on the type of base and target.
    match (base, target) {
        // Handle the case where both entries are directories.
        (&Directory(ref bs), &Directory(ref ts)) => {
            // Perform an outer join over the two content maps.
            for (n, (ob, ot)) in bs.iter().outer_join(ts.iter()) {
                // Compute the path for the content.
                let entry_path = join(&path, n);

                // Handle based on the presence of an entry in each map.
                // NOTE: There is an open issue on the ordered_iter crate about
                // returning a custom struct from outer_join:
                //  https://github.com/contain-rs/ordered_iter/issues/5
                // If this happens, we'll be able to remove the panic below,
                // which should never happen in practice since at least one of
                // the maps will have yielded a non-None entry.
                match (ob, ot) {
                    // If the entry is present in both content maps, then we
                    // need to recurse.
                    (Some(b), Some(t)) => {
                        diff_recursive(changes, entry_path, b, t);
                    },
                    // If the entry is only present in target, it's a creation.
                    (None, Some(t)) => {
                        changes.push(Change{
                            path: entry_path,
                            old: None,
                            new: Some(t.clone())
                        });
                    },
                    // If the entry is only present in target, it's a deletion.
                    (Some(b), None) => {
                        changes.push(Change{
                            path: entry_path,
                            old: Some(b.clone()),
                            new: None
                        });
                    },
                    // This case cannot happen - see NOTE above.
                    (None, None) => {
                        panic!("invalid join iteration");
                    }
                }
            }
        },
        // Handle the case where both entries are files.
        (&File{executable: ref be, digest: ref bd},
         &File{executable: ref te, digest: ref td}) => {
            if be != te || bd != td {
                changes.push(Change{
                    path: path,
                    old: Some(base.clone()),
                    new: Some(target.clone()),
                })
            }
        },
        // Handle the case where the entries differ in type. This case is
        // trivial - we simply throw out the old value and replace it with the
        // new one.
        _ => {
            changes.push(Change{
                path: path,
                old: Some(base.clone()),
                new: Some(target.clone())
            })
        }
    }
}

/// Performs a recursive diff between two (potentially non-existent) `Entry`
/// objects.
pub fn diff(path: String,
            base: &Option<Entry>,
            target: &Option<Entry>) -> Vec<Change> {
    // Switch based on the presence of base and target.
    match (base, target) {
        // If both are present, then we need to process recursively.
        (&Some(ref b), &Some(ref t)) => {
            let mut changes = vec![];
            diff_recursive(&mut changes, path, b, t);
            changes
        },
        // If the only the target entry is present, it's a creation.
        (&None, &Some(ref t)) => {
            vec![Change{
                path: path,
                old: None,
                new: Some(t.clone())
            }]
        },
        // If the only the base entry is present, it's a creation.
        (&Some(ref b), &None) => {
            vec![Change{
                path: path,
                old: Some(b.clone()),
                new: None
            }]
        },
        // If neither entry is present, the diff is trivial.
        (&None, &None) => {
            vec![]
        }
    }
}
