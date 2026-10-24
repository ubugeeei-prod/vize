use super::{initialization_options::initialization_options, zed::serde_json::json};

#[test]
fn default_options_always_defer_project_discovery_to_the_server() {
    assert_eq!(initialization_options(None), json!({}));
}

#[test]
fn explicit_options_are_preserved() {
    for explicit in [
        json!({}),
        json!({ "formatting": true }),
        json!({ "hover": false, "typecheck": false, "custom": [1, "x"] }),
        json!(null),
        json!({ "editor": true, "ecosystem": true, "lint": true, "typecheck": true }),
    ] {
        assert_eq!(initialization_options(Some(explicit.clone())), explicit);
    }
}
