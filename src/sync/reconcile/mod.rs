//! Provides reconciliation functionality for filesystem hierarchies.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use ordered_iter::OrderedMapIterator;
use super::change::{Change, Changes};
use super::conflict::{Conflict, Conflicts};
use super::entry::Entry::{self, Directory};
use super::path::join;

// TODO: Can we lazily construct path values? E.g., could we use something like
// a Vec<&str> (or some stack type) that only joins and allocates if needed?

fn diff<'a>(changes: &mut Changes,
            path: String,
            base: Option<&'a Entry>,
            target: Option<&'a Entry>) {
    // First, check if both base and target are directories. If so, then there's
    // no change at this node, we merely need to recurse. Otherwise, if there's
    // a non-None-to-None transition, record a change if the nodes differ. We
    // know that this comparison will be cheap (non-recursive) because at least
    // one of base or target is not a directory (otherwise the first conditional
    // would have been triggered).
    if let (Some(&Directory(ref bases)),
            Some(&Directory(ref targets))) = (base, target) {
        for (name, (b, t)) in bases.iter().outer_join(targets.iter()) {
            diff(changes, join(&path, name), b, t);
        }
    } else if (base != None || target != None) && base != target {
        changes.push(Change{
            path: path,
            old: base.map(|e| e.clone()),
            new: target.map(|e| e.clone()),
        });
    }
}

fn reconcile_recursive<'a>(
    ancestor_changes: &mut Changes,
    alpha_changes: &mut Changes,
    beta_changes: &mut Changes,
    conflicts: &mut Conflicts,
    path: String,
    ancestor: Option<&'a Entry>,
    alpha: Option<&'a Entry>,
    beta: Option<&'a Entry>
) {
    // Check if both alpha and beta are directories. If they are, then we're
    // going to recurse. Before we do that though, we need to determine if the
    // ancestor is also a directory, and if not we need to record a shallow
    // change for this node (this catches both-modified-same behavior). The
    // nature of ancestor will also affect whether we do a 2-way outer-join or a
    // 3-way outer-join.
    if let (Some(&Directory(ref alphas)),
            Some(&Directory(ref betas))) = (alpha, beta) {
        if let Some(&Directory(ref ancestors)) = ancestor {
            for (name, (oa, b)) in ancestors.iter()
                                    .outer_join(alphas.iter())
                                    .outer_join(betas.iter()) {
                let (o, a) = oa.unwrap_or((None, None));
                reconcile_recursive(
                    ancestor_changes, alpha_changes, beta_changes, conflicts,
                    join(&path, name), o, a, b
                );
            }
        } else {
            ancestor_changes.push(Change{
                path: path.clone(),
                old: ancestor.map(|e| e.clone()),
                new: Some(Directory(BTreeMap::new())),
            });
            for (name, (a, b)) in alphas.iter().outer_join(betas.iter()) {
                reconcile_recursive(
                    ancestor_changes, alpha_changes, beta_changes, conflicts,
                    join(&path, name), None, a, b
                );
            }
        }
        return;
    }

    // Check if alpha and beta are equal. We know that this comparison will be
    // cheap (non-recursive) because at least one of alpha or beta is not a
    // directory (otherwise the above conditional would have been triggered).
    // If they are equal, then we may need to record an ancestor change (in
    // order to catch both-modified-same behavior), but we're done either way.
    // We also know that the ancestor/alpha comparison will be cheap
    // (non-recursive) because neither alpha nor beta can be a directory at that
    // point (at least one of them can't be, and since they are equal, they both
    // must not be.)
    if alpha == beta {
        if ancestor != alpha {
            ancestor_changes.push(Change{
                path: path,
                old: ancestor.map(|e| e.clone()),
                new: alpha.map(|e| e.clone()),
            });
        }
        return;
    }

    // Alpha and beta weren't equal at this node. Thus, at least one of them
    // must differ from ancestor *at this node* (due to the transitivity of
    // equality, we can't have ancestor == alpha and ancestor == beta but not
    // alpha == beta). The other may also differ from the ancestor at this node,
    // a subnode, or not at all. We compute the delta between ancestor and each
    // side. If one of the sides has no changes, then we can simply propagate
    // changes from the other side.
    let mut alpha_delta = Changes::new();
    diff(&mut alpha_delta, path.clone(), ancestor, alpha);
    if alpha_delta.len() == 0 {
        alpha_changes.push(Change{
            path: path,
            old: alpha.map(|e| e.clone()),
            new: beta.map(|e| e.clone()),
        });
        return;
    }
    let mut beta_delta = Changes::new();
    diff(&mut beta_delta, path.clone(), ancestor, beta);
    if beta_delta.len() == 0 {
        beta_changes.push(Change{
            path: path,
            old: beta.map(|e| e.clone()),
            new: alpha.map(|e| e.clone()),
        });
        return;
    }

    // It appears that both sides have been modified. Before we mark a conflict,
    // check if one side has only deletion changes. If so, we can propagate the
    // changes from the other side without fear of losing any information. This
    // is essentially the only form of automated conflict resolution that we can
    // do. In some sense, it is a heuristic designed to avoid conflicts in very
    // common cases, but more importantly, it is necessary to enable our form of
    // manual conflict resolution: having the user delete the side they don't
    // want to keep.
    //
    // Now, you're probably asking yourself a few questions here:
    //
    // Why didn't we simply make this check first? Why do we need to check the
    // full diffs above? Well, imagine that one side had deleted and the other
    // was unmodified. If we only looked at non-deletion changes, we would not
    // detect this because both sides would have no changes or deletion-only
    // changes, and both lists below would be empty, and the winning side would
    // be determined simply by the ordering of the conditional statement below
    // (essentially beta would always win out as it is currently structured).
    //
    // What if both sides have completely deleted this node? Well, that would
    // have passed the equality check above and would have been treated as a
    // both-deleted scenario. Thus, we know at least one side has a non-deletion
    // change.
    //
    // What if both sides are directories and have only deleted some subset of
    // the tree below here? Well, then we would have simply recursed - there
    // would be no detectable changes at this node.
    //
    // Note that, when recording these changes, we use the side we're going to
    // overrule as the "old" value in the change, because that's what it should
    // expect to see on disk, not the ancestor. And since that "old" must be a
    // subtree of ancestor (it contains only deletion changes), it still
    // represents a valid value to return from a transition in the case that the
    // transition fails, and, as a nice side-effect in that case, no information
    // about the deletions that have happened on that side is lost.
    let alpha_delta_non_deletion = alpha_delta.iter()
                                    .filter(|c| c.new != None)
                                    .cloned()
                                    .collect::<Changes>();
    if alpha_delta_non_deletion.len() == 0 {
        alpha_changes.push(Change{
            path: path,
            old: alpha.map(|e| e.clone()),
            new: beta.map(|e| e.clone()),
        });
        return;
    }
    let beta_delta_non_deletion = beta_delta.iter()
                                    .filter(|c| c.new != None)
                                    .cloned()
                                    .collect::<Changes>();
    if beta_delta_non_deletion.len() == 0 {
        beta_changes.push(Change{
            path: path,
            old: beta.map(|e| e.clone()),
            new: alpha.map(|e| e.clone()),
        });
        return;
    }

    // At this point, both sides have made changes that would cause information
    // to be lost if we were to propgate changes from one side to the other, so
    // we need to record a conflict. We only record non-deletion changes because
    // those are the only ones that create conflict.
    conflicts.push(Conflict{
        alpha_changes: alpha_delta_non_deletion,
        beta_changes: beta_delta_non_deletion,
    });
}

pub fn reconcile(
    ancestor: &Option<Entry>,
    alpha: &Option<Entry>,
    beta: &Option<Entry>
) -> (Changes, Changes, Changes, Conflicts) {
    let mut ancestor_changes = Changes::new();
    let mut alpha_changes = Changes::new();
    let mut beta_changes = Changes::new();
    let mut conflicts = Conflicts::new();
    reconcile_recursive(
        &mut ancestor_changes,
        &mut alpha_changes,
        &mut beta_changes,
        &mut conflicts,
        "".to_owned(),
        ancestor.as_ref(),
        alpha.as_ref(),
        beta.as_ref(),
    );
    (ancestor_changes, alpha_changes, beta_changes, conflicts)
}
