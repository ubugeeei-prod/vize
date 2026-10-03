//! One immutable derived file is visible to the real configured compiler.

use corsa::api::{ApiFileSystem, DirectoryEntries, FileSystemCapabilities, ReadFileResult};
use std::path::{Path, PathBuf};
use vize_l0::{String, cstr};

pub(super) struct ProjectionFileSystem {
    path: PathBuf,
    text: String,
    entries: DirectoryEntries,
}
impl ProjectionFileSystem {
    pub fn new(path: &Path, text: &str) -> Result<Self, String> {
        let parent = path
            .parent()
            .ok_or_else(|| cstr!("projection has no parent"))?;
        let mut entries = DirectoryEntries::default();
        for entry in std::fs::read_dir(parent)
            .map_err(|error| cstr!("cannot retain projection directory: {error}"))?
        {
            let entry = entry.map_err(|error| cstr!("cannot retain projection entry: {error}"))?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| cstr!("projection directory has non-UTF8 entries"))?;
            if entry.path().is_dir() {
                entries.directories.push(name.into());
            } else {
                entries.files.push(name.into());
            }
        }
        entries.files.push(
            path.file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| cstr!("projection filename is not UTF8"))?
                .into(),
        );
        entries.files.sort();
        entries.directories.sort();
        Ok(Self {
            path: path.to_path_buf(),
            text: text.into(),
            entries,
        })
    }
}
impl ApiFileSystem for ProjectionFileSystem {
    fn capabilities(&self) -> FileSystemCapabilities {
        FileSystemCapabilities {
            read_file: true,
            file_exists: true,
            get_accessible_entries: true,
            realpath: true,
            ..Default::default()
        }
    }
    fn read_file(&self, path: &str) -> ReadFileResult {
        if Path::new(path) == self.path {
            ReadFileResult::Content(self.text.as_str().into())
        } else {
            ReadFileResult::Fallback
        }
    }
    fn file_exists(&self, path: &str) -> Option<bool> {
        (Path::new(path) == self.path).then_some(true)
    }
    fn get_accessible_entries(&self, path: &str) -> Option<DirectoryEntries> {
        (Some(Path::new(path)) == self.path.parent()).then(|| self.entries.clone())
    }
    fn realpath(&self, path: &str) -> Option<corsa::fast::CompactString> {
        (Path::new(path) == self.path).then(|| path.into())
    }
}
