//! Regression tests for multi-step sibling navigation (#3330).
//!
//! The Vue Vapor runtime's `next(node, i)` advances **exactly one** sibling
//! outside hydration — `i` is an absolute logical index consulted only while
//! hydrating, never a step count:
//!
//! ```js
//! function next(node, logicalIndex) {
//!   if (isHydrating) return locateChildByLogicalIndex(node.parentNode, logicalIndex)
//!   return _next(node) // one sibling
//! }
//! ```
//!
//! Emitting `_next(node, 3)` for a three-sibling jump therefore landed one
//! sibling over, and the chained `_child()` that followed dereferenced `null`
//! (`TypeError: Cannot read properties of null (reading 'firstChild')`).
//! Multi-step jumps must use `_nthChild(parent, index)`, which honours the
//! index in both modes — the same rule `@vue/compiler-vapor` follows.

use super::compile_vapor;
use vize_carton::{Bump, String, cstr};

fn compile(template: &str) -> String {
    let allocator = Bump::new();
    let result = compile_vapor(&allocator, template, Default::default());
    assert!(
        result.error_messages.is_empty(),
        "expected no errors: {:?}",
        result.error_messages
    );
    result.code.clone()
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
    // The single-sibling hop between the two `<li>` elements stays a `_next`,
    // carrying its absolute index so hydration resolves the same node.
    assert!(
        code.contains("_next(n2, 1)"),
        "expected a single-step _next for the adjacent sibling:\n{code}"
    );
    assert!(
        code.contains("nthChild as _nthChild"),
        "the nthChild helper must be imported:\n{code}"
    );
}

/// No `_next` call may ever carry a step count above one: that argument is a
/// hydration index, and treating it as a count is exactly the #3330 defect.
#[test]
fn no_emitted_next_call_advances_more_than_one_sibling() {
    // Plain HTML tags only: an unknown tag resolves as a component and never
    // reaches the sibling-navigation path, which would make this vacuous.
    for template in [
        r#"<div><p/><p/><span :id="x"><i/></span></div>"#,
        r#"<div><p/><p/><p/><span :id="x"><i/></span></div>"#,
        r#"<div><span :id="x"><i/></span><p/><p/><span :id="y"><i/></span></div>"#,
        r#"<div><p/><span :id="x"><i/></span><p/><p/><span :id="y"><i/></span><p/><span :id="z"><i/></span></div>"#,
    ] {
        let code = compile(template);
        assert!(
            code.contains("_next(") || code.contains("_nthChild("),
            "expected this template to exercise sibling navigation:\n{template}\n{code}"
        );
        // Every `_next(base, i)` advances exactly one sibling at runtime, so
        // its second argument must never be read as a step count. The only
        // shapes the generator may emit are `_next(_child(nP), 1)` and
        // `_next(nX, i)` for an adjacent hop; a `_next` starting from
        // `_child` with an index above 1 is a counted jump — the #3330 defect.
        for index in 2..=8 {
            let counted = cstr!("_next(_child(n1), {})", index);
            assert!(
                !code.contains(counted.as_str()),
                "a counted _next advances only one sibling at runtime:\n{template}\n{code}"
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
        second.contains("_next(_child(n1), 1)"),
        "index 1 stays one _next step from the first child:\n{second}"
    );
    assert!(
        !second.contains("_nthChild"),
        "index 1 must not need an absolute lookup:\n{second}"
    );
}

/// The hydration hint on a single-step `_next` must be the target's absolute
/// index in the parent, not a literal `1`. During hydration the runtime reads
/// it as `locateChildByLogicalIndex(parent, i)`, so a wrong index resolves the
/// wrong node — or `null` — even though client-side rendering looks correct.
#[test]
fn single_step_next_carries_the_targets_absolute_index() {
    // `<span :id="y">` is the parent's child at index 3, exactly one rendered
    // sibling past `<span :id="x">` at index 2.
    let code = compile(r#"<div><p/><p/><span :id="x"><i/></span><span :id="y"><i/></span></div>"#);

    // The base variable name depends on id allocation; the hydration hint is
    // what this pins.
    let next_calls: Vec<&str> = code
        .match_indices("_next(")
        .map(|(at, _)| {
            let rest = &code[at + "_next(".len()..];
            &rest[..rest.find(')').unwrap_or(rest.len())]
        })
        .collect();
    assert!(
        next_calls.iter().any(|call| call.ends_with(", 3")),
        "expected the absolute index 3 as the hydration hint, got {next_calls:?}:\n{code}"
    );
    assert!(
        !next_calls.iter().any(|call| call.ends_with(", 1")),
        "a literal 1 resolves the wrong node while hydrating, got {next_calls:?}:\n{code}"
    );
}

/// Chained bare `_next(node)` calls must never be emitted: each one reaches
/// `locateChildByLogicalIndex(parent, undefined)` during hydration, where no
/// index equals `undefined`, so the chain yields `null`.
#[test]
fn no_bare_next_call_is_emitted() {
    for template in [
        r#"<div><p/><span :id="x"><i/></span><p/><p/><span :id="y"><i/></span></div>"#,
        r#"<div><span :id="x"><i/></span><p/><p/><p/><span :id="y"><i/></span></div>"#,
    ] {
        let code = compile(template);
        for line in code.lines() {
            let mut from = 0;
            while let Some(at) = line[from..].find("_next(") {
                let open = from + at + "_next(".len();
                // Balance parentheses: the base may itself be a call, e.g.
                // `_next(_child(n2), 1)`.
                let (mut depth, mut i, mut has_top_level_comma) = (1usize, open, false);
                let bytes = line.as_bytes();
                while i < bytes.len() && depth > 0 {
                    match bytes[i] {
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        b',' if depth == 1 => has_top_level_comma = true,
                        _ => {}
                    }
                    i += 1;
                }
                assert!(
                    has_top_level_comma,
                    "every _next must carry a hydration index:\n{template}\n{line}"
                );
                from = open;
            }
        }
    }
}
