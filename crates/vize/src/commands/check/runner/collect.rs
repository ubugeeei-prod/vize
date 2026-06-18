//! Input collection for the `check` runner.
//!
//! The direct runner and socket runner both use these helpers to normalize
//! explicit paths, globs, and directories into stable file lists.

use std::path::{Path, PathBuf};

use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use vize_carton::{FxHashSet, String};

use super::ignores::CheckIgnoreSet;

const TARGET_DIR: &str = "target";
const NODE_MODULES_DIR: &str = "node_modules";
const VIZE_CACHE_DIR: &str = ".vize";

#[cfg(test)]
#[allow(clippy::disallowed_types)]
pub(super) fn collect_check_files(
    patterns: &[std::string::String],
    include_jsx: bool,
) -> Vec<PathBuf> {
    collect_check_files_with_ignores(patterns, include_jsx, None)
}

#[allow(clippy::disallowed_types)]
pub(super) fn collect_check_files_with_ignores(
    patterns: &[std::string::String],
    include_jsx: bool,
    ignore_set: Option<&CheckIgnoreSet>,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                let candidate = normalize_input_path(&candidate);
                if is_supported_check_file(&candidate, include_jsx)
                    && !is_ignored(&candidate, ignore_set)
                    && seen.insert(candidate.clone())
                {
                    files.push(candidate);
                }
                continue;
            }
            if candidate.is_dir() {
                collect_from_dir(&candidate, &mut files, &mut seen, include_jsx, ignore_set);
                continue;
            }
        }

        let base_dir = base_dir_from_pattern(pattern);
        let matcher = InputGlob::new(pattern);
        collect_from_dir_with_matcher(
            base_dir.as_path(),
            &mut files,
            &mut seen,
            include_jsx,
            matcher.as_ref(),
            ignore_set,
        );
    }

    files.sort();
    files
}

#[allow(clippy::disallowed_types)]
pub(super) fn collect_vue_files(patterns: &[std::string::String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                let candidate = normalize_input_path(&candidate);
                if candidate
                    .extension()
                    .and_then(|extension| extension.to_str())
                    == Some("vue")
                    && seen.insert(candidate.clone())
                {
                    files.push(candidate);
                }
                continue;
            }
            if candidate.is_dir() {
                collect_from_dir_filtered(
                    &candidate, &mut files, &mut seen, true, false, None, None,
                );
                continue;
            }
        }

        let base_dir = base_dir_from_pattern(pattern);
        let matcher = InputGlob::new(pattern);
        collect_from_dir_filtered(
            &base_dir,
            &mut files,
            &mut seen,
            true,
            false,
            matcher.as_ref(),
            None,
        );
    }

    files.sort();
    files
}

fn collect_from_dir(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
    include_jsx: bool,
    ignore_set: Option<&CheckIgnoreSet>,
) {
    collect_from_dir_with_matcher(dir, files, seen, include_jsx, None, ignore_set);
}

fn collect_from_dir_with_matcher(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
    include_jsx: bool,
    matcher: Option<&InputGlob>,
    ignore_set: Option<&CheckIgnoreSet>,
) {
    collect_from_dir_filtered(dir, files, seen, false, include_jsx, matcher, ignore_set);
}

fn collect_from_dir_filtered(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
    vue_only: bool,
    include_jsx: bool,
    matcher: Option<&InputGlob>,
    ignore_set: Option<&CheckIgnoreSet>,
) {
    // Walk with the `ignore` crate so repository-level ignore rules prune whole
    // subtrees before we test patterns. The root path is canonicalized once and
    // reattached to each relative entry below; doing `canonicalize` per file was
    // the old hot path when checking large workspaces.
    let skip_generated = should_skip_generated_for_root(dir);
    let normalized_dir = normalize_input_path(dir);
    let walker = WalkBuilder::new(dir)
        .standard_filters(true)
        .hidden(true)
        .build_parallel();

    let collected = std::sync::Mutex::new(Vec::<PathBuf>::new());
    walker.run(|| {
        let collected = &collected;
        let normalized_dir = normalized_dir.clone();
        Box::new(move |entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file()
                    && is_supported_collect_file(path, vue_only, include_jsx)
                    && matcher.is_none_or(|matcher| matcher.matches(path))
                    && (!skip_generated || !is_generated_path(path))
                    && !is_ignored(path, ignore_set)
                    && let Ok(mut collected) = collected.lock()
                {
                    collected.push(normalize_walked_path(dir, &normalized_dir, path));
                }
            }
            ignore::WalkState::Continue
        })
    });

    let Ok(collected) = collected.into_inner() else {
        return;
    };
    for path in collected {
        if seen.insert(path.clone()) {
            files.push(path);
        }
    }
}

fn is_ignored(path: &Path, ignore_set: Option<&CheckIgnoreSet>) -> bool {
    ignore_set.is_some_and(|ignore_set| ignore_set.is_ignored(path))
}

fn base_dir_from_pattern(pattern: &str) -> PathBuf {
    let glob_start = pattern.find(['*', '?', '[', '{']).unwrap_or(pattern.len());
    let prefix = &pattern[..glob_start];
    let base = if prefix.is_empty() {
        "."
    } else if let Some(index) = prefix.rfind('/') {
        &prefix[..index]
    } else {
        prefix
    };
    if base.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(base)
    }
}

pub(super) struct InputGlob {
    pattern: Pattern,
    cwd: PathBuf,
    absolute: bool,
}

impl InputGlob {
    pub(super) fn new(pattern: &str) -> Option<Self> {
        let normalized = normalize_glob_pattern(pattern);
        let absolute = Path::new(normalized.as_str()).is_absolute();
        Pattern::new(normalized.as_str()).ok().map(|pattern| Self {
            pattern,
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            absolute,
        })
    }

    pub(super) fn matches(&self, path: &Path) -> bool {
        let candidate = if self.absolute {
            let absolute = if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.cwd.join(path)
            };
            normalize_path(&absolute)
        } else {
            normalize_path(path)
        };

        self.pattern.matches_with(&candidate, glob_match_options())
    }
}

fn normalize_glob_pattern(pattern: &str) -> String {
    strip_leading_current_dir(&pattern.replace('\\', "/"))
}

fn normalize_path(path: &Path) -> String {
    strip_leading_current_dir(&path.to_string_lossy().replace('\\', "/"))
}

fn strip_leading_current_dir(value: &str) -> String {
    let mut normalized = value;
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped;
    }
    normalized.into()
}

fn normalize_input_path(path: &Path) -> PathBuf {
    vize_carton::path::canonicalize_non_verbatim(path)
}

fn normalize_walked_path(root: &Path, normalized_root: &Path, path: &Path) -> PathBuf {
    // Avoid a canonicalize syscall per walked file; normalize the root once.
    path.strip_prefix(root)
        .map(|relative| normalized_root.join(relative))
        .unwrap_or_else(|_| normalize_input_path(path))
}

fn should_skip_generated_for_root(root: &Path) -> bool {
    !path_is_generated_root(root)
}

fn is_generated_path(path: &Path) -> bool {
    let mut previous = None;
    path.components().any(|component| {
        let Some(name) = component.as_os_str().to_str() else {
            previous = None;
            return false;
        };
        let generated = is_generated_component(previous, name);
        previous = Some(name);
        generated
    })
}

fn path_is_generated_root(path: &Path) -> bool {
    let mut previous = None;
    for component in path.components() {
        let Some(name) = component.as_os_str().to_str() else {
            previous = None;
            continue;
        };
        if is_generated_component(previous, name) {
            return true;
        }
        previous = Some(name);
    }
    false
}

fn is_generated_component(previous: Option<&str>, name: &str) -> bool {
    name == TARGET_DIR || (previous == Some(NODE_MODULES_DIR) && name == VIZE_CACHE_DIR)
}

fn is_supported_collect_file(path: &Path, vue_only: bool, include_jsx: bool) -> bool {
    if vue_only {
        return path.extension().and_then(|extension| extension.to_str()) == Some("vue");
    }
    is_supported_check_file(path, include_jsx)
}

fn is_supported_check_file(path: &Path, include_jsx: bool) -> bool {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
    {
        return true;
    }

    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(extension, "vue" | "ts" | "tsx" | "mts" | "cts")
                || (include_jsx && extension == "jsx")
        })
}

fn glob_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: !cfg!(windows),
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}

#[cfg(test)]
#[path = "collect_tests.rs"]
mod tests;
