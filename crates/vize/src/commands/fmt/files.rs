use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use std::path::{Component, Path, PathBuf};

use super::ignores::FmtIgnoreSet;
use super::patterns::is_format_extension;

const NODE_MODULES_DIR: &str = "node_modules";

#[cfg(test)]
#[path = "files_tests.rs"]
mod files_tests;

pub(crate) fn collect_files(
    patterns: &[impl AsRef<str>],
    ignore_set: Option<&FmtIgnoreSet>,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if vize_l0::path::is_git_metadata_path(&cwd) {
        return files;
    }

    for pattern in patterns {
        let normalized = normalize_fmt_pattern(pattern.as_ref());
        // File-based routes (`pages/[id].vue`) contain `[` but are literal files.
        // A glob would treat the brackets as a character class and skip them.
        let literal = PathBuf::from(normalized.as_str());
        if vize_l0::path::is_git_metadata_path(&literal) {
            continue;
        }
        if literal.is_file() {
            if should_include_format_file(&literal, ignore_set) {
                files.push(literal);
            }
            continue;
        }
        if contains_glob_char(&normalized) {
            if let Some(pattern) = FmtPattern::new(&normalized, &cwd) {
                collect_walked_files(&pattern, &normalized, ignore_set, &mut files);
            }
        } else {
            let path = PathBuf::from(&normalized);
            if path.is_file() && should_include_format_file(&path, ignore_set) {
                files.push(path);
            }
        }
    }

    files.sort();
    files.dedup();

    files
}

fn collect_walked_files(
    pattern: &FmtPattern,
    input: &str,
    ignore_set: Option<&FmtIgnoreSet>,
    files: &mut Vec<PathBuf>,
) {
    let mut root = PathBuf::new();
    for component in Path::new(input).components() {
        if contains_glob_char(&component.as_os_str().to_string_lossy()) {
            break;
        }
        root.push(component);
    }
    if root.as_os_str().is_empty() {
        root.push(".");
    }
    let respect_ignores = should_walk_with_gitignore(input);
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .ignore(respect_ignores)
        .git_ignore(respect_ignores)
        .git_global(respect_ignores)
        .git_exclude(respect_ignores)
        .require_git(false)
        .filter_entry(|entry| {
            !vize_l0::path::is_git_metadata_path(entry.path()) && !is_dependency_path(entry.path())
        })
        .build();

    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if pattern.matches(path) && should_include_format_file(path, ignore_set) {
            files.push(path.strip_prefix(".").unwrap_or(path).to_path_buf());
        }
    }
}

fn should_include_format_file(path: &Path, ignore_set: Option<&FmtIgnoreSet>) -> bool {
    !vize_l0::path::is_git_metadata_path(path)
        && path.is_file()
        && is_format_target(path)
        && !is_dependency_path(path)
        && !ignore_set.is_some_and(|ignore_set| ignore_set.is_ignored(path))
}

#[inline]
fn should_walk_with_gitignore(pattern: &str) -> bool {
    // `normalize_fmt_pattern` strips leading `./`, but accept it defensively.
    let bare = pattern.strip_prefix("./").unwrap_or(pattern);
    let path = Path::new(bare);
    contains_glob_char(bare)
        && !path.is_absolute()
        && !path
            .components()
            .any(|component| component == Component::ParentDir)
}

pub(super) struct FmtPattern {
    pattern: Pattern,
    cwd: PathBuf,
    absolute: bool,
}

impl FmtPattern {
    pub(super) fn new(pattern: &str, cwd: &Path) -> Option<Self> {
        let normalized = normalize_fmt_pattern(pattern);
        let absolute = Path::new(&normalized).is_absolute();
        Pattern::new(&normalized).ok().map(|pattern| Self {
            pattern,
            cwd: cwd.to_path_buf(),
            absolute,
        })
    }

    pub(super) fn matches(&self, path: &Path) -> bool {
        let candidate = if self.absolute {
            let relative = path.strip_prefix(".").unwrap_or(path);
            let absolute = if relative.is_absolute() {
                relative.to_path_buf()
            } else {
                self.cwd.join(relative)
            };
            normalize_path(&absolute)
        } else {
            normalize_path(path.strip_prefix(".").unwrap_or(path))
        };

        self.pattern
            .matches_with(candidate.as_str(), fmt_glob_match_options())
    }
}

fn normalize_fmt_pattern(pattern: &str) -> vize_l0::String {
    let mut normalized: vize_l0::String = pattern.replace('\\', "/").into();
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.into();
    }

    if normalized.is_empty() || normalized == "." {
        return "**/*".into();
    }

    if !contains_glob_char(&normalized) && Path::new(&normalized).is_dir() {
        if !normalized.ends_with('/') {
            normalized.push('/');
        }
        normalized.push_str("**/*");
    }

    normalized
}

#[inline]
fn is_format_target(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_format_extension)
}

#[inline]
fn normalize_path(path: &Path) -> vize_l0::String {
    path.to_string_lossy().replace('\\', "/").into()
}

#[inline]
fn contains_glob_char(pattern: &str) -> bool {
    pattern.contains(['*', '?', '['])
}

fn is_dependency_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == NODE_MODULES_DIR)
}

#[inline]
fn fmt_glob_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: !cfg!(windows),
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}

#[cfg(test)]
#[path = "files/contract_tests.rs"]
mod tests;
