use super::super::{
    InspectorOptions, InspectorSourceFile, InspectorTarget, InspectorTemplateSyntax, build_payload,
    build_playground_url, serialize_payload,
};
use vize_carton::cstr;

#[test]
fn builds_inspector_payload_json_and_url() {
    let payload = build_payload(
        InspectorTarget::Dom,
        InspectorOptions {
            custom_renderer: false,
            template_syntax: InspectorTemplateSyntax::Quirks,
        },
        vec![InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!("<template><div>msg</div></template>"),
        }],
    );
    let json = serialize_payload(&payload).expect("payload serializes");

    assert_eq!(
        json.as_str(),
        r#"{"version":1,"target":"dom","selectedFile":"src/App.vue","options":{"customRenderer":false,"templateSyntax":"quirks"},"files":[{"path":"src/App.vue","source":"<template><div>msg</div></template>"}]}"#
    );
    assert_eq!(
        build_playground_url("https://vizejs.dev/play/?foo=bar#old", json.as_str()).as_str(),
        "https://vizejs.dev/play/?foo=bar&tab=inspector#inspector=%7B%22version%22%3A1%2C%22target%22%3A%22dom%22%2C%22selectedFile%22%3A%22src%2FApp.vue%22%2C%22options%22%3A%7B%22customRenderer%22%3Afalse%2C%22templateSyntax%22%3A%22quirks%22%7D%2C%22files%22%3A%5B%7B%22path%22%3A%22src%2FApp.vue%22%2C%22source%22%3A%22%3Ctemplate%3E%3Cdiv%3Emsg%3C%2Fdiv%3E%3C%2Ftemplate%3E%22%7D%5D%7D"
    );
}

#[test]
fn builds_vapor_inspector_payload_json() {
    let payload = build_payload(
        InspectorTarget::Vapor,
        InspectorOptions {
            custom_renderer: false,
            template_syntax: InspectorTemplateSyntax::Standard,
        },
        vec![InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!("<template><div>msg</div></template>"),
        }],
    );
    let json = serialize_payload(&payload).expect("payload serializes");

    assert!(json.contains(r#""target":"vapor""#));
}
