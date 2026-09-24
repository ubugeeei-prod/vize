//! `lib` config section used by `vize lib` source distribution commands.

use serde::{Deserialize, Serialize};

use crate::String;

/// Where `vize lib pull` writes source-owned units.
///
/// Every path is relative to the directory that holds the config file. Pulled
/// files keep the registry's relative layout below the chosen directory, so
/// relative imports between pulled items keep resolving without rewrites.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LibConfig {
    /// Destination for every registry kind unless a kind-specific directory is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    /// Destination for `@vizejs/ui` items. Default: `src/components/vize`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_dir: Option<String>,
    /// Destination for `@vizejs/composable` items. Default: `src/composables/vize`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composable_dir: Option<String>,
    /// Lockfile path. Default: `vize-lib.lock.json`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile: Option<String>,
}

impl LibConfig {
    /// Configured destination for a registry kind (`"ui"` or `"composable"`).
    pub fn dir_for_kind(&self, kind: &str) -> Option<&str> {
        let specific = match kind {
            "ui" => self.ui_dir.as_deref(),
            "composable" => self.composable_dir.as_deref(),
            _ => None,
        };
        specific.or(self.dir.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::LibConfig;

    #[test]
    fn kind_specific_directory_wins_over_shared_directory() {
        let config: LibConfig =
            serde_json::from_str(r#"{ "dir": "src/vendor", "uiDir": "src/ui" }"#).unwrap();

        assert_eq!(config.dir_for_kind("ui"), Some("src/ui"));
        assert_eq!(config.dir_for_kind("composable"), Some("src/vendor"));
        assert_eq!(LibConfig::default().dir_for_kind("ui"), None);
    }
}
