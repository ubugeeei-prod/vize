//! Nested interactive-content recovery must match the shipped DOM lane once
//! S1 lowers to S2. The AIRI DOM corpus exposed this with same-named ancestor
//! wrappers around nested `<a>` and `<button>` elements.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom_source;

fn shipped(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, old) = vize_atelier_dom::compile_template(&allocator, source);
    assert!(
        errors.iter().all(|error| error.is_compatibility_notice()),
        "{source}: shipped lane should recover with compatibility notices only: {errors:?}"
    );
    format!("{}\n{}", old.preamble, old.code)
}

fn emitted(source: &str) -> String {
    let allocator = Allocator::new();
    emit_dom_source(&allocator, source)
        .unwrap_or_else(|error| panic!("S2 DOM emit refused {source:?}: {error:?}"))
        .assembled()
        .to_string()
}

#[test]
fn nested_anchor_with_same_named_ancestor_matches_shipped_dom() {
    let source = r#"<div><a href="/"><div><a href="/foo">inner</a></div></a></div>"#;
    assert_eq!(emitted(source), shipped(source));
}

#[test]
fn nested_button_with_same_named_ancestor_matches_shipped_dom() {
    let source = "<div><button><div><button>bbb</button></div></button></div>";
    assert_eq!(emitted(source), shipped(source));
}

#[test]
fn foreign_anchor_children_stay_nested_and_match_shipped_dom() {
    let source = "<svg><a><a>x</a></a></svg>";
    assert_eq!(emitted(source), shipped(source));
}
