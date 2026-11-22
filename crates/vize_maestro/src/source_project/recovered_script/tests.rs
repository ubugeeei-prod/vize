use super::*;
use crate::source_project::SourceSnapshotCache;
use tower_lsp::lsp_types::{
    HoverContents, MarkupContent, MarkupKind, TextDocumentContentChangeEvent, Url,
};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::container::{Vue, vue::DescriptorOptions};

pub(super) fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    }
}
pub(super) fn replace(
    docs: &DocumentStore,
    uri: &Url,
    range: Option<Range>,
    text: &str,
    version: i32,
) {
    assert!(docs.apply_changes(
        uri,
        vec![TextDocumentContentChangeEvent {
            range,
            range_length: None,
            text: text.into()
        }],
        version
    ));
}

#[test]
fn first_open_of_a_damaged_file_serves_a_current_utf16_hover_and_survives_an_edit() {
    let docs = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///AlreadyBroken.vue").unwrap();
    let text = "<script lang=ts>\r\nconst pre: string='🦀'; const 雪: number=false;\r\nconst broken: = ;</script>";
    docs.open(uri.clone(), text.into(), 1, "vue".into());
    let initial = cache.capture(&docs, &uri).unwrap();
    let arena = Allocator::new();
    let owner = Vue.observe_descriptor(&arena, initial.source(), options());
    let idx = initial
        .open_literal_prefix(&docs, &owner, ScriptRole::Ordinary)
        .unwrap();
    let position = Position::new(
        1,
        "const pre: string='🦀'; const ".encode_utf16().count() as u32,
    );
    let response = idx.recover(&docs, &initial, &owner).unwrap();
    let published = response.publish(&docs, position).unwrap();
    assert_eq!(
        (published.version, published.bindings, published.assignment),
        (1, 2, Some(Verdict::Mismatch))
    );
    assert_eq!(
        published.hover.unwrap().range,
        Some(Range::new(
            position,
            Position::new(1, position.character + 1)
        ))
    );
    replace(&docs, &uri, None, &text.replace("= ;", "= ["), 2);
    assert_eq!(
        response.publish(&docs, position).unwrap_err(),
        RecoveredScriptError::Host(SnapshotRefusal::Superseded)
    );
    let edited = cache.capture(&docs, &uri).unwrap();
    let current = Vue.observe_descriptor(&arena, edited.source(), options());
    let next = idx
        .recover(&docs, &edited, &current)
        .unwrap()
        .publish(&docs, position)
        .unwrap();
    assert_eq!(
        (next.version, next.bindings, next.assignment),
        (2, 2, Some(Verdict::Mismatch))
    );
}
#[test]
fn successive_keystrokes_recover_original_utf16_bindings_and_reject_stale_publication() {
    let docs = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///Editing.vue").unwrap();
    let old = "<script setup lang=ts>\r\nconst emoji: string='🦀'; let 雪: number = false;\r\nlet unfinished: number = 4;\r\n</script>";
    docs.open(uri.clone(), old.into(), 2, "vue".into());
    let initial = cache.capture(&docs, &uri).unwrap();
    let first = Allocator::new();
    let source = Vue.observe_descriptor(&first, initial.source(), options());
    let saved = initial
        .retain_literal_prefix(&docs, &source, ScriptRole::Setup)
        .unwrap();
    let line = "const emoji: string='🦀'; let ";
    let column = line.encode_utf16().count() as u32;
    // Move the SFC block and leave the last initializer in an actual parse error.
    let new = format!("<!--新-->\r\n{}", old.replace("= 4;", "= ;"));
    replace(&docs, &uri, None, &new, 3);
    let edited = cache.capture(&docs, &uri).unwrap();
    let second = Allocator::new();
    let owner = Vue.observe_descriptor(&second, edited.source(), options());
    assert!(matches!(
        edited.retain_literal_prefix(&docs, &owner, ScriptRole::Setup),
        Err(RecoveredScriptError::Syntax(_))
    ));
    let local = saved.recover(&docs, &edited, &owner).unwrap();
    let result = local.publish(&docs, Position::new(2, column)).unwrap();
    assert_eq!(
        (result.version, result.bindings, result.assignment),
        (3, 2, Some(Verdict::Mismatch))
    );
    assert_eq!(
        result.covered,
        Range::new(
            Position::new(1, "<script setup lang=ts>".len() as u32),
            Position::new(
                2,
                "const emoji: string='🦀'; let 雪: number = false;"
                    .encode_utf16()
                    .count() as u32
            )
        )
    );
    let hover = result.hover.unwrap();
    assert_eq!(
        hover.range,
        Some(Range::new(
            Position::new(2, column),
            Position::new(2, column + 1)
        ))
    );
    assert_eq!(
        hover.contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "```typescript\nlet 雪: number\n```".into()
        })
    );
    assert_eq!(
        local.publish(&docs, Position::new(3, 6)).unwrap().hover,
        None
    );
    // Replace only the earlier binding. The proof for every later byte now
    // depends on changed text as well and must disappear.
    replace(
        &docs,
        &uri,
        Some(Range::new(Position::new(2, 6), Position::new(2, 11))),
        "crab",
        4,
    );
    assert_eq!(
        local.publish(&docs, Position::new(2, column)).unwrap_err(),
        RecoveredScriptError::Host(SnapshotRefusal::Superseded)
    );
    let latest = cache.capture(&docs, &uri).unwrap();
    let third = Allocator::new();
    let descriptor = Vue.observe_descriptor(&third, latest.source(), options());
    let empty = saved
        .recover(&docs, &latest, &descriptor)
        .unwrap()
        .publish(&docs, Position::new(2, 6))
        .unwrap();
    assert_eq!((empty.bindings, empty.hover), (0, None));
}

#[test]
fn role_grammar_document_and_directive_changes_withhold_old_results() {
    let docs = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///First.vue").unwrap();
    let source = "<script lang=ts>var x: boolean=9;</script>";
    docs.open(uri.clone(), source.into(), 8, "vue".into());
    let old = cache.capture(&docs, &uri).unwrap();
    let arena = Allocator::new();
    let original = Vue.observe_descriptor(&arena, old.source(), options());
    let index = old
        .retain_literal_prefix(&docs, &original, ScriptRole::Ordinary)
        .unwrap();
    for (text, reason) in [
        (
            "<script lang=ts>var x: boolean=9;// @ts-nocheck\n</script>",
            RecoveredScriptError::Prefix(PrefixRefusal::DiagnosticDirective),
        ),
        ("<script>var x=9;</script>", RecoveredScriptError::Role),
        (
            "<script setup lang=ts>var x: boolean=9;</script>",
            RecoveredScriptError::Role,
        ),
        (
            "<script src='ext.ts' lang=ts/>",
            RecoveredScriptError::Descriptor,
        ),
    ] {
        docs.open(uri.clone(), text.into(), 9, "vue".into());
        let current = cache.capture(&docs, &uri).unwrap();
        let observation = Vue.observe_descriptor(&arena, current.source(), options());
        assert_eq!(
            index.recover(&docs, &current, &observation).err(),
            Some(reason)
        );
    }
    let other = Url::parse("file:///Second.vue").unwrap();
    docs.open(other.clone(), source.into(), 8, "vue".into());
    let current = cache.capture(&docs, &other).unwrap();
    let observation = Vue.observe_descriptor(&arena, current.source(), options());
    assert_eq!(
        index.recover(&docs, &current, &observation).err(),
        Some(RecoveredScriptError::Document)
    );
}
