use super::super::{
    InspectorOptions, InspectorSourceFile, InspectorTarget, InspectorTemplateSyntax,
    build_agent_report, build_payload, build_playground_url, serialize_agent_report,
    serialize_payload,
};
use vize_carton::{String, cstr};

#[test]
fn builds_agent_report_with_payload_url_and_graph() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!("<script setup>import Child from './Child'</script>\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/Child.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
    ];
    let payload = build_payload(
        InspectorTarget::Ssr,
        InspectorOptions {
            custom_renderer: true,
            template_syntax: InspectorTemplateSyntax::Standard,
        },
        files.clone(),
    );
    let json = serialize_payload(&payload).expect("payload serializes");
    let url = build_playground_url("https://vizejs.dev/play/", json.as_str());
    let report = build_agent_report(payload, url, files);
    let report_json = serialize_agent_report(&report).expect("report serializes");

    assert!(report_json.contains(r#""schema": "vize.inspector.agent""#));
    assert!(report_json.contains(r#""target": "ssr""#));
    assert!(report_json.contains(r#""to": "src/Child.vue""#));
}

#[test]
fn agent_report_includes_semantic_summary_counts() {
    let files = vec![InspectorSourceFile {
        path: cstr!("src/App.vue"),
        source: String::from(
            r#"<script setup>
const count = 0
provide('count', count)
</script>
<template>
  <button id="save" @click="count++">{{ count }}</button>
</template>"#,
        ),
    }];
    let payload = build_payload(
        InspectorTarget::Dom,
        InspectorOptions {
            custom_renderer: false,
            template_syntax: InspectorTemplateSyntax::Standard,
        },
        files.clone(),
    );
    let json = serialize_payload(&payload).expect("payload serializes");
    let url = build_playground_url("https://vizejs.dev/play/", json.as_str());
    let report = build_agent_report(payload, url, files);
    let report_json = serialize_agent_report(&report).expect("report serializes");

    assert!(report_json.contains(r#""semantic": {"#));
    assert!(report_json.contains(r#""semanticFiles": ["#));
    assert!(report_json.contains(r#""snapshot": {"#));
    assert!(report_json.contains(r#""provides": ["#));
    assert!(report_json.contains(r#""analyzedFiles": 1"#));
    assert!(report_json.contains(r#""provideCount": 1"#));
    assert!(report_json.contains(r#""elementIdCount": 1"#));
}
