//! File collection and glob pattern matching for the build command.

use std::path::PathBuf;

use ignore::Walk;
use vize_carton::cstr;
use vize_carton::{String, ToCompactString};

pub(super) struct CollectedFiles {
    pub files: Vec<PathBuf>,
    pub roots: Vec<PathBuf>,
}

/// Collect `.vue` files matching the given glob patterns.
#[allow(clippy::disallowed_types)]
pub(super) fn collect_files(patterns: &[std::string::String]) -> CollectedFiles {
    let mut files = Vec::new();
    let mut roots = Vec::new();

    for pattern in patterns {
        let (root, glob_pattern) = parse_pattern(pattern);
        let root_path = PathBuf::from(root.as_str());
        if root_path.is_dir() {
            roots.push(root_path);
        }

        for entry in Walk::new(&root).flatten() {
            let path = entry.path();

            if path.is_file()
                && path.extension().is_some_and(|ext| ext == "vue")
                && pattern_matches(path, &glob_pattern)
            {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();
    files.dedup();
    roots.sort();
    roots.dedup();
    CollectedFiles { files, roots }
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

#[cfg(test)]
mod tests {
    use super::collect_files;
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use vize_carton::ToCompactString;

    #[test]
    fn collect_files_ignores_vue_extension_directories() {
        let root = unique_case_dir("build-vue-extension-directories");
        let src = root.join("src");
        let component_dir = src.join("Directory.vue");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&component_dir).unwrap();
        fs::write(src.join("App.vue"), "<template><div /></template>").unwrap();
        fs::write(
            component_dir.join("Nested.vue"),
            "<template><div /></template>",
        )
        .unwrap();

        let collected = collect_files(&vec![root.display().to_string()]);
        let _ = fs::remove_dir_all(&root);

        let mut expected = vec![component_dir.join("Nested.vue"), src.join("App.vue")];
        expected.sort();
        assert_eq!(collected.files, expected);
        assert_eq!(collected.roots, vec![root]);
    }

    #[test]
    fn collect_files_keeps_direct_vue_file_patterns() {
        let root = unique_case_dir("build-direct-vue-file");
        let src = root.join("src");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&src).unwrap();
        let app = src.join("App.vue");
        let sibling = src.join("Sibling.vue");
        fs::write(&app, "<template><div /></template>").unwrap();
        fs::write(sibling, "<template><div /></template>").unwrap();

        let collected = collect_files(&vec![app.display().to_string()]);
        let _ = fs::remove_dir_all(&root);

        assert_eq!(collected.files, vec![app]);
        assert_eq!(collected.roots, vec![src]);
    }

    #[test]
    fn collect_files_keeps_empty_searched_roots() {
        let root = unique_case_dir("build-empty-searched-root");
        let alpha = root.join("packages/alpha/src");
        let beta = root.join("packages/beta/src");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&alpha).unwrap();
        fs::create_dir_all(&beta).unwrap();
        let app = alpha.join("App.vue");
        fs::write(&app, "<template><div /></template>").unwrap();

        let collected = collect_files(&[
            alpha.to_string_lossy().into_owned(),
            beta.to_string_lossy().into_owned(),
        ]);
        let _ = fs::remove_dir_all(&root);

        assert_eq!(collected.files, vec![app]);
        assert_eq!(collected.roots, vec![alpha, beta]);
    }

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join(format!(
                "{}-{}-{}",
                name,
                std::process::id(),
                case_id.to_compact_string()
            ))
    }
}
