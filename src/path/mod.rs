//! Provides path computation and normalization routines.

#[cfg(test)]
mod tests;

use std::env::home_dir;
use std::fs::{create_dir, create_dir_all};
use std::path::{Path, PathBuf};

use super::errors::{Result, ResultExt, Error};

const MUTAGEN_DIRECTORY_NAME: &'static str = ".mutagen";

#[cfg(windows)]
fn mark_hidden<P: AsRef<Path>>(path: P) -> Result<()> {
    unimplemented!();
}

pub fn mutagen<P: AsRef<Path>>(subpath: P) -> Result<PathBuf> {
    // Compute the user's home directory.
    let home = home_dir()
                .ok_or(Error::from("unable to compute home directory"))?;

    // Compute the path to the Mutagen directory.
    let mutagen = home.as_path().join(MUTAGEN_DIRECTORY_NAME);

    // Ensure the Mutagen directory exists. Since it's a direct descendant of
    // the home directory (which should exist), we don't use create_dir_all.
    create_dir(&mutagen).chain_err(|| "unable to create Mutagen directory")?;

    // On Windows, ensure the Mutagen directory is hidden.
    #[cfg(windows)]
    mark_hidden(&mutagen)?;

    // Compute the path to the subdirectory.
    let result = mutagen.as_path().join(subpath);

    // Ensure the subdirectory exists.
    create_dir_all(&result)
        .chain_err(|| "unable to create Mutagen subdirectory")?;

    // Success.
    Ok(result)
}
