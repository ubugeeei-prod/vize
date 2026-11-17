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
    let admitted = fixture("lint-rule-defaults.json");
    let projected = json!({ "digest": admitted["digest"], "plan": admitted["plan"] });
    let loaded = serialized(json!({ "__vizePluginPlan": projected })).unwrap();
    assert_eq!(loaded.document.plugin_plan(), Some(&projected));
    let expect = fixture("declared-capabilities.json");
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
            "[plugin/capability-unadmitted] @acme/routes: no Vize product implements sfc.custom-block `route:json` yet; run `vize plugin inspect` for the plan"
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

#[test]
fn the_editor_admits_plugin_lint_rules() {
    let dir = tempfile::tempdir().unwrap();
    let sdk = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../npm/plugin-sdk/v2.js")
        .canonicalize()
        .unwrap();
    let config = crate::cstr!(
        "import {{ pathToFileURL }} from \"node:url\";\n\
         const {{ defineVizePlugin }} = await import(pathToFileURL({}).href);\n\
         const rule = defineVizePlugin({{ apiVersion: 2, stability: \"experimental\", name: \"@acme/r\", version: \"1\", setup(api) {{ api.lint.rule({{ id: \"x\", meta: {{ messages: {{ m: \"m\" }} }}, visit: [\"ui.element\"], run() {{}} }}); }} }});\n\
         export default {{ vue: {{ plugins: [rule()] }} }};\n",
        serde_json::to_string(&sdk.to_string_lossy()).unwrap()
    );
    std::fs::write(dir.path().join("vite.config.mjs"), config.as_str()).unwrap();
    let snapshot = super::load_lsp_config_snapshot(Some(dir.path()));
    assert!(snapshot.valid, "{:?}", snapshot.error);
    assert!(snapshot.project.document.plugin_plan().is_some());
}
