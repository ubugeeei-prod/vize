use glob::glob;
use std::path::{Path, PathBuf};
use vize_carton::String;

use super::patterns::has_explicit_patterns;
use super::{FmtArgs, files::FmtPattern};
use crate::config;

#[allow(clippy::disallowed_types)]
pub(super) struct ResolvedFmtPatterns {
    pub(super) values: Vec<String>,
    pub(super) explicit: bool,
}

pub(super) struct FmtEntryFileSet {
    entries: Vec<FmtEntryFileScope>,
}

#[allow(clippy::disallowed_types)]
pub(super) fn resolve_patterns(args: &FmtArgs) -> ResolvedFmtPatterns {
    let explicit = has_explicit_patterns(&args.patterns);
    let values = if explicit {
        load_fmt_entry_file_set(args)
            .as_ref()
            .map(|entry_file_set| entry_file_set.expand_patterns(&args.patterns))
            .unwrap_or_else(|| compact_patterns(&args.patterns))
    } else {
        compact_patterns(&args.patterns)
    };
    ResolvedFmtPatterns { values, explicit }
}

impl FmtEntryFileSet {
    #[allow(clippy::disallowed_types)]
    pub(super) fn expand_patterns(&self, patterns: &[std::string::String]) -> Vec<String> {
        let mut expanded = Vec::new();
        for pattern in patterns {
            push_unique(&mut expanded, pattern.as_str().into());
            if local_pattern_has_match(pattern) {
                continue;
            }
            for entry in &self.entries {
                entry.expand_missing_pattern(pattern, &mut expanded);
            }
        }
        expanded
    }
}

pub(super) fn load_fmt_entry_file_set(args: &FmtArgs) -> Option<FmtEntryFileSet> {
    if args.no_config {
        return None;
    }
    let loaded = config::load_config_entry_files_with_source(args.config.as_deref());
    if loaded.entries.is_empty() {
        return None;
    }
    let config_dir = loaded
        .source_path
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let entries = loaded
        .entries
        .into_iter()
        .filter_map(|entry| FmtEntryFileScope::new(entry, &config_dir))
        .collect::<Vec<_>>();
    (!entries.is_empty()).then_some(FmtEntryFileSet { entries })
}

struct FmtEntryFileScope {
    base_dir: PathBuf,
    base_prefix: Option<String>,
    files: Vec<EntryFilePattern>,
}

impl FmtEntryFileScope {
    fn new(entry: config::ConfigEntryFiles, config_dir: &Path) -> Option<Self> {
        let files = entry
            .files
            .into_iter()
            .filter_map(|pattern| EntryFilePattern::new(pattern.as_str()))
            .collect::<Vec<_>>();
        if files.is_empty() {
            return None;
        }
        let base_path = entry.base_path.as_deref();
        Some(Self {
            base_dir: resolve_base_dir(base_path, config_dir),
            base_prefix: normalize_base_prefix(base_path),
            files,
        })
    }

    fn expand_missing_pattern(&self, pattern: &str, expanded: &mut Vec<String>) {
        let normalized = normalize_pattern_text(pattern);
        let pattern_path = Path::new(pattern);
        if pattern_path.is_absolute() {
            if let Some(relative) = strip_prefix_text(pattern_path, &self.base_dir)
                && self.accepts_entry_relative_pattern(relative.as_str())
            {
                push_unique(expanded, normalize_path_text(pattern_path));
            }
            return;
        }

        if let Some(relative) = self.entry_relative_from_root_pattern(normalized.as_str())
            && self.accepts_entry_relative_pattern(relative)
        {
            push_unique(expanded, join_pattern(&self.base_dir, relative));
        }
        if self.accepts_entry_relative_pattern(normalized.as_str()) {
            push_unique(expanded, join_pattern(&self.base_dir, normalized.as_str()));
        }
    }

    fn entry_relative_from_root_pattern<'a>(&self, pattern: &'a str) -> Option<&'a str> {
        let base_prefix = self.base_prefix.as_ref()?;
        if pattern == base_prefix {
            return Some(".");
        }
        pattern
            .strip_prefix(base_prefix.as_str())
            .and_then(|relative| relative.strip_prefix('/'))
    }

    fn accepts_entry_relative_pattern(&self, pattern: &str) -> bool {
        let normalized = normalize_pattern_text(pattern);
        if normalized.is_empty() || normalized == "." {
            return true;
        }
        if contains_glob_char(normalized.as_str()) {
            return self
                .files
                .iter()
                .any(|file| glob_patterns_may_overlap(file.raw.as_str(), normalized.as_str()));
        }
        let path = Path::new(normalized.as_str());
        if is_format_target(path) {
            return self.files.iter().any(|file| file.matcher.matches(path));
        }
        let directory_prefix = format!("{}/", normalized.trim_end_matches('/'));
        self.files
            .iter()
            .any(|file| file.raw == normalized || file.raw.starts_with(directory_prefix.as_str()))
    }
}

struct EntryFilePattern {
    raw: String,
    matcher: FmtPattern,
}

impl EntryFilePattern {
    fn new(pattern: &str) -> Option<Self> {
        let raw = normalize_pattern_text(pattern);
        FmtPattern::new(raw.as_str(), Path::new(".")).map(|matcher| Self { raw, matcher })
    }
}

fn resolve_base_dir(base_path: Option<&str>, config_dir: &Path) -> PathBuf {
    let Some(base_path) = base_path.filter(|base_path| !base_path.is_empty()) else {
        return config_dir.to_path_buf();
    };
    let path = Path::new(base_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        config_dir.join(path)
    }
}

fn normalize_base_prefix(base_path: Option<&str>) -> Option<String> {
    let mut base = normalize_pattern_text(base_path?);
    while let Some(stripped) = base.strip_prefix("./") {
        base = stripped.into();
    }
    base = base.trim_end_matches('/').into();
    if base.is_empty() || base == "." || Path::new(base.as_str()).is_absolute() {
        None
    } else {
        Some(base)
    }
}

fn local_pattern_has_match(pattern: &str) -> bool {
    let normalized = normalize_pattern_text(pattern);
    if contains_glob_char(normalized.as_str()) {
        return glob(normalized.as_str())
            .map(|paths| paths.filter_map(Result::ok).next().is_some())
            .unwrap_or(false);
    }
    Path::new(normalized.as_str()).exists()
}

fn glob_patterns_may_overlap(configured: &str, requested: &str) -> bool {
    configured == requested || {
        let configured_prefix = static_prefix_before_glob(configured);
        let requested_prefix = static_prefix_before_glob(requested);
        configured_prefix.is_empty()
            || requested_prefix.is_empty()
            || configured_prefix.starts_with(requested_prefix)
            || requested_prefix.starts_with(configured_prefix)
    }
}

fn static_prefix_before_glob(pattern: &str) -> &str {
    let glob_index = pattern.find(['*', '?', '[']).unwrap_or(pattern.len());
    let prefix = &pattern[..glob_index];
    prefix
        .rfind('/')
        .map(|index| &prefix[..=index])
        .unwrap_or("")
}

fn strip_prefix_text(path: &Path, base: &Path) -> Option<String> {
    path.strip_prefix(base).ok().map(normalize_path_text)
}

fn join_pattern(base: &Path, pattern: &str) -> String {
    if pattern == "." {
        return normalize_path_text(base);
    }
    normalize_path_text(&base.join(pattern))
}

fn normalize_pattern_text(pattern: &str) -> String {
    let mut normalized: String = pattern.replace('\\', "/").into();
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.into();
    }
    normalized
}

fn normalize_path_text(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").into()
}

fn push_unique(patterns: &mut Vec<String>, pattern: String) {
    if !patterns.contains(&pattern) {
        patterns.push(pattern);
    }
}

#[allow(clippy::disallowed_types)]
fn compact_patterns(patterns: &[std::string::String]) -> Vec<String> {
    patterns
        .iter()
        .map(|pattern| pattern.as_str().into())
        .collect()
}

fn contains_glob_char(pattern: &str) -> bool {
    pattern.contains(['*', '?', '['])
}

fn is_format_target(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "vue" | "jsx" | "tsx" | "js" | "mjs" | "cjs" | "ts" | "mts" | "cts"
            )
        })
}
