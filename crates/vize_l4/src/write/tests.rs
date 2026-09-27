use vize_l0::Span;

use super::{NoLinks, Writer};
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
