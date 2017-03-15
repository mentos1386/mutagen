//! Provides macOS-specific functionality for the scan module.

#[cfg(test)]
mod tests;

use std::ffi;
use std::os::unix::ffi::OsStrExt;
use std::mem;
use std::path::Path;
use libc;

use unicode_normalization::UnicodeNormalization;

use super::super::super::super::errors::{Result, ResultExt};

pub fn decomposes_unicode<P: AsRef<Path>>(path: P) -> Result<bool> {
    // Convert the path to a C string suitable for FFI usage.
    let path = ffi::CString::new(path.as_ref().as_os_str().as_bytes())
                    .chain_err(|| "unable to convert path")?;

    // Create a zeroed-out statfs structure. There's no documented requirements
    // for the initialization of this structure, so this is the easiest thing to
    // do and should be safe.
    let mut stat: libc::statfs = unsafe{ mem::zeroed() };

    // Invoke statfs and verify its success.
    let result = unsafe { libc::statfs(path.as_ptr(), &mut stat)};
    // TODO: Can we grab errno and convert it to a message here?
    ensure!(result == 0, "statfs failed");

    // Check if the f_fstypename field starts with "hfs". This is *not* the
    // ideal way of doing this, but unfortunately macOS' statvfs and statfs
    // implementations are a bit terrible. According to the man pages, the
    // f_fsid field of the statvfs structure is not meaningful, and it is
    // seemingly not populated. The man pages also say that the f_type field of
    // the statfs structure is reserved, but there is no documentation of its
    // value. Before macOS 10.12, its value was 17 for HFS volumes (including
    // all HFS variants such as HFS+), but then it changed to 23. The only place
    // this value is available is in the XNU sources (xnu/bsd/vfs/vfs_conf.c),
    // and those aren't even available for 10.12 yet. Other people have solved
    // this by checking for both:
    //  http://stackoverflow.com/questions/39350259
    //  https://trac.macports.org/ticket/52463
    //  https://github.com/jrk/afsctool/commit/1146c90
    // But this doesn't seem ideal, especially with APFS coming soon. Thus, the
    // only sensible recourse is to use f_fstypename field, which is BARELY
    // documented. I suspect this is what's being used by NSWorkspace's
    // getFileSystemInfoForPath... method.
    // Anyway, since the name field is stack-allocated and we zero it on every
    // call, we can just check the first 3 bytes directly. The name seems to be
    // "hfs" even for HFS+.
    Ok(stat.f_fstypename[0] == ('h' as libc::c_char) &&
       stat.f_fstypename[1] == ('f' as libc::c_char) &&
       stat.f_fstypename[2] == ('s' as libc::c_char))
}

/// Converts a UTF-8 string to use NFC normalization. This might not be perfect
/// for fixing HFS' behavior, because HFS actually uses a custom variant of NFD,
/// but my understanding is that it's just NFD with certain CJK characters not
/// decomposed. It's evolved a lot over time though and is under-documented, so
/// it is difficult to say. The iconv library has a "utf-8-mac" encoding that is
/// supposed to be the same as this pseudo-NFD format, so we could use that to
/// convert back to NFC, but I think the end result would be the same. This link
/// has a lot of information:
///  https://bugzilla.mozilla.org/show_bug.cgi?id=703161
/// Of course, this isn't perfect in the case that the file name originally had
/// a NFD normalization, but converting to NFC should be a fairly decent
/// approximation for most cases because almost every other system will use NFC
/// for Unicode filenames. Well, actually, that's not true, they usually don't
/// enforce a normalization, they just store the code points that they get, so
/// in theory we could see NFD or other normalization coming from other systems,
/// but that is less likely and this is really the best we can do. Once Apple
/// decides to take HFS out behind the shed and shoot it, this should be less
/// of an issue (unless they end up propagating this behavior to AppleFS, but it
/// sounds like they have not).
pub fn recompose_unicode_name(name: &str) -> String {
    name.nfc().collect::<String>()
}
