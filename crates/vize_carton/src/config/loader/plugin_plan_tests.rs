//! Projected plugin plans verify before any product reads a document (#13074).

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::{parse::document_from_json, try_load_project_config_from_serialized};

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/plugin-plan/resolve")
        .join(name);
    let fixture: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    fixture["expect"].clone()
}

fn serialized(vue: Value) -> Result<super::LoadedProjectConfig, std::string::String> {
    let json = serde_json::to_vec(&json!({ "vue": vue })).unwrap();
    try_load_project_config_from_serialized(&json, PathBuf::from("/p/vite.config.json"))
}

fn plan_error(error: &str) -> &str {
    error.split_once(": ").map_or(error, |(_, message)| message)
}

#[test]
fn documents_without_a_plan_are_unchanged() {
    let loaded = serialized(json!({ "linter": { "preset": "essential" } })).unwrap();
    assert_eq!(loaded.document.plugin_plan(), None);
    let document =
        document_from_json(br#"{"linter":{"preset":"essential"}}"#, Path::new("a.json")).unwrap();
    assert_eq!(document.plugin_plan(), None);
}

#[test]
fn projected_plans_verify_and_unimplemented_capabilities_fail_closed() {
    let expect = fixture("lint-rule-defaults.json");
    let projected = json!({ "digest": expect["digest"], "plan": expect["plan"] });
    for result in [
        serialized(json!({ "__vizePluginPlan": projected })).map(|_| ()),
        document_from_json(
            &serde_json::to_vec(&json!({ "__vizePluginPlan": projected })).unwrap(),
            Path::new("a.json"),
        )
        .map(|_| ())
        .map_err(|error| crate::cstr!("x: {error}").into_string()),
    ] {
        assert_eq!(
            plan_error(&result.unwrap_err()),
            "[plugin/capability-unadmitted] @acme/design-system: no Vize product implements lint.rule `@acme/design-system/no-raw-color` yet; run `vize plugin inspect` for the plan"
        );
    }
}

#[test]
fn stale_malformed_and_unprojected_plans_fail_closed() {
    let expect = fixture("lint-rule-defaults.json");
    let stale = json!({ "digest": "0".repeat(64), "plan": expect["plan"] });
    let empty = fixture("empty.json");
    let cases = [
        (json!({ "__vizePluginPlan": stale }), "plan-stale"),
        (
            json!({ "__vizePluginPlan": { "plan": expect["plan"] } }),
            "plan-invalid",
        ),
        (
            json!({ "__vizePluginPlan": { "digest": empty["digest"], "plan": empty["plan"] } }),
            "plan-invalid",
        ),
        (
            json!({ "plugins": [{ "name": "x", "version": "1" }] }),
            "entry-invalid",
        ),
    ];
    for (vue, code) in cases {
        let error = serialized(vue).unwrap_err();
        let message = plan_error(&error);
        assert_eq!(
            message.split_once(']').map(|(code, _)| code),
            Some(crate::cstr!("[plugin/{code}").as_str()),
            "{error}"
        );
    }
}
