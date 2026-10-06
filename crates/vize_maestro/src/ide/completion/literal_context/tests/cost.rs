//! Observational classifier-only work in the ordinary source test suite.

use super::{ORIGINAL, at};
use crate::{ide::IdeContext, server::ServerState};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{hint::black_box, time::Instant};
use tower_lsp::lsp_types::Url;

#[test]
fn records_completion_local_classifier_work_without_native_or_hover_requests() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Page.vue").unwrap();
    state
        .documents
        .open(uri.clone(), ORIGINAL.into(), 1, "vue".into());
    state.update_virtual_docs(&uri, ORIGINAL);
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".into()))
        .join("string-literal-classification-cost")
        .join(if cfg!(feature = "native") {
            "native.json"
        } else {
            "non-native.json"
        });
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    let classifier = include_str!("../../literal_context.rs");
    let source_hash = Sha256::digest(ORIGINAL.as_bytes());
    let classifier_hash = Sha256::digest(classifier.as_bytes());
    let mut packet = json!({
        "schema": "vize.lsp.literal-classifier-work.v1",
        "complete": false,
        "sourceHead": std::env::var("GITHUB_SHA").ok(),
        "nativeFeature": cfg!(feature = "native"),
        "debugAssertions": cfg!(debug_assertions),
        "source": ORIGINAL,
        "sourceSha256Bytes": source_hash.as_slice(),
        "classifier": classifier,
        "classifierSha256Bytes": classifier_hash.as_slice(),
        "iterationsPerCursor": 16,
        "rows": [],
        "qualification": "Wall time around only the existing classifier call, after context preparation; noisy test-host observations, not CPU/allocation counts, a gain claim or a new performance ceiling. No provider or hover requests."
    });
    let persist = |packet: &serde_json::Value| {
        std::fs::write(&output, serde_json::to_vec_pretty(packet).unwrap()).unwrap();
    };
    persist(&packet);
    for (name, needle, expected) in [
        ("original-script-literal", "t(\"", true),
        ("original-template-literal", "<p>{{ t(\"", true),
        ("ordinary-template-identifier", "<h1>{{ ti", false),
    ] {
        let offset = at(ORIGINAL, needle);
        let ctx = IdeContext::at_completion(&state, &uri, offset).unwrap();
        let projection = ctx
            .virtual_docs
            .as_ref()
            .and_then(|docs| docs.template.as_ref())
            .map(|document| document.content.as_str());
        let mut observations = Vec::new();
        for _ in 0..16 {
            let started = Instant::now();
            let actual = black_box(super::super::contains_cursor(black_box(&ctx)));
            let elapsed_ns = started.elapsed().as_nanos();
            observations.push(json!({"actual": actual, "elapsedNs": elapsed_ns}));
        }
        packet
            .get_mut("rows")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(json!({
                "name": name,
                "offset": offset,
                "expected": expected,
                "wholeTemplateProjection": projection,
                "observations": observations,
            }));
        persist(&packet);
        assert!(
            observations
                .iter()
                .all(|row| row.get("actual") == Some(&json!(expected)))
        );
    }
    *packet.get_mut("complete").unwrap() = json!(true);
    persist(&packet);
}
