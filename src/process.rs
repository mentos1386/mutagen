//! Provides an extension trait for `std::process::Command` objects with
//! Mutagen-specific functionality.

use std::process::Command;
#[cfg(unix)]
use std::os::unix::process::CommandExt as StdCommandExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt as StdCommandExt;

#[cfg(unix)]
use libc;

#[cfg(windows)]
use winapi::winbase::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS};

/// Provides an extension to `std::process::Command` that allows for
/// backgrounding child processes.
pub trait CommandExt {
    /// Modifies the command to start the process in the background. On Unix
    /// systems this is accomplished via `setsid` and on Windows systems it's
    /// accomplished via a combination of the `CREATE_NEW_PROCESS_GROUP` and
    /// `DETACHED_PROCESS` flags. The `setsid` function will always create a new
    /// process group, so the `new_group` argument has no effect on Unix
    /// systems, but if set to `false` on Windows systems it will exclude the
    /// `CREATE_NEW_PROCESS_GROUP` flag, merely creating a detached process.
    fn background(&mut self, new_group: bool) -> &mut Command;
}

impl CommandExt for Command {
    #[cfg(unix)]
    fn background(&mut self, _: bool) -> &mut Command {
        self.before_exec(|| {
            unsafe { libc::setsid(); }
            Ok(())
        })
    }

    #[cfg(windows)]
    fn background(&mut self, new_group: bool) -> &mut Command {
        if new_group {
            self.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
        } else {
            self.creation_flags(DETACHED_PROCESS)
        }
    }
}

// TODO: Add tests for backgrounding extension. Not sure how to go about this.
// Maybe start up a shell ("/bin/sh" or "cmd.exe") and have it write process
// information to a file? Not super pressing - the daemon and SSH subprocesses
// functioning will be a pretty good test.
