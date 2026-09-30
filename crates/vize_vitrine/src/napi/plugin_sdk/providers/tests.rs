//! Provider closure, table schema, deterministic audit and native batch proofs.
use super::super::batch::{PluginSpec, diagnostics};
use super::super::document::PluginDocument;
use super::super::facts::REGISTRY;
use super::super::plugin_cache::{PluginCacheInput, content_key};
use super::{ProviderCatalog, ProviderOutput, ProviderSpec, custom_batch};
use serde_json::{Value, json};
use vize_l0::fact::FactManager;

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|name| (*name).to_owned()).collect()
}

fn spec<'a>(name: &'a str, demands: &'a [String]) -> PluginSpec<'a> {
    PluginSpec {
        name,
        version: "1",
        fingerprint: "code",
        visit: None,
        demands,
    }
}

fn provider<'a>(name: &'a str, provides: &'a [String], demands: &'a [String]) -> ProviderSpec<'a> {
    ProviderSpec {
        plugin: spec(name, demands),
        provides,
        inputs_declared: true,
        inputs: &[],
    }
}

#[test]
fn computes_only_the_demanded_transitive_closure_in_dependency_order() {
    let (tokens, usage, unused) = (
        names(&["tokens/colors"]),
        names(&["usage/colors"]),
        names(&["unused/data"]),
    );
    let dependencies = names(&["tokens/colors", "templateScopes"]);
    let catalog = ProviderCatalog::new(&[
        provider("usage", &usage, &dependencies),
        provider("tokens", &tokens, &[]),
        provider("unused", &unused, &[]),
    ])
    .unwrap();
    assert_eq!(catalog.demanded(&spec("consumer", &usage)).unwrap(), [1, 0]);
    assert_eq!(
        catalog.demanded(&spec("consumer", &[])).unwrap(),
        Vec::<usize>::new()
    );
}

#[test]
fn rejects_missing_dependencies_and_all_cycles_before_execution() {
    let (first, second) = (names(&["first/data"]), names(&["second/data"]));
    assert!(ProviderCatalog::new(&[provider("first", &first, &second)]).is_err());
    assert!(
        ProviderCatalog::new(&[
            provider("first", &first, &second),
            provider("second", &second, &first),
        ])
        .is_err()
    );
    assert!(ProviderCatalog::new(&[provider("first", &first, &first)]).is_err());
}

#[test]
fn prevents_primary_overwrite_foreign_namespaces_and_duplicate_declarations() {
    for provides in [
        names(&["templateScopes"]),
        names(&["other/data"]),
        names(&["first/"]),
        names(&["first/data", "first/data"]),
    ] {
        assert!(ProviderCatalog::new(&[provider("first", &provides, &[])]).is_err());
    }
    let provides = names(&["first/data"]);
    assert!(
        ProviderCatalog::new(&[
            provider("first", &provides, &[]),
            provider("first", &provides, &[])
        ])
        .is_err()
    );
    assert!(
        ProviderCatalog::new(&[provider("@vize/first", &names(&["@vize/first/data"]), &[])])
            .is_err()
    );
    assert!(ProviderCatalog::new(&[provider("@vize", &names(&["@vize/data"]), &[])]).is_err());
}

#[test]
fn requires_explicit_ambient_inputs_and_reserves_host_inputs() {
    let groups = names(&["first/data"]);
    let mut first = provider("first", &groups, &[]);
    first.inputs_declared = false;
    assert!(ProviderCatalog::new(&[first]).is_err());
    let inputs = [PluginCacheInput {
        name: "@vize/fact:other/data",
        value: "spoof",
    }];
    let mut first = provider("first", &groups, &[]);
    first.inputs = &inputs;
    assert!(ProviderCatalog::new(&[first]).is_err());
}

#[test]
fn canonical_audit_ignores_object_order_but_rejects_different_values() {
    let groups = names(&["tokens/colors"]);
    let first = r##"{"tokens/colors":[[0,{"name":"red","value":"#f00"}]]}"##;
    let second = r##"{"tokens/colors":[[0,{"value":"#f00","name":"red"}]]}"##;
    let output = ProviderOutput::audit("tokens", &groups, first, second).unwrap();
    assert_eq!(
        output.values["tokens/colors"],
        json!([[0, {"name": "red", "value": "#f00"}]])
    );
    assert!(
        ProviderOutput::audit(
            "tokens",
            &groups,
            first,
            r##"{"tokens/colors":[[0,{"name":"blue","value":"#00f"}]]}"##
        )
        .is_err()
    );
}

#[test]
fn refuses_undeclared_missing_malformed_and_duplicate_table_keys() {
    let groups = names(&["tokens/colors"]);
    for output in [
        "{}",
        r#"{"tokens/colors":[],"templateScopes":[]}"#,
        r#"{"tokens/colors":{}}"#,
        r#"{"tokens/colors":[[0,"red"],[0,"blue"]]}"#,
        r#"{"tokens/colors":[[{},"red"]]}"#,
        r#"{"tokens/colors":[[-1,"red"]]}"#,
        r#"{"tokens/colors":[[0,"red","extra"]]}"#,
    ] {
        assert!(
            ProviderOutput::audit("tokens", &groups, output, output).is_err(),
            "{output}"
        );
    }
}

#[test]
fn native_batches_expose_only_the_consumers_declared_tables_and_reports_use_host_spans() {
    let source = "<template><button class=\"danger\">Go</button></template>";
    let document = PluginDocument::build(source, "Button.vue").unwrap();
    let mut manager = FactManager::new(&REGISTRY);
    let custom = json!({"tokens/colors": [[0, "red"]], "tokens/private": [[0, "secret"]]})
        .as_object()
        .unwrap()
        .clone();
    let demands = names(&["tokens/colors"]);
    let built = custom_batch(
        &document,
        &spec("consumer", &demands),
        &mut manager,
        &custom,
    )
    .unwrap();
    let batch: Value = serde_json::from_str(&built.json).unwrap();
    assert_eq!(batch["facts"], json!({"tokens/colors": [[0, "red"]]}));
    let found = diagnostics(
        &document,
        "consumer",
        r#"[{"rule":"no-danger","node":0,"message":"red token"}]"#,
    )
    .unwrap();
    let [diagnostic] = found.as_slice() else {
        panic!("expected one diagnostic");
    };
    assert_eq!(
        (
            &diagnostic.rule_id,
            &diagnostic.message,
            diagnostic.start,
            diagnostic.end
        ),
        (
            &"consumer/no-danger".to_owned(),
            &"red token".to_owned(),
            10,
            44
        )
    );
    assert!(
        custom_batch(
            &document,
            &spec("consumer", &names(&["missing/data"])),
            &mut manager,
            &custom
        )
        .is_err()
    );
}

#[test]
fn consumer_keys_change_when_provider_result_stamps_change() {
    let demands = names(&["tokens/colors"]);
    let consumer = spec("consumer", &demands);
    let groups = names(&["tokens/colors"]);
    let red = r#"{"tokens/colors":[[0,"red"]]}"#;
    let blue = r#"{"tokens/colors":[[0,"blue"]]}"#;
    let first = ProviderOutput::audit("tokens", &groups, red, red).unwrap();
    let second = ProviderOutput::audit("tokens", &groups, blue, blue).unwrap();
    let keyed = |value: &str| {
        content_key(
            "source",
            "file.vue",
            &consumer,
            &[PluginCacheInput {
                name: "@vize/fact:tokens/colors",
                value,
            }],
        )
        .unwrap()
    };
    assert_ne!(keyed(&first.result_key), keyed(&second.result_key));
}
