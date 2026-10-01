//! Production SFCs use inline templates, as the Vite production build does.

use super::super::{Value, json, trace};
use super::models;

fn assert_modes(source: &str, mut extra: Value) -> Value {
    extra["production"] = json!(false);
    let development = trace(source, "vdom", extra.clone());
    let mut production = extra;
    production["production"] = json!(true);
    let dom = trace(source, "vdom", production.clone());
    assert_eq!(dom, development, "production DOM behavior changed");
    let vapor = trace(source, "vapor", production);
    assert_eq!(vapor, dom, "production Vapor behavior differs from DOM");
    vapor
}

#[test]
fn production_inline_setup_refs_computed_values_and_events_remain_live() {
    let source = include_str!("../fixtures/vapor-production/reactive-events.vue");
    let actual = assert_modes(
        source,
        json!({"steps": [{"click": "button"}, {"click": "button"}]}),
    );
    assert_eq!(
        actual[0]["tree"][0]["children"][0]["children"],
        json!(["0:0"])
    );
    assert_eq!(
        actual[2]["tree"][0]["children"][0]["children"],
        json!(["2:4"])
    );
    assert_eq!(actual[2]["events"], json!([1, 2]));
    let native_default = trace(
        source,
        "vapor",
        json!({
            "production": true, "separateTemplate": true,
            "steps": [{"click": "button"}, {"click": "button"}]
        }),
    );
    assert_eq!(
        native_default, actual,
        "production adapters must inline script setup"
    );
}

#[test]
fn production_keyed_components_slots_scope_and_remount_preserve_behavior() {
    let source = include_str!("../fixtures/vapor-production/keyed-slots.vue");
    let child = r#"<script setup>
defineProps({ label: String });
const emit = defineEmits(['example']);
</script><template><div><button @click="emit('example', label)">{{ label }}</button><slot :label="label"/></div></template>"#;
    let actual = assert_modes(
        source,
        json!({"childSource": child,
        "scopedSelectors": ["section"],
        "context": {"items": [{"id": "a", "label": "A"}, {"id": "b", "label": "B"}]},
        "steps": [
            {"click": "[data-id=b] button"},
            {"patch": {"label": "updated", "items": [{"id": "b", "label": "B2"}, {"id": "a", "label": "A2"}]}, "preserve": ["[data-id=a]", "[data-id=b]"]},
            {"click": "[data-id=b] button"},
            {"patch": {"ready": false}},
            {"patch": {"ready": true}},
            {"click": "[data-id=a] button"}
        ]}),
    );
    assert_eq!(actual[6]["events"], json!(["B", "B2", "A2"]));
}

#[test]
fn production_component_models_switch_keys_and_ignore_stale_listeners() {
    let source = models::source("v-model:[state.field].trim=\"state.value\"");
    let actual = assert_modes(
        &source,
        json!({"childSource": models::CHILD,
        "context": {"field": "first", "value": "initial", "other": "untouched"},
        "steps": [{"click": ".first"}, {"patch": {"field": "second", "value": "external"}, "preserve": [".first", ".second"]}, {"click": ".first"}, {"click": ".second"}]}),
    );
    assert_eq!(
        models::outputs(&actual),
        [
            "initial|untouched",
            "first|untouched",
            "external|untouched",
            "external|untouched",
            "second|untouched"
        ]
    );
}

#[test]
fn production_native_models_keep_live_values_and_checked_state() {
    let source = include_str!("../fixtures/vapor-production/native-models.vue");
    let actual = assert_modes(
        source,
        json!({
        "context": {"value": "initial", "enabled": "no"},
        "steps": [
            {"dispatch": "input", "selector": ".text", "value": " typed "},
            {"dispatch": "change", "selector": ".check", "checked": true},
            {"patch": {"value": "external", "enabled": "no"}, "preserve": [".text", ".check"]},
            {"dispatch": "change", "selector": ".check", "checked": true}
        ]}),
    );
    let outputs: Vec<_> = actual
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|snapshot| snapshot["tree"][0]["children"][2]["children"][0].as_str())
        .collect();
    assert_eq!(
        outputs,
        [
            "initial|no",
            "typed|no",
            "typed|yes",
            "external|no",
            "external|yes"
        ]
    );
}
