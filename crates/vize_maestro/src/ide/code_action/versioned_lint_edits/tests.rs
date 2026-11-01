use super::*;
use crate::server::ServerState;
use std::sync::Arc;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

const TEMPLATE: &str = "\r\n  <!-- 雪🌸 --> <my-widget />\r\n";
const SOURCE: &str = "<template>\r\n  <!-- 雪🌸 --> <my-widget />\r\n</template>";

fn opened(state: &ServerState, uri: &Url) {
    state
        .documents
        .open(uri.clone(), SOURCE.into(), 7, "vue".into());
}
fn stamp(state: &ServerState, uri: &Url) -> LintEditStamp {
    let document = state.documents.get(uri).unwrap();
    LintEditStamp {
        revision: document.revision(),
        version: document.version,
    }
}
fn edit() -> vize_patina::TextEdit {
    let start = u32::try_from(TEMPLATE.find("my-widget").unwrap()).unwrap();
    vize_patina::TextEdit::replace(start, start + 9, "MyWidget")
}
fn result(
    ctx: &IdeContext,
    stamp: LintEditStamp,
    edits: &[vize_patina::TextEdit],
) -> Option<WorkspaceEdit> {
    workspace_edit(ctx, stamp, TEMPLATE, "<template>".len(), edits)
}

#[test]
fn checked_original_spans_emit_whole_versioned_utf16_edits_without_changing_source() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Versioned.vue").unwrap();
    opened(&state, &uri);
    let ctx = IdeContext::new(&state, &uri, 0).unwrap();
    let at_end = u32::try_from(TEMPLATE.len()).unwrap();
    let edits = [edit(), vize_patina::TextEdit::insert(at_end, "\t")];
    let actual = result(&ctx, stamp(&state, &uri), &edits).unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::json!({
            "documentChanges": [{
                "textDocument": { "uri": uri, "version": 7 },
                "edits": [
                    { "range": { "start": {"line":1,"character":16}, "end": {"line":1,"character":25} }, "newText": "MyWidget" },
                    { "range": { "start": {"line":2,"character":0}, "end": {"line":2,"character":0} }, "newText": "\t" }
                ]
            }]
        })
    );
    assert_eq!(state.documents.text(&uri).unwrap(), SOURCE);
}

#[test]
fn stale_versions_reopened_documents_and_missing_documents_refuse_before_publication() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Stale.vue").unwrap();
    opened(&state, &uri);
    let ctx = IdeContext::new(&state, &uri, 0).unwrap();
    let captured = stamp(&state, &uri);
    assert!(result(&ctx, captured, &[edit()]).is_some());
    assert!(
        result(
            &ctx,
            LintEditStamp {
                version: 8,
                ..captured
            },
            &[edit()]
        )
        .is_none()
    );
    assert!(state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: SOURCE.into(),
        }],
        8
    ));
    assert!(result(&ctx, captured, &[edit()]).is_none());
    state.documents.close(&uri);
    opened(&state, &uri);
    assert_ne!(stamp(&state, &uri).revision, captured.revision);
    assert!(result(&ctx, captured, &[edit()]).is_none());
    state.documents.close(&uri);
    assert!(result(&ctx, captured, &[edit()]).is_none());
}

#[test]
fn foreign_uri_and_equal_copied_buffer_cannot_borrow_the_original_owner() {
    let state = ServerState::new();
    let first = Url::parse("file:///First.vue").unwrap();
    let second = Url::parse("file:///Second.vue").unwrap();
    opened(&state, &first);
    opened(&state, &second);
    let ctx = IdeContext::new(&state, &first, 0).unwrap();
    let other = IdeContext::new(&state, &second, 0).unwrap();
    let captured = stamp(&state, &first);
    assert!(result(&other, captured, &[edit()]).is_none());
    let foreign = IdeContext::with_content(&state, &first, 0, Arc::new(SOURCE.to_string()));
    assert_eq!(ctx.content, foreign.content);
    assert!(!Arc::ptr_eq(&ctx.content, &foreign.content));
    assert!(result(&foreign, captured, &[edit()]).is_none());
    assert!(result(&ctx, captured, &[edit()]).is_some());
}

#[test]
fn invalid_body_ranges_utf8_boundaries_order_and_overlaps_refuse_without_partial_edits() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Refusals.vue").unwrap();
    opened(&state, &uri);
    let ctx = IdeContext::new(&state, &uri, 0).unwrap();
    let captured = stamp(&state, &uri);
    let inside_unicode = u32::try_from(TEMPLATE.find('雪').unwrap() + 1).unwrap();
    for edits in [
        vec![vize_patina::TextEdit::replace(3, 2, "x")],
        vec![vize_patina::TextEdit::replace(
            inside_unicode,
            inside_unicode + 1,
            "x",
        )],
        vec![vize_patina::TextEdit::insert(u32::MAX, "x")],
        vec![
            vize_patina::TextEdit::insert(4, "a"),
            vize_patina::TextEdit::insert(3, "b"),
        ],
        vec![
            vize_patina::TextEdit::insert(4, "a"),
            vize_patina::TextEdit::insert(4, "b"),
        ],
        vec![
            vize_patina::TextEdit::replace(2, 5, "a"),
            vize_patina::TextEdit::replace(4, 6, "b"),
        ],
    ] {
        assert!(result(&ctx, captured, &edits).is_none());
        assert_eq!(state.documents.text(&uri).unwrap(), SOURCE);
    }
    assert!(workspace_edit(&ctx, captured, "different", "<template>".len(), &[edit()]).is_none());
    assert!(workspace_edit(&ctx, captured, TEMPLATE, usize::MAX, &[edit()]).is_none());
}
