use std::path::Path;

/// Every component strictly below `project_root` is a real file or directory.
///
/// Stopping at the root keeps a symlinked project root on the lexical
/// spelling. A metadata error falls back to canonicalization.
pub(super) fn ancestors_below_root_are_real(project_root: &Path, path: &Path) -> bool {
    let mut current = path;
    while current != project_root {
        match std::fs::symlink_metadata(current) {
            Ok(metadata) if !metadata_is_link(&metadata) => {}
            _ => return false,
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent,
            _ => return false,
        }
    }
    true
}

fn metadata_is_link(metadata: &std::fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        // Directory junctions are not `is_symlink()` but still rename the
        // canonical file, matching the pre-fast-path `canonicalize`.
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        return metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    }
    #[cfg(not(windows))]
    false
}
