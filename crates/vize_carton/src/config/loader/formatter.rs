//! Formatter settings projected from one host config read.
use super::{
    LoadedConfigWithFeatures, LoadedRawConfig, load_raw_config_checked, load_raw_config_with_source,
};
use crate::config::{ConfigEntryFiles, ConfigEntryIgnore, SortImportsSetting};
use std::path::Path;

/// All formatting and source-selection settings from one configuration evaluation.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct LoadedFormatterSnapshot {
    pub loaded: LoadedConfigWithFeatures,
    pub sort_imports: Option<SortImportsSetting>,
    pub entries: Vec<ConfigEntryFiles>,
    pub ignores: Vec<ConfigEntryIgnore>,
}

/// Load stable formatting options, Vue features, and import sorting in one parse.
pub fn load_config_with_formatter_options_and_source(
    path: Option<&Path>,
) -> (LoadedConfigWithFeatures, Option<SortImportsSetting>) {
    let snapshot = snapshot(load_raw_config_with_source(path));
    (snapshot.loaded, snapshot.sort_imports)
}

/// Reject invalid configuration before any source selection or writes.
pub fn try_load_formatter_snapshot(
    path: Option<&Path>,
) -> Result<LoadedFormatterSnapshot, std::string::String> {
    load_raw_config_checked(path).map(snapshot)
}

fn snapshot(loaded: LoadedRawConfig) -> LoadedFormatterSnapshot {
    let sort_imports = loaded.config.formatter_sort_imports().cloned();
    let ignores = loaded.config.entry_ignores();
    let entries = loaded.config.clone().into_entry_files();
    let (config, features) = loaded.config.into_config_and_features();
    LoadedFormatterSnapshot {
        loaded: LoadedConfigWithFeatures {
            config,
            features,
            source_path: loaded.source_path,
        },
        sort_imports,
        entries,
        ignores,
    }
}
