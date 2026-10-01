//! Vue source discovery for inspector payloads.

use ignore::WalkBuilder;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use vize_l0::String;

pub(super) fn collect_files(patterns: &[String], max_files: Option<usize>) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();

    for pattern in patterns {
        let path = Path::new(pattern.as_str());
        if path.is_file() {
            if is_vue_file(path) {
                files.insert(path.to_path_buf());
            }
            continue;
        }

        if path.is_dir() {
            if collect_walked_vue_files(path, &mut files, max_files) {
                return files.into_iter().collect();
            }
            continue;
        }

        if let Some(root) = recursive_vue_glob_root(pattern)
            && root.exists()
        {
            if collect_walked_vue_files(&root, &mut files, max_files) {
                return files.into_iter().collect();
            }
            continue;
        }

        match glob::glob(pattern.as_str()) {
            Ok(paths) => {
                for path in paths.flatten() {
                    if path.is_file() && is_vue_file(&path) {
                        files.insert(path);
                        if max_files.is_some_and(|limit| files.len() >= limit) {
                            return files.into_iter().collect();
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("Invalid glob pattern {pattern}: {error}");
                std::process::exit(1);
            }
        }
    }

    let mut files: Vec<_> = files.into_iter().collect();
    if let Some(limit) = max_files {
        files.truncate(limit);
    }
    files
}

fn collect_walked_vue_files(
    root: &Path,
    files: &mut BTreeSet<PathBuf>,
    max_files: Option<usize>,
) -> bool {
    for entry in WalkBuilder::new(root).require_git(false).build().flatten() {
        let entry_path = entry.path();
        if entry_path.is_file() && is_vue_file(entry_path) {
            files.insert(entry_path.to_path_buf());
            if max_files.is_some_and(|limit| files.len() >= limit) {
                return true;
            }
        }
    }

    false
}

fn recursive_vue_glob_root(pattern: &str) -> Option<PathBuf> {
    let normalized = pattern.replace('\\', "/");
    if normalized == "**/*.vue" || normalized == "./**/*.vue" {
        return Some(PathBuf::from("."));
    }

    let root = normalized.strip_suffix("/**/*.vue")?;
    if root.is_empty() || contains_glob_meta(root) {
        return None;
    }

    Some(PathBuf::from(root))
}

fn contains_glob_meta(value: &str) -> bool {
    value
        .bytes()
        .any(|byte| matches!(byte, b'*' | b'?' | b'[' | b']' | b'{' | b'}'))
}

fn is_vue_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}
