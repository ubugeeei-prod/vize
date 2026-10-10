use std::path::{Path, PathBuf};

use super::files::FmtPattern;
use crate::config;

pub(crate) struct FmtIgnoreSet {
    patterns: Vec<FmtPattern>,
    project: Option<config::matcher::ProjectIgnoreSet>,
}

impl FmtIgnoreSet {
    pub(super) fn new(ignores: &[config::ConfigEntryIgnore], config_dir: &Path) -> Option<Self> {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let patterns = ignores
            .iter()
            .flat_map(|ignore| expand_entry_ignore_patterns(ignore, config_dir))
            .filter_map(|pattern| FmtPattern::new(pattern.to_string_lossy().as_ref(), &cwd))
            .collect::<Vec<_>>();
        (!patterns.is_empty()).then_some(Self {
            patterns,
            project: None,
        })
    }

    pub(super) fn is_ignored(&self, path: &Path) -> bool {
        self.project.as_ref().map_or_else(
            || self.patterns.iter().any(|pattern| pattern.matches(path)),
            |project| project.is_ignored(path),
        )
    }
}

pub(super) fn load_fmt_ignore_set(
    snapshot: &config::LoadedFormatterSnapshot,
) -> Option<FmtIgnoreSet> {
    if snapshot.ignores.is_empty() {
        return None;
    }
    let config_dir = snapshot
        .loaded
        .source_path
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    if snapshot.project_root.is_some() {
        config::matcher::ProjectIgnoreSet::new(&snapshot.ignores, &config_dir).map(|project| {
            FmtIgnoreSet {
                patterns: Vec::new(),
                project: Some(project),
            }
        })
    } else {
        FmtIgnoreSet::new(&snapshot.ignores, &config_dir)
    }
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
        return pattern.to_path_buf();
    }

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

fn nested_node_modules_ignore(pattern: &Path) -> Option<PathBuf> {
    let pattern_text = pattern.to_string_lossy().replace('\\', "/");
    let suffix = "node_modules/**";
    if !pattern_text.ends_with(suffix) || pattern_text.contains("**/node_modules/**") {
        return None;
    }
    let prefix = pattern_text.trim_end_matches(suffix).trim_end_matches('/');
    Some(PathBuf::from(
        vize_l0::cstr!("{prefix}/**/{suffix}").as_str(),
    ))
}
