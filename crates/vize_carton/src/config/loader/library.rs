use std::path::{Path, PathBuf};

use super::load_raw_config_with_source;
use crate::config::model::LibConfig;

/// `lib` config section plus the config file it came from.
#[derive(Debug, Clone, Default)]
pub struct LoadedLibConfig {
    pub config: LibConfig,
    pub source_path: Option<PathBuf>,
}

/// Load the `lib` section used by `vize lib`.
pub fn load_lib_config_with_source(path: Option<&Path>) -> LoadedLibConfig {
    let loaded = load_raw_config_with_source(path);
    LoadedLibConfig {
        config: loaded.config.lib,
        source_path: loaded.source_path,
    }
}

#[cfg(test)]
mod tests {
    use super::load_lib_config_with_source;

    #[test]
    fn reads_lib_section_from_json_config() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("vize.config.json"),
            r#"{ "lib": { "dir": "src/vendor", "composableDir": "src/use" } }"#,
        )
        .unwrap();

        let loaded = load_lib_config_with_source(Some(directory.path()));

        assert_eq!(loaded.config.dir.as_deref(), Some("src/vendor"));
        assert_eq!(loaded.config.dir_for_kind("composable"), Some("src/use"));
        assert!(loaded.source_path.is_some());
    }

    #[test]
    fn missing_config_yields_defaults() {
        let directory = tempfile::tempdir().unwrap();

        let loaded = load_lib_config_with_source(Some(directory.path()));

        assert_eq!(loaded.config, Default::default());
    }
}
