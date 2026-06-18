use std::path::{Path, PathBuf};

use super::{FmtArgs, files::FmtPattern};
use crate::config;

pub(super) struct FmtIgnoreSet {
    patterns: Vec<FmtPattern>,
}

impl FmtIgnoreSet {
    pub(super) fn new(ignores: &[config::ConfigEntryIgnore], config_dir: &Path) -> Option<Self> {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let patterns = ignores
            .iter()
            .filter_map(|ignore| {
                let pattern = resolve_entry_ignore_pattern(ignore, config_dir);
                FmtPattern::new(pattern.to_string_lossy().as_ref(), &cwd)
            })
            .collect::<Vec<_>>();
        (!patterns.is_empty()).then_some(Self { patterns })
    }

    pub(super) fn is_ignored(&self, path: &Path) -> bool {
        self.patterns.iter().any(|pattern| pattern.matches(path))
    }
}

pub(super) fn load_fmt_ignore_set(args: &FmtArgs) -> Option<FmtIgnoreSet> {
    if args.no_config {
        return None;
    }
    let loaded = config::load_config_entry_ignores_with_source(args.config.as_deref());
    if loaded.ignores.is_empty() {
        return None;
    }
    let config_dir = loaded
        .source_path
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    FmtIgnoreSet::new(&loaded.ignores, &config_dir)
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
