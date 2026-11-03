//! `lib` config section used by `vize lib` source distribution commands.

use std::collections::BTreeMap;

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
    /// Third-party registries by namespace (`"@acme"`), pulled as `@acme/<item>`.
    #[serde(
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "null_as_empty"
    )]
    pub registries: BTreeMap<String, LibRegistryConfig>,
}

/// Pkl renders an unset mapping as `null`; treat it like an empty one.
fn null_as_empty<'de, D>(deserializer: D) -> Result<BTreeMap<String, LibRegistryConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<BTreeMap<String, LibRegistryConfig>>::deserialize(deserializer)
        .map(Option::unwrap_or_default)
}

/// Where a third-party registry lives.
///
/// `source` is a local path (relative to the config file) to a
/// `registry.json`, its directory, or a package directory; `npm:<package>` or
/// `npm:<package>@<range>` for an npm package that ships `registry/registry.json`;
/// or an `https://` URL of a `registry.json` whose files sit under `files/` next to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LibRegistryConfig {
    /// Shorthand: just the source.
    Source(String),
    /// Source plus a destination directory for this namespace.
    Detailed {
        source: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dir: Option<String>,
    },
}

impl LibRegistryConfig {
    pub fn source(&self) -> &str {
        match self {
            Self::Source(source) | Self::Detailed { source, .. } => source,
        }
    }

    pub fn dir(&self) -> Option<&str> {
        match self {
            Self::Source(_) => None,
            Self::Detailed { dir, .. } => dir.as_deref(),
        }
    }
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
    use super::{LibConfig, LibRegistryConfig};

    #[test]
    fn kind_specific_directory_wins_over_shared_directory() {
        let config: LibConfig =
            serde_json::from_str(r#"{ "dir": "src/vendor", "uiDir": "src/ui" }"#).unwrap();

        assert_eq!(config.dir_for_kind("ui"), Some("src/ui"));
        assert_eq!(config.dir_for_kind("composable"), Some("src/vendor"));
        assert_eq!(LibConfig::default().dir_for_kind("ui"), None);
    }

    #[test]
    fn registries_accept_shorthand_and_detailed_entries() {
        let config: LibConfig = serde_json::from_str(
            r#"{ "registries": {
                "@acme": "npm:@acme/ui",
                "@local": { "source": "./registry", "dir": "src/local" }
            } }"#,
        )
        .unwrap();

        let acme = &config.registries["@acme"];
        assert_eq!(acme, &LibRegistryConfig::Source("npm:@acme/ui".into()));
        assert_eq!(acme.dir(), None);
        let local = &config.registries["@local"];
        let null: LibConfig = serde_json::from_str(r#"{ "registries": null }"#).unwrap();
        assert!(null.registries.is_empty());
        assert_eq!(local.source(), "./registry");
        assert_eq!(local.dir(), Some("src/local"));
    }
}
