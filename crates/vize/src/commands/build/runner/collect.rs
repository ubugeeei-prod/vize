//! File collection and glob pattern matching for the build command.

use std::path::PathBuf;

use ignore::Walk;
use vize_carton::cstr;
use vize_carton::{String, ToCompactString};

/// Collect `.vue` files matching the given glob patterns.
#[allow(clippy::disallowed_types)]
pub(super) fn collect_files(patterns: &[std::string::String]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for pattern in patterns {
        let (root, glob_pattern) = parse_pattern(pattern);

        for entry in Walk::new(&root).flatten() {
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "vue")
                && pattern_matches(path, &glob_pattern)
            {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();
    files.dedup();
    files
}

/// Extract a root directory and glob pattern from a user-provided pattern string.
fn parse_pattern(pattern: &str) -> (String, String) {
    if let Some(pos) = pattern.find(['*', '?']) {
        let root_part = &pattern[..pos];
        if let Some(last_slash) = root_part.rfind('/') {
            let root = &pattern[..last_slash];
            let root = if root.is_empty() { "." } else { root };
            return (root.to_compact_string(), pattern.to_compact_string());
        }
    }

    let path = std::path::Path::new(pattern);
    if path.is_dir() {
        return (pattern.to_compact_string(), cstr!("{}/**/*.vue", pattern));
    }

    if path.is_file()
        && pattern.ends_with(".vue")
        && let Some(parent) = path.parent()
    {
        let parent_str = parent.to_string_lossy();
        let parent_str = if parent_str.is_empty() {
            "."
        } else {
            &parent_str
        };
        return (parent_str.to_compact_string(), pattern.to_compact_string());
    }

    (".".into(), pattern.to_compact_string())
}

/// Check whether a file path matches a glob-like pattern.
#[allow(clippy::disallowed_types, clippy::disallowed_methods)]
fn pattern_matches(path: &std::path::Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy().replace("\\", "/");

    if pattern == "./**/*.vue" || pattern == "**/*.vue" {
        return path_str.ends_with(".vue");
    }

    if pattern.contains("**/*.vue")
        && let Some(prefix_end) = pattern.find("**")
    {
        let prefix = &pattern[..prefix_end];
        let prefix_normalized = prefix.trim_end_matches('/');
        let has_prefix_dir = prefix_normalized.is_empty()
            || path_str.match_indices(prefix_normalized).any(|(idx, _)| {
                path_str.as_bytes().get(idx + prefix_normalized.len()) == Some(&b'/')
            });
        return has_prefix_dir && path_str.ends_with(".vue");
    }

    if pattern.ends_with(".vue") {
        let pattern_normalized = pattern.replace("\\", "/");
        if path_str == pattern_normalized {
            return true;
        }

        if !path_str.ends_with(pattern_normalized.as_str()) {
            return false;
        }

        let prefix_len = path_str.len() - pattern_normalized.len();
        let Some(separator_idx) = prefix_len.checked_sub(1) else {
            return false;
        };
        return path_str.as_bytes().get(separator_idx) == Some(&b'/');
    }

    path_str.ends_with(".vue")
}
