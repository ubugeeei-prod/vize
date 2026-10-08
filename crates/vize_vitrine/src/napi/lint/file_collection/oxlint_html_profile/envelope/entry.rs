use std::fs;
use std::path::Path;

use super::super::{Refusal, RefusalKind};

pub(in super::super) fn entry_refusal(
    path: &Path,
    is_symlink: bool,
    is_dir: bool,
    is_file: bool,
    repository: &Path,
    root_json: &Path,
    custom: &str,
) -> Option<Refusal> {
    if path.to_str().is_none() {
        return Some(Refusal::new(
            RefusalKind::UnsupportedFileType,
            path,
            "non-UTF-8 path is unqualified",
        ));
    }
    if is_symlink {
        return Some(Refusal::new(
            RefusalKind::Symlink,
            path,
            "symlink descendants are unqualified",
        ));
    }
    if !is_dir && !is_file {
        return Some(Refusal::new(
            RefusalKind::UnsupportedFileType,
            path,
            "nonregular descendants are unqualified",
        ));
    }
    if is_dir && path != repository {
        for name in [".git", ".jj"] {
            match fs::symlink_metadata(path.join(name)) {
                Ok(_) => {
                    return Some(Refusal::new(
                        RefusalKind::NestedVcs,
                        path,
                        "nested VCS boundary is unqualified",
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Some(Refusal::io(path, error)),
            }
        }
    }
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if !is_dir
        && path != root_json
        && (filename.starts_with(".oxlintrc.") || filename.starts_with("oxlint.config."))
    {
        let mut refusal = Refusal::new(
            RefusalKind::NestedConfig,
            path,
            "only explicit root JSON authority is qualified",
        );
        refusal.original_bytes = fs::read(path).ok();
        return Some(refusal);
    }
    if is_dir {
        for name in [".gitignore", custom] {
            let source = path.join(name);
            match fs::symlink_metadata(&source) {
                Ok(metadata) if metadata.is_symlink() => {
                    return Some(Refusal::new(
                        RefusalKind::Symlink,
                        &source,
                        "ignore symlink is unqualified",
                    ));
                }
                Ok(metadata) if !metadata.is_file() => {
                    return Some(Refusal::new(
                        RefusalKind::UnsupportedFileType,
                        &source,
                        "ignore source must be regular",
                    ));
                }
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                    return Some(Refusal::io(&source, error));
                }
                _ => {}
            }
        }
    }
    None
}
