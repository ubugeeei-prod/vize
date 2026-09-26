//! Stable committed corpus and exact input identities, loaded before timing.

use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use vize_s0::{String, ToCompactString, cstr};

pub struct Input {
    pub filename: String,
    pub source: String,
    pub sha256: String,
}

fn collect(root: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            let excluded = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| matches!(name, "_git" | "_git-worktrees" | "node_modules"));
            if !excluded {
                collect(&path, paths)?;
            }
        } else if path.extension().is_some_and(|extension| extension == "vue") {
            paths.push(path);
        }
    }
    Ok(())
}

pub fn hash(bytes: &[u8]) -> String {
    cstr!("{:x}", Sha256::digest(bytes))
}

pub fn load(root: &Path) -> std::io::Result<Vec<Input>> {
    let mut paths = Vec::new();
    collect(root, &mut paths)?;
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let filename = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_compact_string();
            let source = fs::read_to_string(path)?;
            let sha256 = hash(source.as_bytes());
            Ok(Input {
                filename,
                source: String::from(source),
                sha256,
            })
        })
        .collect()
}

pub fn manifest_hash(inputs: &[Input]) -> String {
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input.filename.as_bytes());
        hasher.update([0]);
        hasher.update(input.sha256.as_bytes());
        hasher.update([0]);
    }
    cstr!("{:x}", hasher.finalize())
}
