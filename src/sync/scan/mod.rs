//! Provides filesystem scanning functionality.

mod ignore;
#[cfg(target_os = "macos")]
mod sys;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::super::errors::{Result, ResultExt};
use super::super::hash::{Algorithm, Hasher};
pub use super::super::proto::sync::{CacheEntry, Cache};
use super::super::time::{AsTimestamp, Order as TimeOrder};
use super::entry::Entry;
use self::ignore::Ignorer;
#[cfg(target_os = "macos")]
use self::sys::macos::{decomposes_unicode, recompose_unicode_name};

struct Scanner<'a> {
    root: &'a Path,
    hash: &'a Algorithm,
    cache: &'a Cache,
    new_cache: Cache,
    ignorer: Ignorer,
}

impl<'a> Scanner<'a> {
    fn scan_file(&mut self,
                 path: String,
                 metadata: &fs::Metadata) -> Result<Entry> {
        // On Windows, nothing is executable.
        #[cfg(windows)]
        let executable = false;

        // On POSIX systems, we treat any executable bit being set as indicating
        // executability.
        #[cfg(unix)]
        let executable = (metadata.permissions().mode() & 0x111) != 0;

        // Extract modification time and convert it to a Protocol Buffers
        // timestamp.
        let modification_time =
            metadata.modified()
                    .chain_err(|| "unable to read modification time")?
                    .as_timestamp()
                    .chain_err(|| "unable to convert modification time")?;

        // Extract size.
        let size = metadata.len();

        // Try to find a matching cache entry so that we don't need to recompute
        // the digest. We only enforce that modification time and size haven't
        // changed.
        let mut cache_hit = self.cache.get_entries().get(&path);
        if let Some(entry) = cache_hit {
            let matching = entry.has_modification_time() &&
                TimeOrder::compute(
                    entry.get_modification_time(),
                    &modification_time
                ) == TimeOrder::Equal &&
                entry.size == size;
            if !matching {
                cache_hit = None;
            }
        }

        // Compute the digest. If we have a valid cache hit, re-use the digest,
        // otherwise do a full recomputation.
        let digest = match cache_hit {
            Some(entry) => entry.digest.clone(),
            None => {
                let full_path = self.root.join(&path);
                let mut file = fs::File::open(&full_path)
                                        .chain_err(|| "unable to open file")?;
                let mut hasher = Hasher::new(self.hash);
                let copied = io::copy(&mut file, &mut hasher)
                                .chain_err(|| "unable to hash file contents")?;
                if copied != size {
                    bail!("short copy when hashing");
                }
                hasher.digest().0
            }
        };

        // Create an entry in the new cache.
        // TODO: If we want to keep the mode bit option open, we need to set it
        // here. I think we should try to emulate Go's fakery for now.
        let mut new_cache_entry = CacheEntry::new();
        new_cache_entry.set_modification_time(modification_time);
        new_cache_entry.set_size(size);
        new_cache_entry.set_digest(digest.clone());
        self.new_cache.mut_entries().insert(path, new_cache_entry);

        // Success.
        Ok(Entry::File{executable: executable, digest: digest})
    }

    fn scan_directory(&mut self, path: String) -> Result<Entry> {
        // Create the contents container.
        let mut contents = BTreeMap::new();

        // Compute the full path by combining it with the scan root.
        let full_path = self.root.join(&path);

        // If we're on macOS, there's a possibility that the filesystem at this
        // path decomposes Unicode names, so we need to check for that.
        #[cfg(target_os = "macos")]
        let decomposes = decomposes_unicode(&full_path)
                            .chain_err(|| "unable to check Unicode behavior")?;

        // Read the directory contents.
        let entries = fs::read_dir(&full_path)
                            .chain_err(|| "unable to read directory entries")?;

        // Process directory contents.
        for entry in entries {
            // Unwrap the entry.
            let entry = entry.chain_err(|| "unable to read directory entry")?;

            // If we're on a non-macOS system, extract the name and convert it
            // to an owned string.
            #[cfg(not(target_os = "macos"))]
            let entry_name = match entry.file_name().into_string() {
                Ok(name) => name,
                Err(_) => bail!("unable to convert entry name to UTF-8"),
            };

            // If we're on a macOS system, we have a bit more work to do,
            // because we might need to do Unicode normalization, but we'll
            // still end up with an owned string.
            #[cfg(target_os = "macos")]
            let entry_name = match entry.file_name().to_str() {
                Some(name) => if decomposes {
                    recompose_unicode_name(name)
                } else {
                    name.to_owned()
                },
                None => bail!("unable to convert entry name to UTF-8"),
            };

            // Compute the content path relative to the root. We do this
            // manually (rather than Path::join) because we want the result to
            // be a String rather than a PathBuf. That saves us some UTF-8
            // validation and additional allocations due to conversions. We also
            // need '/' joins on Windows (where they are still valid paths) in
            // order for globs to work.
            let entry_path = if path.len() > 0 {
                format!("{}/{}", path, entry_name)
            } else {
                entry_name.clone()
            };

            // Check if the path is ignored.
            if self.ignorer.ignored(&entry_path) {
                continue
            }

            // Grab the entry metadata.
            let metadata = entry.metadata()
                                .chain_err(|| "unable to read entry metadata")?;

            // Check the entry kind and handle appropriately.
            // TODO: Is it possible for std::fs::FileType to return true for
            // is_symlink in addition to is_file or is_dir? If so, we need to
            // eliminate that possibility first.
            let entry_type = metadata.file_type();
            if entry_type.is_file() {
                let file = self.scan_file(entry_path, &metadata)
                                .chain_err(|| "unable to scan file")?;
                contents.insert(entry_name, file);
            } else if entry_type.is_dir() {
                let directory = self.scan_directory(entry_path)
                                    .chain_err(|| "unable to scan directory")?;
                contents.insert(entry_name, directory);
            }
        }

        // Success.
        Ok(Entry::Directory(contents))
    }
}

pub fn scan<P, I, S>(path: P,
                     hash: &Algorithm,
                     cache: &Cache,
                     ignores: I) -> Result<(Entry, Cache)> where
                    P: AsRef<Path>,
                    I: IntoIterator<Item=S>,
                    S: AsRef<str> {
    // Create a scanner.
    let mut scanner = Scanner{
        root: path.as_ref(),
        hash: hash,
        cache: cache,
        new_cache: Cache::new(),
        ignorer: Ignorer::new(ignores)
                    .chain_err(|| "unable to construct ignorer")?,
    };

    // Grab root metadata. We use the metadata function (which follows
    // symlinks), as opposed to symlink_metadata, because we DO want to follow
    // symbolic links at the root.
    let metadata = fs::metadata(path.as_ref())
                    .chain_err(|| "unable to get path metadata")?;

    // Handle based on the root type.
    let root_type = metadata.file_type();
    let root_entry = if root_type.is_file() {
        scanner.scan_file("".to_owned(), &metadata)
    } else if root_type.is_dir() {
        scanner.scan_directory("".to_owned())
    } else {
        bail!("unknown root entry type");
    }.chain_err(|| "unable to scan root")?;

    // Success.
    Ok((root_entry, scanner.new_cache))
}
