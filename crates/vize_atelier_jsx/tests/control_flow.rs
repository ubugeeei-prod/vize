//! JSX control-flow expression children -> real v-if / v-for VNodes (feature A).
//!
//! Idiomatic JSX control flow is written as an expression child
//! (`{cond && <X/>}`, `{cond ? <A/> : <B/>}`, `{items.map(i => <li/>)}`). The
//! lowering layer recognizes these patterns and synthesizes structural relief
//! nodes (v-if / v-for) instead of `_toDisplayString(expr)` text.
//!
//! Each integration test file is its own binary, so the `dom`/`vapor` helpers
//! are defined locally (mirroring `tests/dom.rs`). The Vapor helper drives the
//! Vapor backend directly to confirm the same lowered IR also feeds the Vapor
//! if/for codegen paths.

use vize_atelier_jsx::{
    DomCompileOptions, JsxLang, VaporCompileOptions, compile_to_dom, compile_to_vapor,
};
use vize_carton::Bump;

fn dom(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_dom(&bump, src, JsxLang::Jsx, DomCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

fn vapor(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_vapor(&bump, src, JsxLang::Jsx, VaporCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

// 1. `{cond && <X/>}` -> single-branch v-if rendering `<li/>` (not text).
#[test]
fn logical_and_with_jsx_becomes_v_if() {
    let code = dom("const A = () => <ul>{ok && <li/>}</ul>;");
    assert!(!code.contains("_toDisplayString"), "{code}");
    // The `<li/>` is conditionally created, guarded by `ok`.
    assert!(code.contains("_createElementBlock(\"li\""), "{code}");
    assert!(code.contains("(ok)"), "{code}");
    // The absent branch renders an empty comment, the v-if signature.
    assert!(code.contains("_createCommentVNode(\"v-if\""), "{code}");
}

// 2. `{cond ? <A/> : <B/>}` -> two-branch v-if (both arms, no text ternary).
#[test]
fn conditional_with_jsx_arms_becomes_two_branch_v_if() {
    let code = dom("const A = () => <div>{ok ? <a/> : <b/>}</div>;");
    assert!(!code.contains("_toDisplayString"), "{code}");
    assert!(code.contains("_createElementBlock(\"a\""), "{code}");
    assert!(code.contains("_createElementBlock(\"b\""), "{code}");
    assert!(code.contains("(ok)"), "{code}");
}

// 3. `{items.map((i) => <li>{i}</li>)}` -> v-for over `items`.
#[test]
fn map_callback_becomes_v_for() {
    let code = dom("const A = () => <ul>{items.map((i) => <li>{i}</li>)}</ul>;");
    assert!(code.contains("_renderList(items"), "{code}");
    assert!(code.contains("(i) =>"), "{code}");
    // Inner interpolation `{i}` is still real text inside the list item.
    assert!(code.contains("_toDisplayString(i)"), "{code}");
    assert!(code.contains("_createElementBlock(\"li\""), "{code}");
}

// 4. `.map((row, idx) => ...)` -> v-for with both value and index aliases.
#[test]
fn map_callback_with_index_alias() {
    let code = dom("const A = () => <ul>{rows.map((row, idx) => <li key={idx}>{row}</li>)}</ul>;");
    assert!(code.contains("_renderList(rows"), "{code}");
    assert!(code.contains("(row, idx) =>"), "{code}");
    assert!(code.contains("_toDisplayString(row)"), "{code}");
}

// 5. Regression: a plain expression child stays an interpolation.
#[test]
fn plain_expression_still_interpolates() {
    let code = dom("const A = () => <div>{count}</div>;");
    assert!(code.contains("_toDisplayString(count)"), "{code}");
}

// 6. Non-JSX `&&` is value coalescing, not conditional rendering -> stays text.
#[test]
fn non_jsx_logical_and_stays_interpolation() {
    let code = dom("const A = () => <div>{a && b}</div>;");
    assert!(code.contains("_toDisplayString(a && b)"), "{code}");
    // It must NOT have become an If node (no v-if comment fallback).
    assert!(!code.contains("_createCommentVNode(\"v-if\""), "{code}");
}

// 7. Vapor: `{cond && <span/>}` drives the Vapor `createIf` path.
#[test]
fn vapor_logical_and_uses_create_if() {
    let code = vapor("const A = () => <ul>{ok && <span/>}</ul>;");
    assert!(code.contains("_createIf("), "{code}");
    // JSX closure semantics: the condition stays bare, not `_ctx.`-prefixed.
    assert!(code.contains("ok") && !code.contains("_ctx."), "{code}");
    assert!(code.contains("\"<span></span>\""), "{code}");
}

// 8. Vapor: `{items.map(...)}` drives the Vapor `createFor` path.
#[test]
fn vapor_map_uses_create_for() {
    let code = vapor("const A = () => <ul>{items.map((i) => <li/>)}</ul>;");
    assert!(code.contains("_createFor("), "{code}");
    // JSX closure semantics: the for source stays bare, not `_ctx.`-prefixed.
    assert!(code.contains("items") && !code.contains("_ctx."), "{code}");
    assert!(code.contains("\"<li></li>\""), "{code}");
}

// 9. A nested ternary in the alternate is the idiomatic `v-else-if` chain and
//    flattens into one IfNode with multiple branches — rather than slicing the
//    inner ternary into an interpolation expression (invalid JS embedding the
//    JSX), which previously overflowed the Vapor transform's stack. Each arm's
//    element must be reachable as a conditional VNode/template.
#[test]
fn nested_ternary_alternate_flattens_to_else_if_chain() {
    let code = dom("const A = () => <div>{a ? <p/> : b ? <em/> : <span/>}</div>;");
    assert!(!code.contains("_toDisplayString"), "{code}");
    assert!(code.contains("_createElementBlock(\"p\""), "{code}");
    assert!(code.contains("_createElementBlock(\"em\""), "{code}");
    assert!(code.contains("_createElementBlock(\"span\""), "{code}");
    assert!(code.contains("(a)") && code.contains("(b)"), "{code}");
}

// 10. Vapor parity for the nested ternary: it lowers to nested `createIf`
//     without crashing, and conditions stay bare under JSX closure semantics.
#[test]
fn vapor_nested_ternary_uses_nested_create_if() {
    let code = vapor("const A = () => <div>{a ? <p/> : b ? <em/> : <span/>}</div>;");
    assert!(code.contains("_createIf(() => (a)"), "{code}");
    assert!(code.contains("_createIf(() => (b)"), "{code}");
    assert!(!code.contains("_ctx."), "{code}");
}

// 11. A `&&` arm inside a ternary recurses into a nested v-if instead of being
//     mis-lowered as text.
#[test]
fn logical_and_arm_inside_ternary_recurses() {
    let code = dom("const A = () => <div>{a ? <p/> : (cond && <span/>)}</div>;");
    assert!(!code.contains("_toDisplayString"), "{code}");
    assert!(code.contains("_createElementBlock(\"p\""), "{code}");
    assert!(code.contains("_createElementBlock(\"span\""), "{code}");
    // The `&&` arm contributes its own v-if comment fallback.
    assert!(code.contains("_createCommentVNode(\"v-if\""), "{code}");
}
