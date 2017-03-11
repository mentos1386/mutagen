//! Provides tests for the macos module.

use std::env::temp_dir;
use std::fs::{File, remove_file};
use super::{
    decomposes_unicode as decomposes,
    recompose_unicode_name as recompose
};

/// The name "mütágèn" in NFD normalization.
const MUTAGEN_DECOMPOSED: &'static str =
    "\u{6d}\u{75}\u{308}\u{74}\u{61}\u{301}\u{67}\u{65}\u{300}\u{6e}";

/// The name "mütágèn" in NFC normalization.
const MUTAGEN_COMPOSED: &'static str =
    "\u{6d}\u{fc}\u{74}\u{e1}\u{67}\u{e8}\u{6e}";


#[test]
fn decomposes_unicode() {
    // Grab a test directory to work with.
    let directory = temp_dir();

    // Check if we expect it to decompose names.
    let decomposes = decomposes(&directory).unwrap();

    // Create a file with the composed name.
    let composed_path = directory.as_path().join(MUTAGEN_COMPOSED);
    let file = File::create(&composed_path).unwrap();
    drop(file);

    // Attempt to open the file with the decomposed name.
    // TODO: Think about whether or not this is actually an indication of
    // decomposition. Some filesystems might provide normalization insensitivity
    // in the same way that they provide case sensitivity.
    let decomposed_path = directory.as_path().join(MUTAGEN_DECOMPOSED);
    match File::open(&decomposed_path) {
        Ok(file) => {
            drop(file);
            remove_file(&decomposed_path).unwrap();
            assert!(decomposes);
        },
        Err(_) => {
            remove_file(&composed_path).unwrap();
            assert!(!decomposes);
        }
    }
}

// TODO: Ideally we should add tests for macOS where we invoke diskutil to
// create some test volumes with expected behavior.

#[test]
fn recompose_unicode_name() {
    assert_eq!(recompose(MUTAGEN_DECOMPOSED), MUTAGEN_COMPOSED);
}
