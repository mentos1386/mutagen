//! Provides infrastructure for the Mutagen daemon.

use std::fs::File;

use fs2::FileExt;

use super::errors::{Result, ResultExt};
use super::path::mutagen as mutagen_path;

const SUBDIRECTORY_NAME: &'static str = "daemon";
const LOCK_NAME: &'static str = "daemon.lock";

pub struct Lock {
    lock_file: File,
}

impl Lock {
    pub fn acquire() -> Result<Lock> {
        // Compute the path to the daemon directory and ensure it exists.
        let daemon_path = mutagen_path(SUBDIRECTORY_NAME)
                            .chain_err(|| "unable to compute daemon path")?;

        // Compute the lock path.
        let lock_path = daemon_path.as_path().join(LOCK_NAME);

        // Open the lock file.
        let lock_file = File::create(lock_path)
                            .chain_err(|| "unable to open lock file")?;

        // Attempt to acquire the lock.
        lock_file.try_lock_exclusive().chain_err(|| "unable to acquire lock")?;

        // Success.
        Ok(Lock{lock_file: lock_file})
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        self.lock_file.unlock().expect("lock release failure");
    }
}
