#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests assert by panicking; insta and fixtures use format!; fixtures use std strings"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcScriptOutputMode, TemplateCompileOptions, compile_sfc_for_adapter,
    parse_sfc,
};

use vapor_runtime_contracts::process_evidence;

mod vapor_runtime_contracts {
    mod events;
    mod model_arguments;
    mod models;
    pub(super) mod process_evidence;
    mod production;
    mod template_refs;
}

const CHILD: &str = r#"<script setup>
defineProps({ label: String });
const emit = defineEmits(['example']);
</script><template><button @click="emit('example', label)">{{ label }}</button></template>"#;

fn compile(source: &str, backend: &str) -> String {
    compile_for_mode(source, backend, false)
}

fn compile_for_mode(source: &str, backend: &str, production: bool) -> String {
    compile_with_output(
        source,
        backend,
        production,
        if production {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
    )
}

fn compile_with_output(
    source: &str,
    backend: &str,
    production: bool,
    script_output: SfcScriptOutputMode,
) -> String {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let options = SfcCompileOptions {
        vapor: backend == "vapor",
        scope_id: Some("probe".into()),
        template: TemplateCompileOptions {
            is_prod: production,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc_for_adapter(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        script_output,
    )
    .unwrap();
    assert!(result.errors.is_empty(), "{backend}: {:?}", result.errors);
    result.code.to_string()
}

fn trace(source: &str, backend: &str, extra: Value) -> Value {
    let production = extra
        .get("production")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| {
            std::env::var("VIZE_VUE_RUNTIME_PRODUCTION").is_ok_and(|value| value == "1")
        });
    let compile_request = |source| {
        if extra.get("separateTemplate").and_then(Value::as_bool) == Some(true) {
            compile_with_output(
                source,
                backend,
                production,
                SfcScriptOutputMode::SeparateTemplate,
            )
        } else {
            compile_for_mode(source, backend, production)
        }
    };
    let code = compile_request(source);
    let child_source = extra
        .get("childSource")
        .and_then(Value::as_str)
        .unwrap_or(CHILD);
    let mut input =
        json!({"backend": backend, "code": code, "child": compile_request(child_source)});
    input
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/vapor-sfc-runtime.mjs");
    let mut child = Command::new("node")
        .arg(&runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let child_pid = child.id();
    let input_bytes = input.to_string().into_bytes();
    let stdin_write = child.stdin.take().unwrap().write_all(&input_bytes);
    let output = child.wait_with_output().unwrap();
    let evidence = process_evidence::capture(
        backend,
        &runner,
        child_pid,
        (source, child_source),
        &input_bytes,
        &output,
        stdin_write.as_ref().err(),
    );
    if let Some(name) = extra.get("proofName").and_then(Value::as_str) {
        let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
            "full"
        } else {
            "pr"
        };
        let destination = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/nextest")
            .join(profile);
        std::fs::create_dir_all(&destination).unwrap();
        std::fs::write(
            destination.join(format!("vapor-refs-{name}.json")),
            serde_json::to_vec(&json!({
                "source": source, "childSource": child_source, "input": input,
                "exitCode": output.status.code(), "success": output.status.success(),
                "stdoutBytes": output.stdout, "stderrBytes": output.stderr,
                "stdinWriteError": stdin_write.as_ref().err().map(ToString::to_string)
            }))
            .unwrap(),
        )
        .unwrap();
    }
    assert!(
        stdin_write.is_ok(),
        "{backend} stdin: {stdin_write:?}\n{}",
        process_evidence::failure(backend, &input_bytes, &output, &evidence)
    );
    assert!(
        output.status.success(),
        "{}",
        process_evidence::failure(backend, &input_bytes, &output, &evidence)
    );
    assert!(evidence.is_ok(), "{backend} evidence: {evidence:?}");
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
