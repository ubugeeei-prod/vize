//! Fixture globs for the corpus matrix.
//!
//! `glob` 0.3 follows directory links and keeps no cycle set. jellyfin-vue's
//! `packaging/deb/root` points at the repository root, so `**/*.vue` never
//! finishes and the requested-file manifest lists paths the checker does not
//! emit. A directory link is not descended when its canonical target is an
//! ancestor of the link, or already on this walk. A link to a sibling
//! directory is still listed: those paths are part of the baseline hash.

use glob::{MatchOptions, Pattern};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn collect_matching_files(cwd: &Path, patterns: &[String]) -> Result<Vec<String>, String> {
    let options = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };
    let compiled = patterns
        .iter()
        .map(|pattern| {
            let absolute = cwd.join(pattern);
            Pattern::new(&absolute.to_string_lossy())
                .map_err(|error| format!("invalid glob pattern {pattern}: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut files = Vec::new();
    let mut ancestors = Vec::new();
    walk(cwd, cwd, &compiled, &options, &mut ancestors, &mut files)?;
    files.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    files.dedup();
    Ok(files)
}

fn walk(
    root: &Path,
    directory: &Path,
    patterns: &[Pattern],
    options: &MatchOptions,
    ancestors: &mut Vec<PathBuf>,
    files: &mut Vec<String>,
) -> Result<(), String> {
    ancestors.push(directory.to_path_buf());
    let walked = walk_entries(root, directory, patterns, options, ancestors, files);
    ancestors.pop();
    walked
}

fn walk_entries(
    root: &Path,
    directory: &Path,
    patterns: &[Pattern],
    options: &MatchOptions,
    ancestors: &mut Vec<PathBuf>,
    files: &mut Vec<String>,
) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
    for entry in entries {
        let path = entry.path();
        if ignored_source_path(&path) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            visit_symlink(root, &path, patterns, options, ancestors, files)?;
        } else if metadata.is_dir() {
            walk(root, &path, patterns, options, ancestors, files)?;
        } else if metadata.is_file() {
            push_match(root, &path, patterns, options, files)?;
        }
    }
    Ok(())
}

fn visit_symlink(
    root: &Path,
    path: &Path,
    patterns: &[Pattern],
    options: &MatchOptions,
    ancestors: &mut Vec<PathBuf>,
    files: &mut Vec<String>,
) -> Result<(), String> {
    let Ok(followed) = fs::metadata(path) else {
        return Ok(());
    };
    if followed.is_dir() {
        if !directory_link_cycles(path, ancestors) {
            walk(root, path, patterns, options, ancestors, files)?;
        }
        return Ok(());
    }
    if followed.is_file() {
        push_match(root, path, patterns, options, files)?;
    }
    Ok(())
}

/// `true` when descending would walk back into an ancestor, including the
/// jellyfin-vue root link `packaging/deb/root` → `../..`.
fn directory_link_cycles(link: &Path, ancestors: &[PathBuf]) -> bool {
    let Ok(target) = fs::canonicalize(link) else {
        return true;
    };
    if let Some(parent) = link.parent() {
        if fs::canonicalize(parent).is_ok_and(|parent| parent.starts_with(&target)) {
            return true;
        }
    }
    ancestors
        .iter()
        .any(|ancestor| fs::canonicalize(ancestor).is_ok_and(|canonical| canonical == target))
}

fn push_match(
    root: &Path,
    path: &Path,
    patterns: &[Pattern],
    options: &MatchOptions,
    files: &mut Vec<String>,
) -> Result<(), String> {
    if !patterns
        .iter()
        .any(|pattern| pattern.matches_path_with(path, *options))
    {
        return Ok(());
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|error| format!("compiler input is outside fixture root: {error}"))?;
    files.push(relative.to_string_lossy().replace('\\', "/"));
    Ok(())
}

fn ignored_source_path(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        name == ".yarn" || name == "node_modules"
    })
}
