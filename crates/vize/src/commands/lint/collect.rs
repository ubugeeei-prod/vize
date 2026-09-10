//! File discovery and path normalization for the lint command.

use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use vize_s0::{FxHashSet, String};

use super::patterns::{is_lint_extension, is_plain_script_extension, is_standalone_html_extension};
use crate::config;

pub(super) struct LintIgnoreSet {
    patterns: Vec<LintInputGlob>,
}

pub(super) struct LintFileCollection {
    pub(super) files: Vec<PathBuf>,
    pub(super) unmatched_patterns: Vec<String>,
}

impl LintIgnoreSet {
    pub(super) fn new(ignores: &[config::ConfigEntryIgnore], config_dir: &Path) -> Option<Self> {
        let patterns = ignores
            .iter()
            .flat_map(|ignore| expand_entry_ignore_patterns(ignore, config_dir))
            .filter_map(|pattern| LintInputGlob::new(pattern.to_string_lossy().as_ref()))
            .collect::<Vec<_>>();
        (!patterns.is_empty()).then_some(Self { patterns })
    }

    fn is_ignored(&self, path: &Path) -> bool {
        self.patterns.iter().any(|pattern| pattern.matches(path))
    }
}

pub(super) fn collect_lint_file_collection(
    patterns: &[String],
    ignore_set: Option<&LintIgnoreSet>,
) -> LintFileCollection {
    let mut files = Vec::new();
    let mut unmatched_patterns = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                if !add_lint_file(&candidate, ignore_set, &mut files, &mut seen) {
                    unmatched_patterns.push(pattern.clone());
                }
                continue;
            }
            if candidate.is_dir() {
                if !collect_lint_files_from_dir(&candidate, None, ignore_set, &mut files, &mut seen)
                {
                    unmatched_patterns.push(pattern.clone());
                }
                continue;
            }
        }

        let base_dir = base_dir_from_lint_pattern(pattern);
        let matcher = LintInputGlob::new(pattern);
        if !collect_lint_files_from_dir(
            &base_dir,
            matcher.as_ref(),
            ignore_set,
            &mut files,
            &mut seen,
        ) {
            unmatched_patterns.push(pattern.clone());
        }
    }

    files.sort();
    LintFileCollection {
        files,
        unmatched_patterns,
    }
}

pub(super) fn collect_lint_inputs(
    patterns: &[String],
    ignore_set: Option<&LintIgnoreSet>,
) -> (Vec<PathBuf>, usize) {
    let LintFileCollection {
        files,
        unmatched_patterns,
    } = collect_lint_file_collection(patterns, ignore_set);
    let warning_count = if files.is_empty() {
        0
    } else {
        super::patterns::write_unmatched_explicit_patterns(patterns, &unmatched_patterns)
    };
    (files, warning_count)
}

fn collect_lint_files_from_dir(
    dir: &Path,
    matcher: Option<&LintInputGlob>,
    ignore_set: Option<&LintIgnoreSet>,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) -> bool {
    let mut matched = false;
    for entry in WalkBuilder::new(dir)
        .standard_filters(true)
        .hidden(matcher.is_none())
        .build()
    {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_file() && matcher.is_none_or(|matcher| matcher.matches(path)) {
            matched |= add_lint_file(path, ignore_set, files, seen);
        }
    }
    matched
}

fn add_lint_file(
    path: &Path,
    ignore_set: Option<&LintIgnoreSet>,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) -> bool {
    if !is_lintable_path(path) {
        return false;
    }
    let normalized = normalize_lint_input_path(path);
    if ignore_set.is_some_and(|ignore_set| ignore_set.is_ignored(&normalized)) {
        return false;
    }
    if seen.insert(normalized.clone()) {
        files.push(normalized);
    }
    true
}

fn is_lintable_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_lint_extension)
}

pub(super) fn is_standalone_html_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_standalone_html_extension)
}

pub(super) fn is_plain_script_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_plain_script_extension)
}

fn base_dir_from_lint_pattern(pattern: &str) -> PathBuf {
    let normalized = normalize_lint_glob_pattern(pattern);
    let glob_start = normalized
        .find(['*', '?', '[', '{'])
        .unwrap_or(normalized.len());
    let prefix = &normalized[..glob_start];
    if prefix.is_empty() || (glob_start < normalized.len() && !prefix.contains('/')) {
        return PathBuf::from(".");
    }
    if let Some(index) = prefix.rfind('/') {
        if index == 0 {
            return PathBuf::from("/");
        }
        if is_windows_drive_root(prefix, index) {
            return PathBuf::from(&prefix[..=index]);
        }
        return PathBuf::from(&prefix[..index]);
    }
    PathBuf::from(prefix)
}

fn is_windows_drive_root(prefix: &str, slash_index: usize) -> bool {
    slash_index == 2 && prefix.as_bytes().get(1) == Some(&b':')
}

struct LintInputGlob {
    pattern: Pattern,
    cwd: PathBuf,
    absolute: bool,
}

impl LintInputGlob {
    fn new(pattern: &str) -> Option<Self> {
        let normalized = normalize_lint_glob_pattern(pattern);
        let absolute = Path::new(normalized.as_str()).is_absolute();
        Pattern::new(normalized.as_str()).ok().map(|pattern| Self {
            pattern,
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            absolute,
        })
    }

    fn matches(&self, path: &Path) -> bool {
        let candidate = if self.absolute {
            let absolute = if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.cwd.join(path)
            };
            normalize_lint_path(&absolute)
        } else {
            normalize_lint_path(path)
        };

        self.pattern
            .matches_with(&candidate, lint_glob_match_options())
    }
}

fn normalize_lint_glob_pattern(pattern: &str) -> String {
    strip_lint_current_dir_prefix(&pattern.replace('\\', "/"))
}

fn normalize_lint_path(path: &Path) -> String {
    strip_lint_current_dir_prefix(&path.to_string_lossy().replace('\\', "/"))
}

fn strip_lint_current_dir_prefix(value: &str) -> String {
    let mut normalized = value;
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped;
    }
    normalized.into()
}

fn normalize_lint_input_path(path: &Path) -> PathBuf {
    PathBuf::from(normalize_lint_path(path))
}

pub(super) fn resolve_lint_config_path(config_dir: &Path, candidate: &str) -> PathBuf {
    let path = Path::new(candidate);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    config_dir.join(path)
}

fn expand_entry_ignore_patterns(
    ignore: &config::ConfigEntryIgnore,
    config_dir: &Path,
) -> Vec<PathBuf> {
    let resolved = resolve_entry_ignore_pattern(ignore, config_dir);
    let Some(deep_pattern) = nested_node_modules_ignore(&resolved) else {
        return vec![resolved];
    };
    vec![resolved, deep_pattern]
}

fn resolve_entry_ignore_pattern(ignore: &config::ConfigEntryIgnore, config_dir: &Path) -> PathBuf {
    let pattern = Path::new(ignore.pattern.as_str());
    if pattern.is_absolute() {
        return if pattern.exists() {
            vize_s0::path::canonicalize_non_verbatim(pattern)
        } else {
            pattern.to_path_buf()
        };
    }

    let config_dir = absolute_config_dir(config_dir);
    let base = ignore
        .base_path
        .as_deref()
        .map(Path::new)
        .filter(|base_path| !base_path.as_os_str().is_empty());
    match base {
        Some(base_path) if base_path.is_absolute() => base_path.join(pattern),
        Some(base_path) => config_dir.join(base_path).join(pattern),
        None => config_dir.join(pattern),
    }
}

fn absolute_config_dir(config_dir: &Path) -> PathBuf {
    if config_dir.is_absolute() {
        return config_dir.to_path_buf();
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(config_dir)
}

fn nested_node_modules_ignore(pattern: &Path) -> Option<PathBuf> {
    let pattern_text = normalize_lint_path(pattern);
    let suffix = "node_modules/**";
    if !pattern_text.ends_with(suffix) || pattern_text.contains("**/node_modules/**") {
        return None;
    }
    let prefix = pattern_text.trim_end_matches(suffix).trim_end_matches('/');
    Some(PathBuf::from(
        vize_s0::cstr!("{prefix}/**/{suffix}").as_str(),
    ))
}

fn lint_glob_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}

#[cfg(test)]
#[path = "collect_tests.rs"]
mod tests;
