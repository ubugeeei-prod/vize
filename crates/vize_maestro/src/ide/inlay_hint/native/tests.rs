//! Complete structured hints and writable spans survive only exact projections.
#![expect(clippy::disallowed_macros, reason = "whole JSON test oracles")]

use super::*;
use crate::ide::corsa_support::canonical_source_offset_to_position;
use crate::server::ServerState;
use serde_json::{Value, json};
use tower_lsp::lsp_types::{Position, Url};

fn document(uri: &Url, source: &str) -> CanonicalVirtualDocument {
    CanonicalVirtualDocument {
        source_uri: uri.clone(),
        request_uri: format!("{}.ts", uri).into(),
        virtual_result: crate::ide::DiagnosticService::generate_virtual_ts(
            uri, source, false, false,
        )
        .unwrap(),
        dependencies: Vec::new(),
        materialized_sources: Vec::new(),
        session_project_roots: Vec::new(),
        source_catalogs: Vec::new(),
    }
}

fn point(doc: &CanonicalVirtualDocument, offset: usize) -> Value {
    let (line, character) = canonical_source_offset_to_position(doc, offset).unwrap();
    json!({"line":line,"character":character})
}

#[test]
fn complete_checker_label_parts_aliases_metadata_and_edits_are_preserved() {
    let source = "<script setup lang=\"ts\">\r\n/*😀*/ const café = computed(() => [{ id: 1 }]);\r\n</script>\r\n";
    let uri = Url::parse("file:///project/App.vue").unwrap();
    let state = ServerState::new();
    let ctx = IdeContext::testing(&state, &uri, 0, source.into());
    let doc = document(&uri, source);
    let start = source.find("café").unwrap();
    let end = start + "café".len();
    let range = json!({"start":point(&doc,start),"end":point(&doc,end)});
    let native = json!({
        "position":point(&doc,end),
        "label":[{"value":": "},{"value":"ComputedRef","tooltip":{"kind":"markdown","value":"actual alias"},"location":{"uri":doc.request_uri,"range":range},"command":{"title":"Open type","command":"editor.action.goToTypeDefinition","arguments":["unchanged"]}},{"value":"<{ id: number; }[]>"}],
        "kind":1,"paddingLeft":true,"paddingRight":false,
        "tooltip":{"kind":"plaintext","value":"checker type"},
        "textEdits":[{"range":range,"newText":"records"}],
        "data":{"generation":8,"native":{"id":"opaque"}}
    });
    let mapped = map_hint(
        &ctx,
        &doc,
        Range::new(Position::new(0, 0), Position::new(10, 0)),
        serde_json::from_value(native.clone()).unwrap(),
    )
    .unwrap();
    let authored = json!({"start":{"line":1,"character":13},"end":{"line":1,"character":17}});
    let mut expected = native;
    expected["position"] = authored["end"].clone();
    expected["textEdits"][0]["range"] = authored.clone();
    expected["label"][1]["location"] = json!({"uri":uri,"range":authored});
    assert_eq!(serde_json::to_value(mapped).unwrap(), expected);
}

#[test]
fn scaffolding_outside_requested_range_and_inexact_edits_are_rejected() {
    let source = "<script setup lang=\"ts\">\nconst label = 'value';\n</script>\n";
    let uri = Url::parse("file:///project/App.vue").unwrap();
    let state = ServerState::new();
    let ctx = IdeContext::testing(&state, &uri, 0, source.into());
    let doc = document(&uri, source);
    let native = json!({"position":point(&doc,source.find("label").unwrap()+5),"label":": Alias<string>","kind":1,"paddingLeft":true});
    let range = Range::new(Position::new(1, 11), Position::new(1, 11));
    assert_eq!(
        serde_json::to_value(
            map_hint(
                &ctx,
                &doc,
                range,
                serde_json::from_value(native.clone()).unwrap()
            )
            .unwrap()
        )
        .unwrap()["label"],
        ": Alias<string>"
    );
    assert!(
        map_hint(
            &ctx,
            &doc,
            Range::new(Position::new(2, 0), Position::new(3, 0)),
            serde_json::from_value(native.clone()).unwrap()
        )
        .is_none()
    );
    let mut scaffold = native.clone();
    scaffold["position"] = json!({"line":0,"character":0});
    assert!(map_hint(&ctx, &doc, range, serde_json::from_value(scaffold).unwrap()).is_none());
    let mut unsafe_edit = native;
    unsafe_edit["textEdits"] = json!([{"range":{"start":{"line":0,"character":0},"end":point(&doc,source.find("label").unwrap())},"newText":"broken"}]);
    assert!(
        map_hint(
            &ctx,
            &doc,
            range,
            serde_json::from_value(unsafe_edit).unwrap()
        )
        .is_none()
    );
}
