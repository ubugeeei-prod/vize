//! Formatter settings projected from one host config read.
use super::{LoadedConfigWithFeatures, load_raw_config_with_source};
use crate::config::SortImportsSetting;
use std::path::Path;

/// Load stable formatting options, Vue features, and import sorting in one parse.
pub fn load_config_with_formatter_options_and_source(
    path: Option<&Path>,
) -> (LoadedConfigWithFeatures, Option<SortImportsSetting>) {
    let loaded = load_raw_config_with_source(path);
    let sort_imports = loaded.config.formatter_sort_imports().cloned();
    let (config, features) = loaded.config.into_config_and_features();
    (
        LoadedConfigWithFeatures {
            config,
            features,
            source_path: loaded.source_path,
        },
        sort_imports,
    )
}
