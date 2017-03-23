//! Provides tests for the `reconcile` module.

use super::super::entry::Entry::{File, Directory};
use super::super::change::{Change, Changes};
use super::diff;

#[test]
fn diff_trivial() {
    let mut changes = Changes::new();
    diff(&mut changes, "".to_owned(), None, None);
    assert_eq!(changes.len(), 0);
}

#[test]
fn diff_unchanged() {
    let base = File{executable: true, digest: vec![1, 2, 3]};
    let target = File{executable: true, digest: vec![1, 2, 3]};
    let mut changes = Changes::new();
    diff(&mut changes, "".to_owned(), Some(&base), Some(&target));
    assert_eq!(changes.len(), 0);
}

#[test]
fn diff_creation() {
    let created = File{executable: true, digest: vec![]};
    let mut changes = Changes::new();
    diff(&mut changes, "".to_owned(), None, Some(&created));
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "");
    assert_eq!(changes[0].old, None);
    assert_eq!(changes[0].new, Some(created));
}

#[test]
fn diff_creation_root_subpath() {
    let created = File{executable: true, digest: vec![]};
    let mut changes = Changes::new();
    diff(&mut changes, "child path".to_owned(), None, Some(&created));
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "child path");
    assert_eq!(changes[0].old, None);
    assert_eq!(changes[0].new, Some(created));
}

#[test]
fn diff_deletion() {
    let deleted = File{executable: true, digest: vec![]};
    let mut changes = Changes::new();
    diff(&mut changes, "".to_owned(), Some(&deleted), None);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "");
    assert_eq!(changes[0].old, Some(deleted));
    assert_eq!(changes[0].new, None);
}

#[test]
fn diff_change_root_type() {
    let deleted = File{executable: true, digest: vec![]};
    let created = Directory(btreemap!{});
    let mut changes = Changes::new();
    diff(&mut changes, "".to_owned(), Some(&deleted), Some(&created));
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "");
    assert_eq!(changes[0].old, Some(deleted));
    assert_eq!(changes[0].new, Some(created));
}

#[test]
fn diff_delta() {
    // Create some test entries.
    let same_file = File{executable: true, digest: vec![4, 5, 6]};
    let same_directory = Directory(btreemap!{});
    let executability_change_base =
        File{executable: true, digest: vec![1, 2, 3]};
    let executability_change_target =
        File{executable: false, digest: vec![1, 2, 3]};
    let digest_change_base =
        File{executable: true, digest: vec![1, 2, 3]};
    let digest_change_target =
        File{executable: true, digest: vec![4, 5, 6]};
    let executability_and_digest_change_base =
        File{executable: true, digest: vec![1, 2, 3]};
    let executability_and_digest_change_target =
        File{executable: false, digest: vec![4, 5, 6]};
    let deleted_file_base = File{executable: true, digest: vec![1, 2, 3]};
    let deleted_directory_base = Directory(btreemap!{
        "file content".to_owned() =>
            File{executable: false, digest: vec![7, 8, 9]},
        "directory content".to_owned() => Directory(btreemap!{}),
    });
    let created_file_target = File{executable: false, digest: vec![7, 8, 9]};
    let created_directory_target = Directory(btreemap!{
        "created file content".to_owned() =>
            File{executable: true, digest: vec![10, 11, 12]},
        "created directory content".to_owned() => Directory(btreemap!{}),
    });
    let file_to_directory_base = File{executable: false, digest: vec![1, 2, 3]};
    let file_to_directory_target = Directory(btreemap!{});
    let directory_to_file_base = Directory(btreemap!{});
    let directory_to_file_target =
        File{executable: true, digest: vec![1, 2, 3]};

    // Create a base snapshot.
    let base = Directory(btreemap!{
        // Test a file that doesn't change.
        "same file".to_owned() => same_file.clone(),
        // Test a directory that doesn't change.
        "same directory".to_owned() => same_directory.clone(),
        // Test a file that changes in executability.
        "executability change".to_owned() => executability_change_base.clone(),
        // Test a file that changes in digest.
        "digest change".to_owned() => digest_change_base.clone(),
        // Test a file that changes in both executability and digest.
        "executability and digest change".to_owned() =>
            executability_and_digest_change_base.clone(),
        // Test a file that's deleted.
        "deleted file".to_owned() => deleted_file_base.clone(),
        // Test a directory that's deleted.
        "deleted directory".to_owned() => deleted_directory_base.clone(),
        // Test creation of both a file and a directory (the created entries are
        // only present in the target).
        // Test a file that changes to a directory.
        "file to directory".to_owned() => file_to_directory_base.clone(),
        // Test a directory that changes to a file.
        "directory to file".to_owned() => directory_to_file_base.clone(),
        // Test that all the same changes are detected at a sublevel.
        "directory".to_owned() => Directory(btreemap!{
            "same file".to_owned() => same_file.clone(),
            "same directory".to_owned() => same_directory.clone(),
            "executability change".to_owned() =>
                executability_change_base.clone(),
            "digest change".to_owned() => digest_change_base.clone(),
            "executability and digest change".to_owned() =>
                executability_and_digest_change_base.clone(),
            "deleted file".to_owned() => deleted_file_base.clone(),
            "deleted directory".to_owned() => deleted_directory_base.clone(),
            "file to directory".to_owned() => file_to_directory_base.clone(),
            "directory to file".to_owned() => directory_to_file_base.clone(),
        }),
    });

    // Create a target snapshot.
    let target = Directory(btreemap!{
        "same file".to_owned() => same_file.clone(),
        "same directory".to_owned() => same_directory.clone(),
        "executability change".to_owned() =>
            executability_change_target.clone(),
        "digest change".to_owned() => digest_change_target.clone(),
        "executability and digest change".to_owned() =>
            executability_and_digest_change_target.clone(),
        "created file".to_owned() => created_file_target.clone(),
        "created directory".to_owned() => created_directory_target.clone(),
        "file to directory".to_owned() => file_to_directory_target.clone(),
        "directory to file".to_owned() => directory_to_file_target.clone(),
        "directory".to_owned() => Directory(btreemap!{
            "same file".to_owned() => same_file.clone(),
            "same directory".to_owned() => same_directory.clone(),
            "executability change".to_owned() =>
                executability_change_target.clone(),
            "digest change".to_owned() => digest_change_target.clone(),
            "executability and digest change".to_owned() =>
                executability_and_digest_change_target.clone(),
            "created file".to_owned() => created_file_target.clone(),
            "created directory".to_owned() => created_directory_target.clone(),
            "file to directory".to_owned() => file_to_directory_target.clone(),
            "directory to file".to_owned() => directory_to_file_target.clone(),
        }),
    });

    // Create the expected diff.
    let expected = vec![
        Change{
            path: "created directory".to_owned(),
            old: None,
            new: Some(created_directory_target.clone()),
        },
        Change{
            path: "created file".to_owned(),
            old: None,
            new: Some(created_file_target.clone()),
        },
        Change{
            path: "deleted directory".to_owned(),
            old: Some(deleted_directory_base.clone()),
            new: None,
        },
        Change{
            path: "deleted file".to_owned(),
            old: Some(deleted_file_base.clone()),
            new: None,
        },
        Change{
            path: "digest change".to_owned(),
            old: Some(digest_change_base.clone()),
            new: Some(digest_change_target.clone()),
        },
        Change{
            path: "directory/created directory".to_owned(),
            old: None,
            new: Some(created_directory_target.clone()),
        },
        Change{
            path: "directory/created file".to_owned(),
            old: None,
            new: Some(created_file_target.clone()),
        },
        Change{
            path: "directory/deleted directory".to_owned(),
            old: Some(deleted_directory_base.clone()),
            new: None,
        },
        Change{
            path: "directory/deleted file".to_owned(),
            old: Some(deleted_file_base.clone()),
            new: None,
        },
        Change{
            path: "directory/digest change".to_owned(),
            old: Some(digest_change_base.clone()),
            new: Some(digest_change_target.clone()),
        },
        Change{
            path: "directory/directory to file".to_owned(),
            old: Some(directory_to_file_base.clone()),
            new: Some(directory_to_file_target.clone()),
        },
        Change{
            path: "directory/executability and digest change".to_owned(),
            old: Some(executability_and_digest_change_base.clone()),
            new: Some(executability_and_digest_change_target.clone()),
        },
        Change{
            path: "directory/executability change".to_owned(),
            old: Some(executability_change_base.clone()),
            new: Some(executability_change_target.clone()),
        },
        Change{
            path: "directory/file to directory".to_owned(),
            old: Some(file_to_directory_base.clone()),
            new: Some(file_to_directory_target.clone()),
        },
        Change{
            path: "directory to file".to_owned(),
            old: Some(directory_to_file_base.clone()),
            new: Some(directory_to_file_target.clone()),
        },
        Change{
            path: "executability and digest change".to_owned(),
            old: Some(executability_and_digest_change_base.clone()),
            new: Some(executability_and_digest_change_target.clone()),
        },
        Change{
            path: "executability change".to_owned(),
            old: Some(executability_change_base.clone()),
            new: Some(executability_change_target.clone()),
        },
        Change{
            path: "file to directory".to_owned(),
            old: Some(file_to_directory_base.clone()),
            new: Some(file_to_directory_target.clone()),
        },
    ];

    // Compute the actual diff.
    let mut changes = Changes::new();
    diff(&mut changes, "".to_owned(), Some(&base), Some(&target));

    // Verify that the diff is as expected. We do this via iteration (rather
    // than a direct vector comparison) so that assert failures will be easier
    // to read.
    assert_eq!(changes.len(), expected.len());
    for (c, e) in changes.iter().zip(expected.iter()) {
        assert_eq!(*c, *e);
    }
}
