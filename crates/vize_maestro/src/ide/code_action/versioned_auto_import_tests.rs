//! The auto-import insertion retains the existing authored snapshot authority.

use std::sync::Arc;
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

use super::versioned_lint_edits::{LintEditStamp, workspace_edit};
use super::{CodeActionService, IdeContext};
use crate::server::ServerState;

const SOURCE: &str =
    "<!--🌸--><script setup>const label = '🌸'\r\n</script>\r\n<template><MyButton /></template>";

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

fn body_edit(
    ctx: &IdeContext,
    captured: LintEditStamp,
) -> Option<tower_lsp::lsp_types::WorkspaceEdit> {
    let descriptor = ctx.descriptor()?;
    let script = descriptor.script_setup.as_ref()?;
    workspace_edit(
        ctx,
        captured,
        &script.content,
        script.loc.start,
        &[vize_patina::TextEdit::insert(
            0,
            "import MyButton from './MyButton.vue'\n",
        )],
    )
}

#[test]
fn script_body_insertion_has_the_whole_original_utf16_versioned_envelope() {
    let state = ServerState::new();
    let uri = Url::parse("file:///VersionedImport.vue").unwrap();
    opened(&state, &uri);
    let ctx = IdeContext::new(&state, &uri, 0).unwrap();
    let actual = body_edit(&ctx, stamp(&state, &uri)).unwrap();
    eprintln!("whole versioned auto-import insertion: {actual:#?}");
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::json!({
            "documentChanges": [{
                "textDocument": {"uri":uri,"version":7},
                "edits": [{"range":{"start":{"line":0,"character":23},"end":{"line":0,"character":23}},"newText":"import MyButton from './MyButton.vue'\n"}]
            }]
        })
    );
    assert_eq!(state.documents.text(&uri).unwrap(), SOURCE);
}

#[test]
fn script_body_adapter_refuses_wrong_spans_without_crossing_into_template() {
    let state = ServerState::new();
    let uri = Url::parse("file:///ImportSpans.vue").unwrap();
    opened(&state, &uri);
    let ctx = IdeContext::new(&state, &uri, 0).unwrap();
    let captured = stamp(&state, &uri);
    let descriptor = ctx.descriptor().unwrap();
    let script = descriptor.script_setup.as_ref().unwrap();
    let inside_unicode = u32::try_from(script.content.find('🌸').unwrap() + 1).unwrap();
    let outside = u32::try_from(script.content.len() + 1).unwrap();
    let outcomes = [inside_unicode, outside, u32::MAX].map(|offset| {
        workspace_edit(
            &ctx,
            captured,
            &script.content,
            script.loc.start,
            &[vize_patina::TextEdit::insert(offset, "x")],
        )
    });
    eprintln!("whole invalid script insertion outcomes: {outcomes:#?}");
    assert_eq!(outcomes, [None, None, None]);
    assert!(
        workspace_edit(
            &ctx,
            captured,
            "foreign body",
            script.loc.start,
            &[vize_patina::TextEdit::insert(0, "x")]
        )
        .is_none()
    );
    assert!(
        workspace_edit(
            &ctx,
            captured,
            &script.content,
            usize::MAX,
            &[vize_patina::TextEdit::insert(0, "x")]
        )
        .is_none()
    );
    assert_eq!(state.documents.text(&uri).unwrap(), SOURCE);
}

#[test]
fn stale_copied_foreign_reopened_and_closed_snapshots_refuse_import_edits() {
    let state = ServerState::new();
    let first = Url::parse("file:///FirstImport.vue").unwrap();
    let second = Url::parse("file:///OtherImport.vue").unwrap();
    opened(&state, &first);
    opened(&state, &second);
    let ctx = IdeContext::new(&state, &first, 0).unwrap();
    let other = IdeContext::new(&state, &second, 0).unwrap();
    let captured = stamp(&state, &first);
    let copied = IdeContext::with_content(&state, &first, 0, Arc::new(SOURCE.to_owned()));
    assert_eq!(ctx.content, copied.content);
    assert!(!Arc::ptr_eq(&ctx.content, &copied.content));
    let outcomes = [
        body_edit(&other, captured),
        body_edit(&copied, captured),
        body_edit(
            &ctx,
            LintEditStamp {
                version: 8,
                ..captured
            },
        ),
        body_edit(
            &ctx,
            LintEditStamp {
                revision: captured.revision + 1,
                ..captured
            },
        ),
    ];
    eprintln!("whole foreign/copy/stamp import refusals: {outcomes:#?}");
    assert_eq!(outcomes, [None, None, None, None]);
    assert!(body_edit(&ctx, captured).is_some());
    assert!(state.documents.apply_changes(
        &first,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: SOURCE.into()
        }],
        8
    ));
    assert!(body_edit(&ctx, captured).is_none());
    state.documents.close(&first);
    opened(&state, &first);
    assert_ne!(stamp(&state, &first).revision, captured.revision);
    assert!(body_edit(&ctx, captured).is_none());
    state.documents.close(&first);
    assert!(body_edit(&ctx, captured).is_none());
    assert_eq!(state.documents.text(&second).unwrap(), SOURCE);
}

#[test]
fn actual_auto_import_collector_refuses_a_stale_request_before_returning_an_action() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("MyButton.vue"),
        "<template><button /></template>",
    )
    .unwrap();
    let uri = Url::from_file_path(root.path().join("Page.vue")).unwrap();
    let state = ServerState::new();
    opened(&state, &uri);
    let offset = SOURCE.find("MyButton").unwrap() + 4;
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let captured = stamp(&state, &uri);
    let range = Range::new(Position::new(3, 0), Position::new(3, 0));
    let actual = CodeActionService::collect_auto_import(&ctx, range, Some(captured));
    eprintln!("whole current auto-import collector: {actual:#?}");
    assert_eq!(actual.len(), 1);
    assert!(state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: SOURCE.into()
        }],
        8
    ));
    let stale = CodeActionService::collect_auto_import(&ctx, range, Some(captured));
    eprintln!("whole stale auto-import collector: {stale:#?}");
    assert!(stale.is_empty());
    assert_eq!(state.documents.text(&uri).unwrap(), SOURCE);
}
