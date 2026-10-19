//! Checker-owned labels retain their structured native presentation.

use super::InlayHintService;
use crate::ide::corsa_support::{
    CanonicalVirtualDocument, map_canonical_corsa_location, map_canonical_exact_edit_range,
    open_canonical_script_document, open_canonical_virtual_document,
};
use crate::ide::{IdeContext, offset_to_position};
use tower_lsp::lsp_types::{InlayHint, InlayHintLabel, Range};
use vize_canon::CorsaBridge;

impl InlayHintService {
    pub(crate) async fn native_hints(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        range: Range,
    ) -> Vec<InlayHint> {
        if range.start > range.end {
            return Vec::new();
        }
        let document = if crate::utils::is_plain_script_path(ctx.uri.path())
            || crate::utils::is_jsx_path(ctx.uri.path())
        {
            open_canonical_script_document(ctx, bridge, false).await
        } else {
            open_canonical_virtual_document(ctx, bridge).await
        };
        trace_native_inlay("canonicalDocument", || {
            serde_json::json!({
                "authoredUri":ctx.uri,"requestedRange":range,
                "document":document.as_ref().map(|document| serde_json::json!({
                    "requestUri":document.request_uri,"code":document.virtual_result.code
                }))
            })
        });
        let Some(document) = document else {
            return Vec::new();
        };
        // One range request, independent of the number of authored bindings.
        // Generated scaffolding is rejected by the exact authored projection.
        let end = offset_to_position(
            &document.virtual_result.code,
            document.virtual_result.code.len(),
        );
        let response = bridge
            .inlay_hints(document.request_uri.as_str(), (0, 0), end)
            .await;
        trace_native_inlay("canonicalResponse", || match &response {
            Ok(value) => serde_json::json!({"ok":true,"response":value}),
            Err(error) => serde_json::json!({"ok":false,"error":error.to_string()}),
        });
        let Ok(Some(response)) = response else {
            return Vec::new();
        };
        let hints = serde_json::from_value::<Vec<InlayHint>>(response);
        trace_native_inlay("decodedHints", || match &hints {
            Ok(value) => serde_json::json!({"ok":true,"hints":value}),
            Err(error) => serde_json::json!({"ok":false,"error":error.to_string()}),
        });
        let Ok(hints) = hints else {
            return Vec::new();
        };
        let mapped: Vec<_> = hints
            .into_iter()
            .filter_map(|hint| map_hint(ctx, &document, range, hint))
            .collect();
        trace_native_inlay("mappedHints", || serde_json::json!(mapped));
        mapped
    }
}

fn map_hint(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    requested: Range,
    mut hint: InlayHint,
) -> Option<InlayHint> {
    let point = Range::new(hint.position, hint.position);
    hint.position = map_canonical_exact_edit_range(ctx, document, point)?.start;
    if !InlayHintService::position_in_range(hint.position, requested) {
        return None;
    }
    if let Some(edits) = &mut hint.text_edits {
        for edit in edits {
            edit.range = map_canonical_exact_edit_range(ctx, document, edit.range)?;
        }
    }
    if let InlayHintLabel::LabelParts(parts) = &mut hint.label {
        for part in parts {
            if let Some(location) = &part.location {
                let native = serde_json::from_value(serde_json::to_value(location).ok()?).ok()?;
                part.location = map_canonical_corsa_location(ctx, document, &native);
            }
        }
    }
    Some(hint)
}

fn trace_native_inlay(stage: &str, value: impl FnOnce() -> serde_json::Value) {
    if std::env::var_os("VIZE_TRACE_EDITOR_PREPARATION").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        tracing::warn!(stage, value = %value(), "native inlay hint observation");
    }
}

#[cfg(test)]
mod tests {
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
}
