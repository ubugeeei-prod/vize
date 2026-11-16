use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::{canonical_json, plugin_registry, verify_plugin_plan};

fn fixtures(kind: &str) -> Vec<(PathBuf, Value)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/plugin-plan")
        .join(kind);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            (path, value)
        })
        .collect()
}

#[test]
fn the_registry_parses_and_names_every_capability_consumer_as_a_product() {
    let registry = plugin_registry();
    assert_eq!(registry.plan_schema, 1);
    assert_eq!(registry.api_versions, [1, 2]);
    assert_eq!(registry.admitted_backends, ["js"]);
    let mut ids: Vec<&str> = registry.capabilities.keys().map(|id| id.as_str()).collect();
    ids.sort_unstable();
    assert_eq!(
        ids,
        [
            "lint.plugin-v1",
            "lint.rule",
            "sfc.custom-block",
            "vue.directive"
        ]
    );
    let unknown: Vec<&str> = registry
        .capabilities
        .values()
        .flat_map(|capability| capability.consumers.iter().chain(&capability.admitted_by))
        .filter(|product| !registry.products.contains(product))
        .map(|product| product.as_str())
        .collect();
    assert_eq!(unknown, Vec::<&str>::new());
}

/// Every plan the JS resolver produced verifies natively with the same digest.
#[test]
fn shared_resolve_fixtures_verify_with_the_js_digest() {
    let mut verified = 0;
    for (path, fixture) in fixtures("resolve") {
        let expect = &fixture["expect"];
        if expect.get("error").is_some() {
            continue;
        }
        let digest = expect["digest"].as_str().unwrap();
        let plan = verify_plugin_plan(&expect["plan"], Some(digest))
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(plan.digest(), digest, "{}", path.display());
        assert_eq!(
            plan.plan().plugins.len(),
            expect["plan"]["plugins"].as_array().unwrap().len()
        );
        verified += 1;
    }
    assert!(verified >= 8, "{verified}");
}

#[test]
fn tampered_and_stale_plans_fail_closed_with_the_shared_codes() {
    let mut checked = 0;
    for (path, fixture) in fixtures("verify") {
        let digest = fixture["digest"].as_str();
        let result = verify_plugin_plan(&fixture["plan"], digest);
        match fixture["expect"]["code"].as_str() {
            None => {
                result.unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            }
            Some(code) => {
                let error = result.expect_err(path.to_str().unwrap());
                assert_eq!(error.code, code, "{}: {error}", path.display());
                assert_eq!(
                    crate::cstr!("{error}"),
                    crate::cstr!("[plugin/{code}] {}: {}", error.subject, error.message)
                );
            }
        }
        checked += 1;
    }
    assert!(checked >= 13, "{checked}");
}

#[test]
fn canonical_serialization_matches_json_stringify() {
    let value = json!({
        "z": [1, -2, 9007199254740991_u64],
        "a": "quote\" slash\\ tab\t nl\n cr\r bs\u{8} ff\u{c} nul\u{0} us\u{1f} del\u{7f} sep\u{2028} 🦀",
        "é": null,
        "B": true,
        "aa": {"y": false, "x": {}}
    });
    assert_eq!(
        canonical_json(&value).unwrap(),
        "{\"B\":true,\"a\":\"quote\\\" slash\\\\ tab\\t nl\\n cr\\r bs\\b ff\\f nul\\u0000 us\\u001f del\u{7f} sep\u{2028} 🦀\",\"aa\":{\"x\":{},\"y\":false},\"z\":[1,-2,9007199254740991],\"é\":null}"
    );
    assert!(canonical_json(&json!({"ratio": 0.5})).is_err());
    assert!(canonical_json(&json!(9007199254740992_u64)).is_err());
    assert!(canonical_json(&json!(-9007199254740992_i64)).is_err());
}

#[test]
fn admission_refuses_unimplemented_capabilities_per_product() {
    let (_, fixture) = fixtures("resolve")
        .into_iter()
        .find(|(path, _)| path.ends_with("declared-capabilities.json"))
        .unwrap();
    let plan = verify_plugin_plan(&fixture["expect"]["plan"], None).unwrap();
    assert!(!plan.is_empty());
    let refused = plan.refusals("compiler", None);
    assert_eq!(
        refused
            .iter()
            .map(|refusal| (refusal.capability.as_str(), refusal.key.as_str()))
            .collect::<Vec<_>>(),
        [
            ("sfc.custom-block", "route:json"),
            ("sfc.custom-block", "i18n"),
            ("vue.directive", "track")
        ]
    );
    let error = plan.admit("compiler", None).unwrap_err();
    assert_eq!(
        crate::cstr!("{error}"),
        "[plugin/capability-unadmitted] @acme/routes: compiler does not implement sfc.custom-block `route:json` yet (and 2 more)"
    );
    assert_eq!(
        plan.admit("bundler", None).unwrap_err().code,
        "product-unknown"
    );
    let empty =
        verify_plugin_plan(&json!({"claims": [], "planSchema": 1, "plugins": []}), None).unwrap();
    assert!(empty.is_empty());
    assert_eq!(
        empty.canonical_json(),
        r#"{"claims":[],"planSchema":1,"plugins":[]}"#
    );
    for product in ["compiler", "typecheck", "lint", "format", "lsp"] {
        empty.admit(product, Some("vapor")).unwrap();
    }
}
