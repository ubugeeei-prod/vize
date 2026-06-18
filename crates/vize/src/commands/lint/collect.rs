//! File discovery and path normalization for the lint command.

use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use vize_carton::{FxHashSet, String};

pub(super) fn collect_lint_files(patterns: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                add_lint_file(&candidate, &mut files, &mut seen);
                continue;
            }
            if candidate.is_dir() {
                collect_lint_files_from_dir(&candidate, None, &mut files, &mut seen);
                continue;
            }
        }

        let base_dir = base_dir_from_lint_pattern(pattern);
        let matcher = LintInputGlob::new(pattern);
        collect_lint_files_from_dir(&base_dir, matcher.as_ref(), &mut files, &mut seen);
    }

    files.sort();
    files
}

fn collect_lint_files_from_dir(
    dir: &Path,
    matcher: Option<&LintInputGlob>,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) {
    for entry in WalkBuilder::new(dir)
        .standard_filters(true)
        .hidden(true)
        .build()
    {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_file() && matcher.is_none_or(|matcher| matcher.matches(path)) {
            add_lint_file(path, files, seen);
        }
    }
}

fn add_lint_file(path: &Path, files: &mut Vec<PathBuf>, seen: &mut FxHashSet<PathBuf>) {
    if !is_lintable_path(path) {
        return;
    }
    let normalized = normalize_lint_input_path(path);
    if seen.insert(normalized.clone()) {
        files.push(normalized);
    }
}

fn is_lintable_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("vue" | "html" | "htm" | "js" | "mjs" | "cjs" | "ts" | "mts" | "cts" | "jsx" | "tsx",)
    )
}

pub(super) fn is_standalone_html_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("html" | "htm")
    )
}

pub(super) fn is_plain_script_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("js" | "mjs" | "cjs" | "ts" | "mts" | "cts")
    )
}

fn base_dir_from_lint_pattern(pattern: &str) -> PathBuf {
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

fn lint_glob_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}

#[cfg(test)]
mod tests {
    use super::collect_lint_files;
    use std::fs;

    #[test]
    fn collection_includes_vue_html_scripts_and_jsx() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("App.vue"), "").unwrap();
        fs::write(src.join("index.html"), "").unwrap();
        fs::write(src.join("config.js"), "").unwrap();
        fs::write(src.join("store.ts"), "").unwrap();
        fs::write(src.join("Panel.jsx"), "").unwrap();
        fs::write(src.join("Widget.tsx"), "").unwrap();
        fs::write(src.join("notes.md"), "").unwrap();

        let files = collect_lint_files(&[src.display().to_string().into()]);

        assert_eq!(
            files,
            vec![
                src.join("App.vue"),
                src.join("Panel.jsx"),
                src.join("Widget.tsx"),
                src.join("config.js"),
                src.join("index.html"),
                src.join("store.ts")
            ]
        );
    }
}
