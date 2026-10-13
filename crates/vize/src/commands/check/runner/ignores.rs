use std::path::{Path, PathBuf};

use super::collect::InputGlob;
use crate::config;

pub(super) struct CheckIgnoreSet {
    patterns: Vec<InputGlob>,
    project: Option<config::matcher::ProjectIgnoreSet>,
}

impl CheckIgnoreSet {
    pub(super) fn new(ignores: &[config::ConfigEntryIgnore], config_dir: &Path) -> Option<Self> {
        let patterns = ignores
            .iter()
            .flat_map(|ignore| expand_entry_ignore_patterns(ignore, config_dir))
            .filter_map(|pattern| InputGlob::new(pattern.to_string_lossy().as_ref()))
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

    pub(super) fn is_ignored(&self, path: &Path) -> bool {
        self.project.as_ref().map_or_else(
            || self.patterns.iter().any(|pattern| pattern.matches(path)),
            |project| project.is_ignored(path),
        )
    }
}

pub(super) fn retain_unignored(files: &mut Vec<PathBuf>, ignore_set: Option<&CheckIgnoreSet>) {
    if let Some(ignore_set) = ignore_set {
        files.retain(|path| !ignore_set.is_ignored(path));
    }
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

fn absolute_config_dir(config_dir: &Path) -> PathBuf {
    if config_dir.is_absolute() {
        return config_dir.to_path_buf();
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(config_dir)
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
