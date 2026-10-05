use super::serde_json;

pub(super) fn recommended_initialization_options() -> serde_json::Value {
    serde_json::json!({
        "editor": true,
        "ecosystem": true,
        "lint": true,
        "typecheck": true,
    })
}
