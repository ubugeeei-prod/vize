//! Regression tests for sibling navigation (#3330, #6727).
//!
//! Vue 3.6.0-rc.9 changed `next(node, logicalIndex)` to
//! `next(node, isText?)`. Passing an element index as the second argument now
//! inserts a blank text node during hydration and shifts all later targets.
//! One-step element navigation must use `_next(node)`; multi-step jumps still
//! use `_nthChild(parent, index)`, which retains the absolute-index contract.

#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use super::compile_vapor;
use vize_carton::{Allocator, String};

fn compile(template: &str) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(&allocator, template, Default::default());
    assert!(
        result.error_messages.is_empty(),
        "expected no errors: {:?}",
        result.error_messages
    );
    result.code.clone()
}

/// Split every `_next(base, isText?)` call in `code` into its arguments,
/// balancing parentheses so a call base like `_child(n2)` stays intact.
fn next_calls(code: &str) -> Vec<(&str, Option<&str>)> {
    let bytes = code.as_bytes();
    let mut calls = Vec::new();
    let mut from = 0;
    while let Some(at) = code[from..].find("_next(") {
        let open = from + at + "_next(".len();
        let (mut depth, mut i, mut comma) = (1usize, open, None);
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b',' if depth == 1 && comma.is_none() => comma = Some(i),
                _ => {}
            }
            i += 1;
        }
        let end = if depth == 0 { i - 1 } else { bytes.len() };
        match comma {
            Some(comma) => calls.push((&code[open..comma], Some(code[comma + 1..end].trim()))),
            None => calls.push((&code[open..end], None)),
        }
        from = open;
    }
    calls
}

/// The reporter's minimal template (#3330): the compiler must navigate three
/// siblings from the first child to reach `<ul>`.
#[test]
fn multi_step_navigation_uses_nth_child_not_a_counted_next() {
    let code = compile(
        r##"<div>
  <svg role="presentation"><use href="#x" /></svg>
  <h2><span>Title</span></h2>
  <p><span>Sub</span></p>
  <ul>
    <li><a href="https://example.com"><img :src="logo" alt="" /><span>A</span></a></li>
    <li><a href="https://example.org"><img :src="logo" alt="" /><span>B</span></a></li>
  </ul>
</div>"##,
    );

    // The `<ul>` is the parent's child at index 3; one `_next` call cannot
    // reach it, so the jump must be an absolute lookup.
    assert!(
        code.contains("_nthChild(n1, 3)"),
        "expected an absolute lookup for the three-sibling jump:\n{code}"
    );
    assert!(
        !code.contains("_next(_child(n1), 3)"),
        "a counted _next advances only one sibling at runtime:\n{code}"
    );
    // The single-sibling hop between the two `<li>` elements stays a bare
    // `_next`, which resolves the logical sibling in the current runtime.
    assert!(
        code.contains("_next(n2)"),
        "expected a single-step _next for the adjacent sibling:\n{code}"
    );
    assert!(
        code.contains("nthChild as _nthChild"),
        "the nthChild helper must be imported:\n{code}"
    );
}

/// Element references must never pass a second argument to `_next`: Vue
/// 3.6.0-rc.9 interprets any truthy second argument as `isText` (#6727).
#[test]
fn element_next_calls_do_not_pass_is_text() {
    // Plain HTML tags only: an unknown tag resolves as a component and never
    // reaches the sibling-navigation path, which would make this vacuous.
    for template in [
        r#"<div><p/><p/><span :id="x"><i/></span></div>"#,
        r#"<div><p/><p/><p/><span :id="x"><i/></span></div>"#,
        r#"<div><span :id="x"><i/></span><p/><p/><span :id="y"><i/></span></div>"#,
        r#"<div><p/><span :id="x"><i/></span><p/><p/><span :id="y"><i/></span><p/><span :id="z"><i/></span></div>"#,
        // A nested parent: navigation inside `<section>` hangs off a `_child`
        // of something other than the root, so the guard below must hold for
        // every parent, not just `n1`.
        r#"<div><section><p/><span :id="x"><i/></span><span :id="y"><i/></span></section><span :id="z"><i/></span></div>"#,
    ] {
        let code = compile(template);
        assert!(
            code.contains("_next(") || code.contains("_nthChild("),
            "expected this template to exercise sibling navigation:\n{template}\n{code}"
        );
        for (_, is_text) in next_calls(&code) {
            assert_eq!(
                is_text, None,
                "an element target must not pass isText:\n{template}\n{code}"
            );
        }
    }
}

/// Index 0 and index 1 keep their existing, correct shapes.
#[test]
fn single_step_navigation_shapes_are_unchanged() {
    let first = compile(r#"<div><a :id="x"/><b/></div>"#);
    assert!(
        first.contains("_child(n1)"),
        "index 0 stays a plain _child:\n{first}"
    );
    assert!(
        !first.contains("_nthChild"),
        "index 0 must not need an absolute lookup:\n{first}"
    );

    let second = compile(r#"<div><a/><b :id="x"/></div>"#);
    assert!(
        second.contains("_next(_child(n1))"),
        "index 1 stays one _next step from the first child:\n{second}"
    );
    assert!(
        !second.contains("_nthChild"),
        "index 1 must not need an absolute lookup:\n{second}"
    );
}

/// A single-step hop after static siblings still uses `_next(node)`.
#[test]
fn single_step_next_after_static_siblings_has_no_second_argument() {
    // `<span :id="y">` is the parent's child at index 3, exactly one rendered
    // sibling past `<span :id="x">` at index 2.
    let code = compile(r#"<div><p/><p/><span :id="x"><i/></span><span :id="y"><i/></span></div>"#);

    let next_calls = next_calls(&code);
    assert!(
        next_calls.iter().any(|(base, _)| base.starts_with('n')),
        "expected a hop from the previous element, got {next_calls:?}:\n{code}"
    );
    assert!(
        next_calls.iter().all(|(_, is_text)| is_text.is_none()),
        "element hops must omit isText, got {next_calls:?}:\n{code}"
    );
}

/// The reporter's hydration case has an input between two dynamic children.
/// Its second and third element references must not be mistaken for blank
/// text targets by Vue 3.6.0-rc.9.
#[test]
fn input_between_interpolations_uses_element_sibling_navigation() {
    let source = r#"<div><h1>{{ a }}</h1><input v-model="q"><p>{{ b }}</p></div>"#;
    for retained in [false, true] {
        let allocator = Allocator::new();
        let result = compile_vapor(
            &allocator,
            source,
            super::VaporCompilerOptions {
                davinci_retained_lane: retained,
                ..Default::default()
            },
        );
        assert!(
            result.error_messages.is_empty(),
            "{:?}",
            result.error_messages
        );
        let code = result.code;
        let calls = next_calls(&code);
        assert!(calls.len() >= 2, "expected adjacent element hops:\n{code}");
        assert!(
            calls.iter().all(|(_, is_text)| is_text.is_none()),
            "input and following paragraph must be element targets:\n{code}"
        );
    }
}
