//! File discovery and path normalization for the lint command.

use glob::{MatchOptions, Pattern};
use ignore::{DirEntry, WalkBuilder};
use std::path::{Path, PathBuf};
use vize_l0::{FxHashSet, String};

mod ignores;
use ignores::expand_entry_ignore_patterns;

use super::patterns::{is_lint_extension, is_plain_script_extension, is_standalone_html_extension};
use crate::config;

pub(super) struct LintIgnoreSet {
    patterns: Vec<LintInputGlob>,
    project: Option<config::matcher::ProjectIgnoreSet>,
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
        (!patterns.is_empty()).then_some(Self {
            patterns,
            project: None,
        })
    }

    pub(super) fn for_project(
        ignores: &[config::ConfigEntryIgnore],
        config_dir: &Path,
    ) -> Option<Self> {
        config::matcher::ProjectIgnoreSet::new(ignores, config_dir).map(|project| Self {
            patterns: Vec::new(),
            project: Some(project),
        })
    }

    fn is_ignored(&self, path: &Path) -> bool {
        self.project.as_ref().map_or_else(
            || self.patterns.iter().any(|pattern| pattern.matches(path)),
            |project| project.is_ignored(path),
        )
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
    if vize_carton::path::is_git_metadata_path(&normalize_lint_input_path(dir)) {
        return false;
    }
    let explicitly_selected = matcher.map(|matcher| matcher.explicit_directories());
    for entry in WalkBuilder::new(dir)
        .standard_filters(true)
        .hidden(matcher.is_none())
        .filter_entry(move |entry| {
            !vize_carton::path::is_git_metadata_path(entry.path())
                && (entry.depth() == 0
                    || !entry.file_type().is_some_and(|kind| kind.is_dir())
                    || !is_default_excluded_dir(entry, explicitly_selected))
        })
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

/// Generated and dependency trees are never part of a broad lint discovery.
/// A literal input path or a glob that names one of them can still select it.
fn is_default_excluded_dir(
    entry: &DirEntry,
    explicitly_selected: Option<ExplicitDirectories>,
) -> bool {
    let name = entry.file_name().to_str();
    match name {
        Some(".vize") => !explicitly_selected.is_some_and(|dirs| dirs.vize),
        Some("node_modules") => !explicitly_selected.is_some_and(|dirs| dirs.node_modules),
        _ => false,
    }
}

fn add_lint_file(
    path: &Path,
    ignore_set: Option<&LintIgnoreSet>,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) -> bool {
    if vize_carton::path::is_git_metadata_path(path) || !is_lintable_path(path) {
        return false;
    }
    let normalized = normalize_lint_input_path(path);
    if vize_carton::path::is_git_metadata_path(&normalized) {
        return false;
    }
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
    let prefix = normalized.get(..glob_start).unwrap_or_default();
    if prefix.is_empty() || (glob_start < normalized.len() && !prefix.contains('/')) {
        return PathBuf::from(".");
    }
    if let Some(index) = prefix.rfind('/') {
        if index == 0 {
            return PathBuf::from("/");
        }
        if is_windows_drive_root(prefix, index) {
            return PathBuf::from(prefix.get(..=index).unwrap_or_default());
        }
        return PathBuf::from(prefix.get(..index).unwrap_or_default());
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
    explicit_directories: ExplicitDirectories,
}

#[derive(Clone, Copy)]
struct ExplicitDirectories {
    vize: bool,
    node_modules: bool,
}

impl LintInputGlob {
    fn new(pattern: &str) -> Option<Self> {
        let normalized = normalize_lint_glob_pattern(pattern);
        let absolute = Path::new(normalized.as_str()).is_absolute();
        // A directory in the literal prefix is already the walk root. Its
        // name must not opt all nested directories with that name back in.
        let dynamic_part = normalized
            .get(
                normalized
                    .find(['*', '?', '[', '{'])
                    .unwrap_or(normalized.len())..,
            )
            .unwrap_or_default();
        let explicit_directories = ExplicitDirectories {
            vize: dynamic_part.split('/').any(|part| part == ".vize"),
            node_modules: dynamic_part.split('/').any(|part| part == "node_modules"),
        };
        Pattern::new(normalized.as_str()).ok().map(|pattern| Self {
            pattern,
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            absolute,
            explicit_directories,
        })
    }

    fn explicit_directories(&self) -> ExplicitDirectories {
        self.explicit_directories
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

mod node_modules_ignore;

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
