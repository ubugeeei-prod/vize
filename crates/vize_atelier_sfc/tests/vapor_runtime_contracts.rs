#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};

mod vapor_runtime_contracts {
    mod events;
    mod models;
}

const CHILD: &str = r#"<script setup>
defineProps({ label: String });
const emit = defineEmits(['example']);
</script><template><button @click="emit('example', label)">{{ label }}</button></template>"#;

fn compile(source: &str, backend: &str) -> String {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let options = SfcCompileOptions {
        vapor: backend == "vapor",
        scope_id: Some("probe".into()),
        ..Default::default()
    };
    let result = compile_sfc_for_adapter(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .unwrap();
    assert!(result.errors.is_empty(), "{backend}: {:?}", result.errors);
    result.code.to_string()
}

fn trace(source: &str, backend: &str, extra: Value) -> Value {
    let code = compile(source, backend);
    let child_source = extra
        .get("childSource")
        .and_then(Value::as_str)
        .unwrap_or(CHILD);
    let mut input =
        json!({"backend": backend, "code": code, "child": compile(child_source, backend)});
    input
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/vapor-sfc-runtime.mjs");
    let mut child = Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{backend}: {}\n{code}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn scoped_sfc_marks_nested_native_elements() {
    let source = r#"<script setup>import { state } from './fixture';</script>
<template><details><summary>{{ state.label }}</summary><fieldset><label><input />Enabled</label></fieldset></details></template>
<style scoped>fieldset { border: 0 }</style>"#;
    let extra = json!({"scopedSelectors": ["details", "summary", "fieldset", "label", "input"],
        "steps": [{"patch": {"label": "updated"}}]});
    let dom = trace(source, "vdom", extra.clone());
    assert_eq!(trace(source, "vapor", extra), dom);
}

#[test]
fn component_sfc_insertion_preserves_authored_sibling_order() {
    for template in [
        "<div><Child :label=\"state.label\"/><p>After</p></div>",
        "<div><p>Before</p><Child :label=\"state.label\"/><p>After</p></div>",
        "<div><Child :label=\"state.label\"/><Child label=\"second\"/><p>After</p></div>",
        "<div>Before<Child :label=\"state.label\"/>After</div>",
        "<div>{{ state.label }}<Child :label=\"state.label\"/>{{ state.label }}</div>",
        "<div><component :is=\"Child\" :label=\"state.label\"/><p>After</p></div>",
    ] {
        let source = format!(
            "<script setup>import Child from './Child.vue'; import {{ state }} from './fixture';</script><template>{template}</template>"
        );
        let extra = json!({"steps": [{"patch": {"label": "updated"}}]});
        let dom = trace(&source, "vdom", extra.clone());
        assert_eq!(trace(&source, "vapor", extra), dom, "{template}");
    }
}

#[test]
fn scoped_sfc_preserves_namespaces_text_and_dynamic_descendants() {
    for template in [
        r#"<section><template v-if="state.ready"><fieldset><label><input/>{{ state.label }}</label></fieldset></template><span v-else>empty</span></section>"#,
        r#"<section><span v-for="(item, index) in state.label" :key="index">{{ item }}</span><p>after</p></section>"#,
        r#"<section><svg><g><circle r="2"/></g></svg><textarea>&lt;span&gt;</textarea><p title="&lt;label&gt;">&lt;em&gt;</p></section>"#,
        r#"<section><article v-html="state.label"/><input/></section>"#,
    ] {
        let source = format!(
            "<script setup>import {{ state }} from './fixture';</script><template>{template}</template><style scoped>section span {{ color: red }}</style>"
        );
        let extra = json!({"scopedSelectors": ["section"], "steps": [
            {"patch": {"ready": false, "label": "<b>html</b>"}},
            {"patch": {"ready": true, "label": "again"}}
        ]});
        let dom = trace(&source, "vdom", extra.clone());
        assert_eq!(trace(&source, "vapor", extra), dom, "{template}");
    }
}

#[test]
fn keyed_and_conditional_components_keep_order_identity_and_events() {
    let source = r#"<script setup>import Child from './Child.vue'; import { state, record } from './fixture';</script>
<template><section><p>before</p><Child v-if="state.ready" label="conditional" @example="record($event)"/><i>middle</i><Child v-for="item in state.items" :key="item.id" :data-id="item.id" :label="item.label" @example="record($event)"/><p>after</p></section></template>"#;
    let extra = json!({"context": {"items": [{"id":"a", "label":"A"}, {"id":"b", "label":"B"}]},
    "steps": [
        {"click": "[data-id=a]"},
        {"patch": {"ready": false, "items": [{"id":"b", "label":"B2"}, {"id":"a", "label":"A2"}]}, "preserve": ["[data-id=a]", "[data-id=b]"]},
        {"click": "[data-id=a]"},
        {"patch": {"ready": true, "items": [{"id":"b", "label":"B3"}]}, "preserve": ["[data-id=b]"]},
        {"click": "button"}
    ]});
    let dom = trace(source, "vdom", extra.clone());
    assert_eq!(dom[5]["events"], json!(["A", "A2", "conditional"]));
    assert_eq!(trace(source, "vapor", extra), dom);
}

#[test]
fn supplied_slots_keep_insertion_order_across_conditional_remounts() {
    let source = r#"<script setup>import Child from './Child.vue'; import { state } from './fixture';</script>
<template><section><Child v-if="state.ready"><b>{{ state.label }}</b><i>slot-tail</i></Child><p>after</p></section></template>"#;
    let extra = json!({"childSource": "<script setup>defineOptions({ name: 'SlotHost' })</script><template><div><span>child-before</span><slot/><span>child-after</span></div></template>",
        "steps": [{"patch": {"label": "updated"}}, {"patch": {"ready": false}}, {"patch": {"ready": true}}]});
    let dom = trace(source, "vdom", extra.clone());
    assert_eq!(trace(source, "vapor", extra), dom);
}

#[test]
fn component_sfc_inline_handlers_are_lazy_and_receive_emitted_values() {
    for handler in [
        "record($event)",
        "record?.($event)",
        "record",
        "handlers['x;y']",
        "value => record(value)",
        "function (value) { record(value) }",
        "state.ready && record($event)",
        "record($event); record('second')",
    ] {
        let source = format!(
            "<script setup>import Child from './Child.vue'; import {{ state, record, handlers }} from './fixture';</script><template><Child :label=\"state.label\" @example=\"{handler}\"/></template>"
        );
        let extra = json!({"steps": [{"click": "button"}, {"patch": {"label": "updated"}}, {"click": "button"}]});
        let dom = trace(&source, "vdom", extra.clone());
        assert_eq!(dom[0]["events"], json!([]));
        let expected = if handler == "record($event); record('second')" {
            json!(["initial", "second", "updated", "second"])
        } else {
            json!(["initial", "updated"])
        };
        assert_eq!(dom[3]["events"], expected);
        assert_eq!(trace(&source, "vapor", extra), dom, "{handler}");
    }
}

#[test]
fn bound_event_factories_keep_prop_evaluation_semantics() {
    let source = r#"<script setup>import Child from './Child.vue'; import { state, makeHandler } from './fixture';</script>
<template><Child :label="state.label" :onExample="makeHandler()" :only="state.label.toUpperCase()" /></template>"#;
    let extra = json!({"steps": [{"click": "button"}, {"patch": {"label": "updated"}}, {"click": "button"}]});
    let dom = trace(source, "vdom", extra.clone());
    assert_eq!(dom[3]["events"], json!(["initial", "updated"]));
    assert_eq!(trace(source, "vapor", extra), dom);
}
