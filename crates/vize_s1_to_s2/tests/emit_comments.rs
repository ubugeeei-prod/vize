//! Ordinary template comments in the S2 DOM emitter.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_s0::Allocator;
use vize_s1_to_s2::{DomEmitOptions, LegacyCaps, emit_dom_source, emit_dom_source_with_options};

fn assembled_with_comments(source: &str, comments: bool) -> String {
    let allocator = Allocator::new();
    emit_dom_source_with_options(
        &allocator,
        source,
        LegacyCaps::VUE3,
        &DomEmitOptions {
            comments,
            ..DomEmitOptions::DEFAULT
        },
    )
    .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
    .assembled()
    .to_string()
}

#[test]
fn ordinary_template_comments_emit_comment_vnodes_when_enabled() {
    let output = assembled_with_comments("<div><!--kept--><span>ok</span></div>", true);

    assert!(
        output.contains("createCommentVNode: _createCommentVNode"),
        "expected comment helper in output:\n{output}"
    );
    assert!(
        output.contains("_createCommentVNode(\"kept\")"),
        "expected preserved comment vnode in output:\n{output}"
    );
    assert!(
        output.contains("_createElementVNode(\"span\", null, \"ok\")"),
        "expected sibling vnode to stay emitted in output:\n{output}"
    );
}

#[test]
fn ordinary_template_comments_stay_dropped_by_default() {
    let allocator = Allocator::new();
    let output = emit_dom_source(&allocator, "<div><!--kept--><span>ok</span></div>")
        .unwrap_or_else(|error| panic!("emit refused default comments=false case: {error:?}"))
        .assembled()
        .to_string();

    assert!(
        !output.contains("_createCommentVNode"),
        "expected default lowering to drop comments:\n{output}"
    );
}

#[test]
fn component_default_slots_preserve_comment_children_when_enabled() {
    let output = assembled_with_comments("<Foo><!--slot--><span>ok</span></Foo>", true);

    assert!(
        output.contains("_createCommentVNode(\"slot\")"),
        "expected component default slot comment vnode in output:\n{output}"
    );
    assert!(
        output.contains("_createElementVNode(\"span\", null, \"ok\")"),
        "expected component default slot sibling vnode in output:\n{output}"
    );
}

#[test]
fn slot_outlet_fallbacks_preserve_comment_children_when_enabled() {
    let output = assembled_with_comments("<slot><!--fallback--><span>ok</span></slot>", true);

    assert!(
        output.contains("_createCommentVNode(\"fallback\")"),
        "expected slot fallback comment vnode in output:\n{output}"
    );
    assert!(
        output.contains("_createElementVNode(\"span\", null, \"ok\")"),
        "expected slot fallback sibling vnode in output:\n{output}"
    );
}
