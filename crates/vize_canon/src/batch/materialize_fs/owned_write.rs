//! Replacing a cache leaf must never truncate another hardlink to its inode.

use std::fs::Metadata;
use std::io::{self, Write};
use std::path::Path;

use vize_carton::profiler::global_profiler;

pub(in crate::batch) fn must_replace(metadata: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.nlink() > 1
    }
    #[cfg(not(unix))]
    {
        // Stable Windows metadata does not expose the hardlink count. Only
        // changed, existing files reach here; cold files and unchanged writes
        // retain their existing fast path on every platform.
        let _ = metadata;
        true
    }
}

pub(in crate::batch) fn replace(path: &Path, content: &[u8]) -> io::Result<()> {
    let result = (|| {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "cache file has no parent")
        })?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(content)?;
        temporary.persist(path).map_err(|error| error.error)?;
        Ok(())
    })();
    match result {
        Ok(()) => global_profiler().record_fs_write(content.len()),
        Err(_) => global_profiler().record_fs_write_failure(content.len()),
    }
    result
}

#[cfg(test)]
mod tests {
    use super::super::write_if_changed;

    #[test]
    fn replacing_a_hardlinked_cache_leaf_keeps_original_bytes_and_mtime() {
        let directory = tempfile::tempdir().unwrap();
        let original = directory.path().join("installed.ts");
        let cache = directory.path().join("owned.ts");
        std::fs::write(&original, b"export const original = 1;\n").unwrap();
        std::fs::hard_link(&original, &cache).unwrap();
        let modified = std::fs::metadata(&original).unwrap().modified().unwrap();
        write_if_changed(&cache, b"export const replacement = 'new';\n").unwrap();
        assert_eq!(
            std::fs::read(&original).unwrap(),
            b"export const original = 1;\n"
        );
        assert_eq!(
            std::fs::metadata(&original).unwrap().modified().unwrap(),
            modified
        );
        assert_eq!(
            std::fs::read(&cache).unwrap(),
            b"export const replacement = 'new';\n"
        );
    }
}
