use super::*;
use crate::ide::code_action::versioned_lint_edits::LintEditStamp;
use crate::server::ServerState;
use std::sync::Arc;
use tower_lsp::lsp_types::{DocumentChanges, OneOf, Position, TextDocumentContentChangeEvent, Url};

const INLINE: &str = "<template><div title=\"hello\"  class=\"a\">text</div></template>";
const UNICODE: &str = "<!-- 😀 --><template><div title=\"😀\"  class=\"a\">text</div></template>";
const HEADER: &str = "<template\n  ><div title=\"hello\"  class=\"a\">text</div></template>";
const CRLF: &str = "<script setup>const label = '😀'</script>\r\n<template>\r\n\t<div title=\"hello\"  class=\"a\">text</div>\r\n</template>\r\n";

fn stamp(state: &ServerState, uri: &Url) -> LintEditStamp {
    let document = state.documents.get(uri).unwrap();
    LintEditStamp {
        revision: document.revision(),
        version: document.version,
    }
}

fn gap_range(source: &str) -> Range {
    let offset = source.find("  class=").unwrap();
    let (line, character) = crate::ide::offset_to_position(source, offset);
    Range::new(
        Position::new(line, character),
        Position::new(line, character + 2),
    )
}

fn collect(ctx: &IdeContext, captured: LintEditStamp) -> Vec<CodeActionOrCommand> {
    let lint = CodeActionService::lint_template_once(ctx).unwrap();
    CodeActionService::collect_forget_suppress(
        ctx,
        gap_range(&ctx.content),
        &lint,
        &[],
        Some(captured),
    )
}

fn expected(
    uri: &Url,
    line: u32,
    character: u32,
    text: &str,
    version: Option<i32>,
) -> serde_json::Value {
    let edits = serde_json::json!([{"range":{"start":{"line":line,"character":character},"end":{"line":line,"character":character}},"newText":text}]);
    let edit = if let Some(version) = version {
        serde_json::json!({"documentChanges":[{"textDocument":{"uri":uri,"version":version},"edits":edits}]})
    } else {
        serde_json::json!({"changes":{(uri.as_str()):edits}})
    };
    serde_json::json!([{"title":"Suppress with @vize:forget (vue/no-multi-spaces)","kind":"quickfix","edit":edit,"isPreferred":false}])
}

#[test]
fn original_inline_unicode_header_and_crlf_insertions_keep_whole_edits_and_requery() {
    for (source, newline, indent, line, character, byte_offset) in [
        (INLINE, "\n", "", 0, 10, 10),
        (UNICODE, "\n", "", 0, 21, 23),
        (HEADER, "\n", "", 1, 3, 13),
        (CRLF, "\r\n", "\t", 2, 0, CRLF.find("\t<div").unwrap()),
    ] {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/OriginalSuppression.vue").unwrap();
        state
            .documents
            .open(uri.clone(), source.into(), 7, "vue".into());
        let offset = source.find("  class=").unwrap() + 1;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let lint = CodeActionService::lint_template_once(&ctx).unwrap();
        let unversioned =
            CodeActionService::collect_forget_suppress(&ctx, gap_range(source), &lint, &[], None);
        let actual = collect(&ctx, stamp(&state, &uri));
        eprintln!(
            "whole original suppression source={source:?}; fallback={unversioned:#?}; advertised={actual:#?}"
        );
        let text = [indent, "<!-- @vize:forget vue/no-multi-spaces -->", newline].concat();
        assert_eq!(
            serde_json::to_value(unversioned).unwrap(),
            expected(&uri, line, character, &text, None)
        );
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            expected(&uri, line, character, &text, Some(7))
        );
        let CodeActionOrCommand::CodeAction(action) = &actual[0] else {
            panic!("expected suppression action")
        };
        let Some(DocumentChanges::Edits(documents)) =
            &action.edit.as_ref().unwrap().document_changes
        else {
            panic!("expected document edit")
        };
        let OneOf::Left(edit) = &documents[0].edits[0] else {
            panic!("expected plain text edit")
        };
        let start = crate::ide::position_to_offset(
            source,
            edit.range.start.line,
            edit.range.start.character,
        )
        .unwrap();
        let end =
            crate::ide::position_to_offset(source, edit.range.end.line, edit.range.end.character)
                .unwrap();
        assert_eq!((start, end), (byte_offset, byte_offset));
        let mut applied = source.to_owned();
        applied.insert_str(start, &edit.new_text);
        let intended = [
            source.get(..byte_offset).unwrap(),
            &text,
            source.get(byte_offset..).unwrap(),
        ]
        .concat();
        eprintln!("whole actually applied template suppression: {applied:?}");
        assert_eq!(applied, intended);
        assert_eq!(state.documents.text(&uri).unwrap(), source);
        assert!(state.documents.apply_changes(
            &uri,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: applied.clone()
            }],
            8
        ));
        state.update_virtual_docs(&uri, &applied);
        let next_offset = applied.find("  class=").unwrap() + 1;
        let next = IdeContext::new(&state, &uri, next_offset).unwrap();
        let clean = collect(&next, stamp(&state, &uri));
        eprintln!("whole fresh-cursor suppression requery: {clean:#?}");
        assert!(clean.is_empty());
        assert_eq!(state.documents.text(&uri).unwrap(), applied);
    }
}

#[test]
fn malformed_diagnostic_and_body_spans_refuse_instead_of_guessing_the_first_line() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/SuppressionBounds.vue").unwrap();
    state
        .documents
        .open(uri.clone(), UNICODE.into(), 7, "vue".into());
    let ctx = IdeContext::new(&state, &uri, UNICODE.find("  class=").unwrap()).unwrap();
    let captured = stamp(&state, &uri);
    let original = CodeActionService::lint_template_once(&ctx).unwrap();
    let inside = u32::try_from(original.content.find('😀').unwrap() + 1).unwrap();
    let beyond = u32::try_from(original.content.len() + 1).unwrap();
    let broad = Range::new(Position::new(0, 0), Position::new(999, 999));
    for (start, end) in [
        (2, 1),
        (inside, inside + 1),
        (beyond, beyond),
        (0, u32::MAX),
    ] {
        let mut lint = CodeActionService::lint_template_once(&ctx).unwrap();
        let diagnostic = lint
            .result
            .diagnostics
            .iter_mut()
            .find(|x| x.rule_name == "vue/no-multi-spaces")
            .unwrap();
        diagnostic.start = start;
        diagnostic.end = end;
        let actual =
            CodeActionService::collect_forget_suppress(&ctx, broad, &lint, &[], Some(captured));
        eprintln!("whole malformed original diagnostic span {start}..{end}: {actual:#?}");
        assert!(actual.is_empty());
    }
    for start in [original.start_offset + 1, UNICODE.len() + 1, usize::MAX] {
        let mut lint = CodeActionService::lint_template_once(&ctx).unwrap();
        lint.start_offset = start;
        let actual =
            CodeActionService::collect_forget_suppress(&ctx, broad, &lint, &[], Some(captured));
        eprintln!("whole foreign body start {start}: {actual:#?}");
        assert!(actual.is_empty());
    }
    let mut foreign = CodeActionService::lint_template_once(&ctx).unwrap();
    foreign.content = foreign.content.replace("class", "other");
    assert!(
        CodeActionService::collect_forget_suppress(&ctx, broad, &foreign, &[], Some(captured))
            .is_empty()
    );
    assert_eq!(state.documents.text(&uri).unwrap(), UNICODE);
}

#[test]
fn stale_foreign_copied_reopened_and_closed_owners_never_publish_suppression_edits() {
    let state = ServerState::new();
    let first = Url::parse("file:///workspace/FirstSuppression.vue").unwrap();
    let second = Url::parse("file:///workspace/OtherSuppression.vue").unwrap();
    for uri in [&first, &second] {
        state
            .documents
            .open(uri.clone(), UNICODE.into(), 7, "vue".into());
    }
    let ctx = IdeContext::new(&state, &first, UNICODE.find("  class=").unwrap()).unwrap();
    let other = IdeContext::new(&state, &second, ctx.offset).unwrap();
    let copied = IdeContext::with_content(&state, &first, ctx.offset, Arc::new(UNICODE.to_owned()));
    let captured = stamp(&state, &first);
    assert_eq!(ctx.content, copied.content);
    assert!(!Arc::ptr_eq(&ctx.content, &copied.content));
    let outcomes = [
        collect(&other, captured),
        collect(&copied, captured),
        collect(
            &ctx,
            LintEditStamp {
                version: 8,
                ..captured
            },
        ),
        collect(
            &ctx,
            LintEditStamp {
                revision: captured.revision + 1,
                ..captured
            },
        ),
    ];
    eprintln!("whole suppression owner/version refusals: {outcomes:#?}");
    assert!(outcomes.iter().all(Vec::is_empty));
    assert_eq!(collect(&ctx, captured).len(), 1);
    assert!(state.documents.apply_changes(
        &first,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: UNICODE.into()
        }],
        8
    ));
    assert!(collect(&ctx, captured).is_empty());
    state.documents.close(&first);
    state
        .documents
        .open(first.clone(), UNICODE.into(), 7, "vue".into());
    assert_ne!(stamp(&state, &first).revision, captured.revision);
    assert!(collect(&ctx, captured).is_empty());
    state.documents.close(&first);
    assert!(collect(&ctx, captured).is_empty());
    assert_eq!(state.documents.text(&second).unwrap(), UNICODE);
}
