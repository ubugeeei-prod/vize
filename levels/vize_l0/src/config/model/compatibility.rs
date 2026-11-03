use super::vue;

/// Legacy top-level compatibility aliases accepted by older config examples.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RawCompatibilityConfig {
    pub(crate) vue_version: Option<vue::VueVersion>,
}
