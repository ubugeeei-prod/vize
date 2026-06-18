use std::path::{Path, PathBuf};

use super::{super::CheckArgs, collect::InputGlob};
use crate::config;

pub(super) struct CheckIgnoreSet {
    patterns: Vec<InputGlob>,
}

impl CheckIgnoreSet {
    pub(super) fn new(ignores: &[config::ConfigEntryIgnore], config_dir: &Path) -> Option<Self> {
        let patterns = ignores
            .iter()
            .filter_map(|ignore| {
                let pattern = resolve_entry_ignore_pattern(ignore, config_dir);
                InputGlob::new(pattern.to_string_lossy().as_ref())
            })
            .collect::<Vec<_>>();
        (!patterns.is_empty()).then_some(Self { patterns })
    }

    pub(super) fn is_ignored(&self, path: &Path) -> bool {
        self.patterns.iter().any(|pattern| pattern.matches(path))
    }
}

pub(super) fn load_check_ignore_set(args: &CheckArgs, config_dir: &Path) -> Option<CheckIgnoreSet> {
    if args.no_config {
        return None;
    }
    let loaded_ignores = config::load_config_entry_ignores_with_source(args.config.as_deref());
    CheckIgnoreSet::new(&loaded_ignores.ignores, config_dir)
}

pub(super) fn retain_unignored(files: &mut Vec<PathBuf>, ignore_set: Option<&CheckIgnoreSet>) {
    if let Some(ignore_set) = ignore_set {
        files.retain(|path| !ignore_set.is_ignored(path));
    }
}

fn resolve_entry_ignore_pattern(ignore: &config::ConfigEntryIgnore, config_dir: &Path) -> PathBuf {
    let pattern = Path::new(ignore.pattern.as_str());
    if pattern.is_absolute() {
        return pattern.to_path_buf();
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
