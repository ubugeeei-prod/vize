//! Shared test fixtures.

/// A temporary directory under the canonical temp root. macOS's `/var` is a
/// symlink to `/private/var`, and the server compares physical paths, so a
/// fixture rooted in `/var/...` would disagree with every path it resolves.
pub(crate) fn tempdir() -> std::io::Result<tempfile::TempDir> {
    let root = std::env::temp_dir();
    tempfile::tempdir_in(root.canonicalize().unwrap_or(root))
}
