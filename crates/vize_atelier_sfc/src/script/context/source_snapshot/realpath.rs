//! `realpath` in one path walk on Linux, and the `O_PATH` handles that prove it.
//!
//! glibc's `realpath` calls `readlink` on every prefix, so a path `n`
//! components deep walks `n(n+1)/2` components. On a WSL2 Windows mount (9P
//! without a dentry cache) each walked component is a round trip to the host.
//! An `O_PATH` open walks the path once without opening the file itself, and
//! the kernel names the physical file through `/proc/self/fd`. When that name
//! equals the spelling byte for byte, no component was a link (a link resolves
//! to another name) and no `.`, `..` or repeated separator was spelled, so the
//! spelling is its own `realpath`. Any other answer (a link, a case-insensitive
//! mount naming a cached spelling, no `/proc`) takes `std::fs::canonicalize`.

use std::io;
use std::path::{Path, PathBuf};

/// `std::fs::canonicalize`, answered in one path walk when the path is
/// already canonical.
pub(super) fn canonicalize(path: &Path) -> io::Result<PathBuf> {
    #[cfg(target_os = "linux")]
    if canonical_handle(path).is_some() {
        return Ok(path.to_path_buf());
    }
    std::fs::canonicalize(path)
}

/// An `O_PATH` handle on `path`, which resolves the path without opening the
/// file itself: a Windows mount answers no open, and the access mode is
/// ignored. `extra` adds `O_NOFOLLOW` or `O_DIRECTORY`.
#[cfg(target_os = "linux")]
pub(super) fn open_path(path: &Path, extra: libc::c_int) -> io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_PATH | extra)
        .open(path)
}

/// A handle on `path` when `path` is its own `realpath`.
#[cfg(target_os = "linux")]
pub(super) fn canonical_handle(path: &Path) -> Option<std::fs::File> {
    use std::os::fd::AsRawFd;
    use vize_carton::ToCompactString;
    if !path.is_absolute() {
        return None;
    }
    let file = open_path(path, 0).ok()?;
    let named = std::fs::read_link(
        Path::new("/proc/self/fd").join(file.as_raw_fd().to_compact_string().as_str()),
    )
    .ok()?;
    (named.as_os_str() == path.as_os_str()).then_some(file)
}
