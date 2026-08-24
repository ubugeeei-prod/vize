//! The inspector payload's Spolvero feed (P2-18): the `spolvero` member of
//! every `build_payload` result validates against the committed schema
//! (`davinci-road/plan/spolvero-feed.schema.json`) through the shared strict
//! validator (TS-15), and its content is pinned exactly.

use std::path::Path;

use davinci_test_support::schema as schema_check;
use vize_carton::cstr;
use vize_curator::inspector::{
    InspectorOptions, InspectorSourceFile, InspectorTarget, InspectorTemplateSyntax, build_payload,
    serialize_payload,
};

/// Load the committed schema relative to this crate's manifest.
fn load_schema() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("davinci-road")
        .join("plan")
        .join("spolvero-feed.schema.json");
    let text = std::fs::read_to_string(&path).expect("committed schema reads");
    serde_json::from_str(&text).expect("committed schema is valid JSON")
}

fn payload_json(files: Vec<InspectorSourceFile>) -> serde_json::Value {
    let payload = build_payload(
        InspectorTarget::Dom,
        InspectorOptions {
            custom_renderer: false,
            template_syntax: InspectorTemplateSyntax::Standard,
        },
        files,
    );
    let json = serialize_payload(&payload).expect("payload serializes");
    serde_json::from_str(json.as_str()).expect("payload is valid JSON")
}

#[test]
fn the_payload_feed_validates_and_carries_the_s1_page_exactly() {
    // The S1 page's text must equal the authored template bytes - the TS-19
    // byte-fidelity law observed at the consumer - proven through the
    // surface tree rather than copied from the source.
    let template = "\n  <div :class=\"cls\">{{ msg }}</div>\n  <br>\n";
    let json = payload_json(vec![InspectorSourceFile {
        path: cstr!("src/App.vue"),
        source: cstr!("<script setup>const msg = 'hi'</script>\n<template>{template}</template>\n"),
    }]);

    let spolvero = &json["spolvero"];
    assert_eq!(
        schema_check::validate(&load_schema(), spolvero, "$"),
        Ok(())
    );
    assert_eq!(
        *spolvero,
        serde_json::json!({
            "schema_version": 1,
            "command": "inspector",
            "pages": [
                { "path": "src/App.vue", "stage": "s1", "pass": "parse", "text": template },
            ],
        })
    );
}

#[test]
fn files_without_a_renderable_template_contribute_no_page() {
    // A non-.vue file, a template-less SFC, and an SFC parse failure all
    // stay out of the feed: it is a stage-dump channel, not a diagnostics
    // channel. The feed itself is still present and schema-valid.
    let json = payload_json(vec![
        InspectorSourceFile {
            path: cstr!("src/util.ts"),
            source: cstr!("export const n = 1;\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/ScriptOnly.vue"),
            source: cstr!("<script setup>const n = 1</script>\n"),
        },
    ]);

    let spolvero = &json["spolvero"];
    assert_eq!(
        schema_check::validate(&load_schema(), spolvero, "$"),
        Ok(())
    );
    assert_eq!(
        *spolvero,
        serde_json::json!({
            "schema_version": 1,
            "command": "inspector",
            "pages": [],
        })
    );
}

#[test]
fn a_malformed_template_still_feeds_its_bytes_faithfully() {
    // S1 is total over malformed input (typed holes, never dropped bytes),
    // so a broken template still produces its page and the text is still
    // the authored bytes.
    let template = "\n<div class=\"open>{{ msg }\n";
    let json = payload_json(vec![InspectorSourceFile {
        path: cstr!("src/Broken.vue"),
        source: cstr!("<template>{template}</template>\n"),
    }]);

    let spolvero = &json["spolvero"];
    assert_eq!(
        schema_check::validate(&load_schema(), spolvero, "$"),
        Ok(())
    );
    assert_eq!(
        spolvero["pages"],
        serde_json::json!([
            { "path": "src/Broken.vue", "stage": "s1", "pass": "parse", "text": template },
        ])
    );
}
