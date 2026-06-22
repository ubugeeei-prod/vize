//! File discovery and path normalization for the lint command.

use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use vize_carton::{FxHashSet, String};

use super::LintArgs;
use crate::config;

pub(super) struct LintIgnoreSet {
    patterns: Vec<LintInputGlob>,
}

impl LintIgnoreSet {
    fn new(ignores: &[config::ConfigEntryIgnore], config_dir: &Path) -> Option<Self> {
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

pub(super) fn load_lint_ignore_set(args: &LintArgs, config_dir: &Path) -> Option<LintIgnoreSet> {
    if args.no_config {
        return None;
    }
    let loaded_ignores = config::load_config_entry_ignores_with_source(args.config.as_deref());
    LintIgnoreSet::new(&loaded_ignores.ignores, config_dir)
}

pub(super) fn collect_lint_files(
    patterns: &[String],
    ignore_set: Option<&LintIgnoreSet>,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                add_lint_file(&candidate, ignore_set, &mut files, &mut seen);
                continue;
            }
            if candidate.is_dir() {
                collect_lint_files_from_dir(&candidate, None, ignore_set, &mut files, &mut seen);
                continue;
            }
        }

        let base_dir = base_dir_from_lint_pattern(pattern);
        let matcher = LintInputGlob::new(pattern);
        collect_lint_files_from_dir(
            &base_dir,
            matcher.as_ref(),
            ignore_set,
            &mut files,
            &mut seen,
        );
    }

    files.sort();
    files
}

fn collect_lint_files_from_dir(
    dir: &Path,
    matcher: Option<&LintInputGlob>,
    ignore_set: Option<&LintIgnoreSet>,
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
            add_lint_file(path, ignore_set, files, seen);
        }
    }
}

fn add_lint_file(
    path: &Path,
    ignore_set: Option<&LintIgnoreSet>,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) {
    if !is_lintable_path(path) {
        return;
    }
    let normalized = normalize_lint_input_path(path);
    if !ignore_set.is_some_and(|ignore_set| ignore_set.is_ignored(&normalized))
        && seen.insert(normalized.clone())
    {
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
            vize_carton::path::canonicalize_non_verbatim(pattern)
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
        vize_carton::cstr!("{prefix}/**/{suffix}").as_str(),
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
mod tests {
    use super::{LintIgnoreSet, collect_lint_files};
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

        let files = collect_lint_files(&[src.display().to_string().into()], None);

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

    #[test]
    fn collection_applies_config_ignores_and_nested_node_modules() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        let scripts_dep = dir.path().join("scripts/node_modules/chalk/source");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&scripts_dep).unwrap();
        fs::write(src.join("App.vue"), "").unwrap();
        fs::write(src.join("Generated.vue"), "").unwrap();
        fs::write(scripts_dep.join("index.d.ts"), "").unwrap();

        let ignore_set = LintIgnoreSet::new(
            &[
                crate::config::ConfigEntryIgnore {
                    base_path: None,
                    pattern: "src/Generated.vue".into(),
                },
                crate::config::ConfigEntryIgnore {
                    base_path: None,
                    pattern: "node_modules/**".into(),
                },
            ],
            dir.path(),
        );
        let files = collect_lint_files(
            &[dir.path().display().to_string().into()],
            ignore_set.as_ref(),
        );

        assert_eq!(files, vec![src.join("App.vue")]);
    }
}
