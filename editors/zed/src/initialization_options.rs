use super::serde_json;

pub(super) fn initialization_options(explicit: Option<serde_json::Value>) -> serde_json::Value {
    // Project discovery belongs to the server, including monorepo package roots.
    explicit.unwrap_or_else(|| serde_json::json!({}))
}
