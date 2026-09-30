use vize_l0::Span;

use super::{EmitDocument, NoLinks, Recorded, Writer};
use crate::runtime::Helper;

#[test]
fn writes_indented_lines() {
    let mut writer = Writer::<NoLinks>::default();
    writer.push("{");
    writer.indent();
    writer.newline();
    writer.push("a");
    writer.deindent();
    writer.deindent();
    writer.newline();
    writer.push("}");
    assert_eq!(writer.as_str(), "{\n  a\n}");
}

#[test]
fn deep_indentation_spans_chunks() {
    let mut writer = Writer::<NoLinks>::default();
    for _ in 0..20 {
        writer.indent();
    }
    writer.newline();
    assert_eq!(writer.len(), 41);
}

#[test]
fn unrecorded_links_leave_text_identical() {
    let mut writer = Writer::<NoLinks>::default();
    writer.anchor(3);
    writer.push_linked("_ctx.a", Span::new(3, 4));
    writer.push_named("b", Span::new(5, 6), "b");
    assert_eq!(writer.as_str(), "_ctx.ab");
}

#[test]
fn preamble_is_joined_last_and_helpers_merge() {
    let (Some(first), Some(second)) = (Helper::from_index(1), Helper::from_index(0)) else {
        return;
    };
    let mut body = Writer::<NoLinks>::default();
    body.use_helper(first);
    body.push("body");
    let mut preamble = Writer::<NoLinks>::default();
    preamble.use_helper(second);
    preamble.use_helper(first);
    preamble.push("head;");
    let done = body.finish_with_preamble(preamble);
    assert_eq!(done.text.as_str(), "head;body");
    assert_eq!(done.helpers.in_use_order(), &[first, second]);
}

#[test]
fn append_merges_fragment_helpers_once() {
    let Some(helper) = Helper::from_index(127) else {
        return;
    };
    assert!(Helper::from_index(128).is_none());
    let mut outer = Writer::<NoLinks>::default();
    outer.use_helper(helper);
    let mut inner = Writer::<NoLinks>::default();
    inner.use_helper(helper);
    inner.push("x");
    outer.append(inner);
    assert_eq!(outer.as_str(), "x");
    assert_eq!(outer.helpers().in_use_order(), &[helper]);
}

#[test]
fn recorded_writer_matches_emit_document() {
    let mut expected = EmitDocument::new(true);
    expected.push_str("head;");
    expected.anchor(2);
    expected.push_linked("_ctx.a", Span::new(2, 3));
    expected.push_named("b", Span::new(4, 5), "b");

    let mut preamble = Writer::<Recorded>::default();
    preamble.push("head;");
    let mut body = Writer::<Recorded>::default();
    body.anchor(2);
    body.push_linked("_ctx.a", Span::new(2, 3));
    let mut fragment = Writer::<Recorded>::default();
    fragment.push_named("b", Span::new(4, 5), "b");
    body.append(fragment);

    let document = body.finish_with_preamble(preamble).into_document();
    assert_eq!(document, expected);
    assert_eq!(
        document.source_map("a.vue", "0123456"),
        expected.source_map("a.vue", "0123456")
    );
}

#[test]
fn unrecorded_writer_finishes_without_links() {
    let mut writer = Writer::<NoLinks>::default();
    writer.push_linked("x", Span::new(0, 1));
    let document = writer.finish().into_document();
    assert!(!document.is_recording());
    assert!(document.links().is_empty());
    assert_eq!(document.as_str(), "x");
}
