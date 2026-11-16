use super::super::batch::{PluginSpec, build_batch};
use super::super::facts::REGISTRY;
use super::*;
const SOURCE: &str = r#"<script setup lang="ts">
import ChildAlias from './Child.vue'
import { ref, readonly, provide, inject, watchEffect } from 'vue'
const count = ref(0)
const locked = readonly({})
provide('count', count)
const injected = inject('count', 0)
watchEffect(async () => { await fetch('/api'); count.value = 1 })
</script><template><ChildAlias :label="count" @save="injected"><template #default="{ item }">{{ item }} {{ missing }}</template></ChildAlias></template>"#;
fn names() -> Vec<String> {
    GROUPS.iter().map(|name| (*name).to_owned()).collect()
}
#[test]
fn registered_production_groups_are_demanded_and_projected_without_inference() {
    let document = PluginDocument::build(SOURCE, "Child.vue").unwrap();
    let found = project(&document, &names()).unwrap();
    assert_eq!(found.len(), GROUPS.len());
    let bindings = found.get("bindings").unwrap().as_array().unwrap();
    assert!(
        bindings
            .iter()
            .any(|row| row.get(0).and_then(Value::as_str) == Some("count"))
    );
    let undefined = found.get("undefined-refs").unwrap().as_array().unwrap();
    assert!(undefined.iter().any(|row| {
        row.get(1)
            .and_then(|value| value.get("name"))
            .and_then(Value::as_str)
            == Some("missing")
    }));
    let components = found.get("component-usages").unwrap().as_array().unwrap();
    assert!(components.iter().any(|row| {
        row.get(1)
            .and_then(|value| value.get("module"))
            .and_then(Value::as_str)
            == Some("./Child.vue")
    }));
    let reactivity = found.get("reactivity").unwrap().as_array().unwrap();
    assert!(reactivity.iter().any(|row| {
        row.get(1)
            .and_then(|value| value.get("verdict"))
            .and_then(Value::as_str)
            == Some("unknown")
    }));
    assert!(
        !found
            .get("provide-inject")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        !found
            .get("race-conditions")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(project(&document, &names()).unwrap(), found);
}
#[test]
fn batches_only_carry_declared_production_groups() {
    let document = PluginDocument::build(SOURCE, "Child.vue").unwrap();
    let demands = vec!["bindings".to_owned()];
    let spec = PluginSpec {
        name: "facts",
        version: "1",
        fingerprint: "code",
        visit: None,
        demands: &demands,
    };
    let mut manager = FactManager::new(&REGISTRY);
    let batch: Value =
        serde_json::from_str(&build_batch(&document, &spec, &mut manager).unwrap().json).unwrap();
    assert_eq!(
        batch
            .get("facts")
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        vec!["bindings"]
    );
}
#[test]
fn unsupported_foreign_script_fact_requests_refuse() {
    let document = PluginDocument::build(
        "<script setup lang=\"moonbit\">let x = 1</script><template>{{ x }}</template>",
        "Foreign.vue",
    )
    .unwrap();
    let found = project(&document, &["bindings".to_owned()]);
    assert!(found.is_err());
    assert!(project(&document, &[]).unwrap().is_empty());
}

#[test]
fn positions_clamp_mid_character_and_past_end_offsets() {
    let document = PluginDocument {
        filename: "Offsets.vue".to_owned(),
        source: "a\né🦀".to_owned(),
        production: ProductionDocument::default(),
        nodes: Vec::new(),
        scopes: Vec::new(),
    };

    assert_eq!(document.position(3), (2, 1)); // Inside the two-byte é.
    assert_eq!(document.position(4), (2, 2)); // After é.
    assert_eq!(document.position(7), (2, 2)); // Inside the four-byte crab.
    assert_eq!(document.position(8), (2, 4)); // After its two UTF-16 units.
    assert_eq!(document.position(u32::MAX), (2, 4));
}

#[test]
fn primary_fact_demands_are_computed_once_per_document() {
    let document = PluginDocument::build(SOURCE, "Child.vue").unwrap();
    let demand = ["bindings".to_owned()];
    let first = project(&document, &demand).unwrap();
    assert_eq!(
        document.production.manager.lock().unwrap().computed(),
        Demand::NONE.with(Bindings::ID)
    );
    assert_eq!(project(&document, &demand).unwrap(), first);
    assert_eq!(
        document.production.manager.lock().unwrap().computed(),
        Demand::NONE.with(Bindings::ID)
    );
    project(&document, &["reactivity".to_owned()]).unwrap();
    assert_eq!(
        document.production.manager.lock().unwrap().computed(),
        Demand::NONE.with(Bindings::ID).with(Reactivity::ID)
    );
}

#[test]
fn authored_foreign_and_external_templates_cannot_be_lowered_as_html() {
    for source in [
        "<template lang=\"pug\">button {{ value }}</template>",
        "<template src=\"./external.html\" />",
    ] {
        assert_eq!(
            PluginDocument::build(source, "Foreign.vue").unwrap_err(),
            HostError::Split(
                "JS plugin visits require an inline HTML template; preprocess template dialects through their compiler integration".into()
            )
        );
    }
    assert!(
        PluginDocument::build(
            "<template lang=\"html\"><button /></template>",
            "Inline.vue"
        )
        .is_ok()
    );
}

#[test]
fn unused_binding_demand_computes_authoritative_dependency_and_exact_span() {
    let source =
        "<script setup>const used = 1; const unused = 2;</script><template>{{ used }}</template>";
    let document = PluginDocument::build(source, "Unused.vue").unwrap();
    let found = project(&document, &["unused-bindings".into()]).unwrap();
    let start = source.find("unused").unwrap();
    assert_eq!(
        found,
        Map::from_iter([(
            "unused-bindings".into(),
            json!([["unused", {"span": [start, start + 6]}]])
        )])
    );
    assert_eq!(
        document.production.manager.lock().unwrap().computed(),
        Demand::NONE.with(Bindings::ID).with(UnusedBindings::ID)
    );
    assert_eq!(
        project(&document, &["unused-bindings".into()]).unwrap(),
        found
    );
}

#[test]
fn binding_spans_map_reordered_split_scripts_without_changing_primary_facts() {
    let source = "<template>日本語😀{{ used }}</template>\n<script setup>const used = 1; const unused = 2</script>\n<style>.button { color: red }</style>\n<script lang=\"ts\">export const outside = 3</script>";
    let document = PluginDocument::build(source, "Split.vue").unwrap();
    let found = project(&document, &["bindings".into(), "unused-bindings".into()]).unwrap();
    let bindings = found.get("bindings").unwrap().as_array().unwrap();
    for name in ["used", "unused", "outside"] {
        let start = source[source.find("<script").unwrap()..]
            .find(name)
            .unwrap()
            + source.find("<script").unwrap();
        let row = bindings
            .iter()
            .find(|row| row.get(0) == Some(&json!(name)))
            .unwrap();
        assert_eq!(
            row.get(1).unwrap().get("span"),
            Some(&json!([start, start + name.len()]))
        );
    }
    let analysis = document.production.analysis(source).unwrap();
    let raw = analysis
        .context
        .croquis
        .binding_spans
        .get("unused")
        .unwrap();
    let authored = source.find("unused").unwrap() as u32;
    assert_ne!(raw.0, authored);
    assert_eq!(
        analysis
            .context
            .script_source_offset(&analysis.descriptor, raw.0),
        authored
    );
}
