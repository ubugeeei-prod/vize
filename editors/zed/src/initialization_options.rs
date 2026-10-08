use super::serde_json;

pub(super) const WORKSPACE_CONFIG_FILES: [&str; 5] = [
    "vize.config.pkl",
    "vize.config.ts",
    "vize.config.js",
    "vize.config.mjs",
    "vize.config.json",
];

pub(super) fn initialization_options(
    explicit: Option<serde_json::Value>,
    readable_config: impl FnMut(&str) -> bool,
) -> serde_json::Value {
    if let Some(explicit) = explicit {
        return explicit;
    }
    if WORKSPACE_CONFIG_FILES.into_iter().any(readable_config) {
        serde_json::json!({})
    } else {
        recommended_initialization_options()
    }
}

pub(super) fn recommended_initialization_options() -> serde_json::Value {
    serde_json::json!({
        "editor": true,
        "ecosystem": true,
        "lint": true,
        "typecheck": true,
    })
}
