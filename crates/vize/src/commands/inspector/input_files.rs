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
        if vize_carton::path::is_git_metadata_path(path) {
            continue;
        }
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

        match collect_glob_files(pattern, &mut files, max_files) {
            Ok(true) => return files.into_iter().collect(),
            Ok(false) => {}
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
    if vize_carton::path::is_git_metadata_path(&vize_carton::path::canonicalize_non_verbatim(root))
    {
        return false;
    }
    for entry in WalkBuilder::new(root)
        .require_git(false)
        .filter_entry(|entry| !vize_carton::path::is_git_metadata_path(entry.path()))
        .build()
        .flatten()
    {
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
    !vize_carton::path::is_git_metadata_path(path)
        && path.extension().is_some_and(|extension| extension == "vue")
}

fn collect_glob_files(
    input: &str,
    files: &mut BTreeSet<PathBuf>,
    max_files: Option<usize>,
) -> Result<bool, glob::PatternError> {
    let pattern = glob::Pattern::new(input)?;
    let mut root = PathBuf::new();
    for component in Path::new(input).components() {
        if contains_glob_meta(&component.as_os_str().to_string_lossy()) {
            break;
        }
        root.push(component);
    }
    if root.as_os_str().is_empty() {
        root.push(".");
    }
    if vize_carton::path::is_git_metadata_path(&vize_carton::path::canonicalize_non_verbatim(&root))
    {
        return Ok(false);
    }
    let options = glob::MatchOptions {
        require_literal_separator: true,
        ..Default::default()
    };
    for entry in WalkBuilder::new(&root)
        .hidden(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .parents(false)
        .filter_entry(|entry| !vize_carton::path::is_git_metadata_path(entry.path()))
        .build()
        .flatten()
    {
        let path = entry.path();
        if path.is_file() && is_vue_file(path) && pattern.matches_path_with(path, options) {
            files.insert(path.to_path_buf());
            if max_files.is_some_and(|limit| files.len() >= limit) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::collect_files;
    use vize_l0::ToCompactString;

    #[test]
    fn inspector_prunes_metadata_for_generic_globs_and_explicit_inputs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let source = root.join(".github/App.vue");
        let metadata = root.join(".git/worktrees/cache/Snapshot.vue");
        for path in [&source, &metadata] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "<template />").unwrap();
        }
        let pattern = root.join("*/*/*.vue").to_string_lossy().to_compact_string();
        assert!(collect_files(&[pattern], None).is_empty());
        let pattern = root.join("**/*").to_string_lossy().to_compact_string();
        assert_eq!(collect_files(&[pattern], None), vec![source]);
        for path in [metadata, root.join(".git")] {
            assert!(collect_files(&[path.to_string_lossy().to_compact_string()], None).is_empty());
        }
    }
}
