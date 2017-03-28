//! Provides path computation and normalization routines.

#[cfg(test)]
mod tests;

use std::env::home_dir;
use std::fs::{create_dir, create_dir_all};
#[cfg(windows)]
use std::io::{Error as IOError};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use kernel32::{GetFileAttributesW, SetFileAttributesW};
#[cfg(windows)]
use winapi::{FILE_ATTRIBUTE_HIDDEN, INVALID_FILE_ATTRIBUTES};

use super::errors::{Result, ResultExt, Error};

const MUTAGEN_DIRECTORY_NAME: &'static str = ".mutagen";

#[cfg(windows)]
fn mark_hidden<P: AsRef<Path>>(path: P) -> Result<()> {
    // Convert the path to a format we can use. Because Windows expects the
    // string to be null-terminated, we have to extend the encoding sequence
    // with a null character.
    let path_utf16: Vec<u16> =
        path.as_ref().as_os_str().encode_wide().chain(Some(0)).collect();

    // Now we have to drop into unsafe mode to invoke the actual system calls.
    unsafe {
        // Grab the current file attributes.
        let mut attributes = GetFileAttributesW(path_utf16.as_ptr());
        if attributes == INVALID_FILE_ATTRIBUTES {
            return Err(IOError::last_os_error())
                    .chain_err(|| "unable to get file attributes");
        }

        // Add the hidden attribute.
        attributes |= FILE_ATTRIBUTE_HIDDEN;

        // Set the file attributes.
        if SetFileAttributesW(path_utf16.as_ptr(), attributes) == 0 {
            Err(IOError::last_os_error())
                .chain_err(|| "unable to set file attributes")
        } else {
            Ok(())
        }
    }
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
