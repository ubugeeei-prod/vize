//! JSX/TSX -> Vue Vapor **SSR** render codegen (#1533).
//!
//! When [`VaporCompileOptions::ssr`] is set, a JSX/Vapor component is
//! server-rendered: [`vize_atelier_ssr`]'s `ssrRender` codegen is reused to emit
//! an HTML-string render function (`_push(`…`)`) instead of the client Vapor IR
//! pipeline. These tests pin the implemented subset — static elements, static +
//! dynamic attributes, text interpolation, **control flow** (`&&` / ternary
//! v-if, `.map` v-for), and **slots** (default scoped + named/object slots),
//! plus nested-component invocation — and guard that the normal client Vapor
//! output is unchanged when SSR is off.
//!
//! Control flow and slots reuse the shared SSR engine end to end: JSX lowering
//! already produces the same `IfNode` / `ForNode` / `<template v-slot>`
//! structures the SFC SSR path emits, so `SsrCodegenContext::generate` renders
//! them without a JSX-specific codegen fork.

use vize_atelier_jsx::{JsxLang, JsxOutputMode, VaporCompileOptions, compile_to_vapor};
use vize_carton::Bump;

/// Compile a single JSX component and return its generated `code`.
fn compile(src: &str, ssr: bool) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_vapor(&bump, src, JsxLang::Jsx, VaporCompileOptions { ssr });
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

fn ssr(src: &str) -> vize_carton::String {
    compile(src, true)
}

fn client(src: &str) -> vize_carton::String {
    compile(src, false)
}

#[test]
fn ssr_emits_an_ssr_render_function() {
    let code = ssr("const A = () => <div/>;");
    assert!(
        code.contains("function ssrRender(_ctx, _push, _parent, _attrs)"),
        "{code}"
    );
    // Server output is an HTML string pushed via `_push`, not a client render fn.
    assert!(code.contains("_push(`<div"), "{code}");
    assert!(!code.contains("export function render"), "{code}");
    assert!(!code.contains("_template("), "{code}");
}

#[test]
fn ssr_imports_the_server_renderer_helpers() {
    let code = ssr("const A = () => <div>{msg}</div>;");
    assert!(
        code.contains("from \"@vue/server-renderer\""),
        "SSR helpers must import from the server renderer:\n{code}"
    );
}

#[test]
fn ssr_renders_a_static_element_to_an_html_string() {
    let code = ssr("const A = () => <p>hi</p>;");
    assert!(code.contains("_push(`<p"), "{code}");
    assert!(code.contains(">hi</p>`)"), "{code}");
}

#[test]
fn ssr_bakes_a_static_attribute_into_the_html() {
    let code = ssr("const A = () => <div class=\"box\">x</div>;");
    // A purely static attribute is rendered as a merged prop in the fallthrough
    // attrs object (Vue's standard SSR shape for a single root element).
    assert!(code.contains("class: \"box\""), "{code}");
}

#[test]
fn ssr_renders_a_dynamic_attribute_from_the_closure() {
    let code = ssr("const A = () => <div id={x}/>;");
    // Dynamic bindings flow through `_ssrRenderAttrs` / `_mergeProps`, and the
    // bound value stays a bare closure reference (no `_ctx.` prefix).
    assert!(code.contains("id: x"), "{code}");
    assert!(code.contains("_ssrRenderAttrs("), "{code}");
    assert!(!code.contains("_ctx.x"), "{code}");
}

#[test]
fn ssr_interpolates_text_with_the_ssr_helper() {
    let code = ssr("const A = () => <div>{msg}</div>;");
    assert!(code.contains("_ssrInterpolate(msg)"), "{code}");
    assert!(!code.contains("_ctx.msg"), "{code}");
}

#[test]
fn ssr_combines_static_element_dynamic_attr_and_interpolation() {
    // The headline first-cut case: one element carrying a static attribute, a
    // dynamic attribute, and a text interpolation, all in one `_push`.
    let code = ssr("const A = () => <div id={x} class=\"box\">{msg}</div>;");
    assert!(
        code.contains("function ssrRender(_ctx, _push, _parent, _attrs)"),
        "{code}"
    );
    assert!(code.contains("id: x"), "{code}");
    assert!(code.contains("class: \"box\""), "{code}");
    assert!(code.contains("_ssrInterpolate(msg)"), "{code}");
    // Single HTML-string push, no client IR helpers.
    assert!(code.contains("_push(`<div"), "{code}");
    assert!(!code.contains("_renderEffect("), "{code}");
    assert!(!code.contains("_setProp("), "{code}");
}

#[test]
fn ssr_member_expression_interpolation_stays_bare() {
    let code = ssr("const A = () => <p>{user.name}</p>;");
    assert!(code.contains("_ssrInterpolate(user.name)"), "{code}");
    assert!(!code.contains("_ctx.user"), "{code}");
}

#[test]
fn ssr_keeps_the_resolved_component_name_and_vapor_mode() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const Widget = () => <div/>;",
        JsxLang::Jsx,
        VaporCompileOptions { ssr: true },
    );
    assert_eq!(out.components[0].component_name.as_deref(), Some("Widget"));
    assert_eq!(out.components[0].mode, JsxOutputMode::Vapor);
    // SSR output carries no client-only static template strings.
    assert!(out.components[0].templates.is_empty());
}

#[test]
fn tsx_compiles_to_ssr() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const A = (): JSX.Element => <span>{label}</span>;",
        JsxLang::Tsx,
        VaporCompileOptions { ssr: true },
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    let code = &out.components[0].code;
    assert!(code.contains("function ssrRender("), "{code}");
    assert!(code.contains("_ssrInterpolate(label)"), "{code}");
}

#[test]
fn client_output_is_unchanged_when_ssr_is_off() {
    // The exact same source compiled with SSR off must still produce the client
    // Vapor render function (template instantiation + reactive effects), with no
    // SSR machinery leaking in.
    let code = client("const A = () => <div id={x} class=\"box\">{msg}</div>;");
    assert!(code.contains("export function render"), "{code}");
    assert!(code.contains("_template("), "{code}");
    assert!(code.contains("_renderEffect("), "{code}");
    assert!(code.contains("_setText("), "{code}");
    assert!(code.contains("_toDisplayString(msg)"), "{code}");

    // None of the SSR shape appears in client output.
    assert!(!code.contains("ssrRender"), "{code}");
    assert!(!code.contains("_push("), "{code}");
    assert!(!code.contains("@vue/server-renderer"), "{code}");
}

#[test]
fn client_and_ssr_outputs_differ_for_the_same_source() {
    let src = "const A = () => <div>{msg}</div>;";
    assert_ne!(ssr(src).as_str(), client(src).as_str());
}

// ---------------------------------------------------------------------------
// Control flow (#1533): `&&` / ternary -> SSR `if`/`else`; `.map` -> for list.
// ---------------------------------------------------------------------------

#[test]
fn ssr_renders_logical_and_as_a_conditional_push() {
    // `{cond && <span/>}` lowers to a single-branch IfNode; the SSR engine emits
    // an `if` with the absent branch rendering the `<!---->` v-if placeholder.
    let code = ssr("const A = () => <div>{cond && <span>yes</span>}</div>;");
    assert!(code.contains("if (cond) {"), "{code}");
    assert!(code.contains("_push(`<span>yes</span>`)"), "{code}");
    assert!(code.contains("_push(`<!---->`)"), "{code}");
    // Closure semantics: the condition stays bare, never `_ctx.`-prefixed.
    assert!(!code.contains("_ctx."), "{code}");
}

#[test]
fn ssr_renders_a_ternary_as_two_branches() {
    let code = ssr("const A = () => <div>{cond ? <span>a</span> : <em>b</em>}</div>;");
    assert!(code.contains("if (cond) {"), "{code}");
    assert!(code.contains("_push(`<span>a</span>`)"), "{code}");
    assert!(code.contains("} else {"), "{code}");
    assert!(code.contains("_push(`<em>b</em>`)"), "{code}");
}

#[test]
fn ssr_renders_a_map_call_as_a_render_list() {
    // `{items.map((i) => <li>{i}</li>)}` -> `_ssrRenderList` with fragment markers
    // and the inner interpolation rendered server-side.
    let code = ssr("const A = () => <ul>{items.map((i) => <li>{i}</li>)}</ul>;");
    assert!(code.contains("_ssrRenderList(items, (i) =>"), "{code}");
    assert!(code.contains("_ssrInterpolate(i)"), "{code}");
    assert!(
        code.contains("from \"@vue/server-renderer\"") && code.contains("ssrRenderList"),
        "{code}"
    );
    assert!(!code.contains("_ctx."), "{code}");
}

#[test]
fn ssr_renders_a_map_of_components_as_a_render_list_of_components() {
    let code = ssr("const A = () => <ul>{rows.map((row) => <Item data={row}/>)}</ul>;");
    assert!(code.contains("_ssrRenderList(rows, (row) =>"), "{code}");
    assert!(
        code.contains("_ssrRenderComponent(_resolveComponent(\"Item\")"),
        "{code}"
    );
    assert!(code.contains("data: row"), "{code}");
}

#[test]
fn ssr_flattens_a_nested_ternary_into_an_else_if_chain() {
    // Regression for the nested-ternary alternate: previously the inner ternary
    // was sliced into an interpolation expression (`_ssrInterpolate(b ? <em/> :
    // <span/>)`, invalid JS) on SSR and crashed the client transform. It now
    // flattens into a real `if / else if / else` chain.
    let code = ssr("const A = () => <div>{a ? <p>A</p> : b ? <em>B</em> : <span>C</span>}</div>;");
    assert!(code.contains("if (a) {"), "{code}");
    assert!(code.contains("} else if (b) {"), "{code}");
    assert!(code.contains("} else {"), "{code}");
    assert!(code.contains("_push(`<p>A</p>`)"), "{code}");
    assert!(code.contains("_push(`<em>B</em>`)"), "{code}");
    assert!(code.contains("_push(`<span>C</span>`)"), "{code}");
    // The nested ternary must NOT survive as embedded JSX in an interpolation.
    assert!(!code.contains("_ssrInterpolate(b"), "{code}");
    assert!(!code.contains("<em>B</em> :"), "{code}");
}

// ---------------------------------------------------------------------------
// Slots (#1533): default scoped + named/object slots render server-side.
// ---------------------------------------------------------------------------

#[test]
fn ssr_renders_a_default_scoped_slot() {
    // `<List>{(item) => <li>{item}</li>}</List>` lowers to a `<template v-slot>`
    // whose SSR codegen is the standard `_ssrRenderComponent(..., { default:
    // _withCtx(...) })` shape, with the scoped param threaded through bare.
    let code = ssr("const A = () => <List>{(item) => <li>{item}</li>}</List>;");
    assert!(
        code.contains("_ssrRenderComponent(_resolveComponent(\"List\")"),
        "{code}"
    );
    assert!(
        code.contains("default: _withCtx((item, _push, _parent, _scopeId) =>"),
        "{code}"
    );
    assert!(
        code.contains("_push(`<li>${_ssrInterpolate(item)}</li>`)"),
        "{code}"
    );
    assert!(!code.contains("_ctx.item"), "{code}");
}

#[test]
fn ssr_renders_named_object_slots() {
    let code = ssr(
        "const A = () => <Comp>{{ header: () => <h1>H</h1>, default: () => <p>P</p> }}</Comp>;",
    );
    assert!(
        code.contains("_ssrRenderComponent(_resolveComponent(\"Comp\")"),
        "{code}"
    );
    assert!(code.contains("header: _withCtx("), "{code}");
    assert!(code.contains("default: _withCtx("), "{code}");
    assert!(code.contains("_push(`<h1>H</h1>`)"), "{code}");
    assert!(code.contains("_push(`<p>P</p>`)"), "{code}");
}

#[test]
fn ssr_renders_a_nested_component_invocation() {
    // A component used as a plain child renders through `_ssrRenderComponent`,
    // with its bound prop kept as a bare closure reference.
    let code = ssr("const A = () => <div><Child msg={x}/></div>;");
    assert!(code.contains("_push(`<div"), "{code}");
    assert!(
        code.contains("_ssrRenderComponent(_resolveComponent(\"Child\")"),
        "{code}"
    );
    assert!(code.contains("msg: x"), "{code}");
    assert!(!code.contains("_ctx.x"), "{code}");
}

#[test]
fn ssr_combines_control_flow_and_slots_in_one_component() {
    // Headline composite: a component whose default scoped slot body itself uses
    // `.map` — both the slot and the nested for-list render server-side.
    let code = ssr("const A = () => <List>{(rows) => rows.map((r) => <li>{r}</li>)}</List>;");
    assert!(
        code.contains("default: _withCtx((rows, _push, _parent, _scopeId) =>"),
        "{code}"
    );
    assert!(code.contains("_ssrRenderList(rows, (r) =>"), "{code}");
    assert!(code.contains("_ssrInterpolate(r)"), "{code}");
}

// ---------------------------------------------------------------------------
// Client parity: control flow / slots stay on the client Vapor IR when SSR off.
// ---------------------------------------------------------------------------

#[test]
fn client_control_flow_is_unchanged_when_ssr_is_off() {
    // The same control-flow source on the client must drive the Vapor IR paths
    // (`_createIf` / `_createFor`), with no SSR machinery leaking in.
    let cond = client("const A = () => <div>{cond ? <span>a</span> : <em>b</em>}</div>;");
    assert!(cond.contains("_createIf("), "{cond}");
    assert!(!cond.contains("ssrRender"), "{cond}");
    assert!(!cond.contains("_push("), "{cond}");

    let list = client("const A = () => <ul>{items.map((i) => <li>{i}</li>)}</ul>;");
    assert!(list.contains("_createFor("), "{list}");
    assert!(!list.contains("@vue/server-renderer"), "{list}");
}

#[test]
fn client_nested_ternary_lowers_to_nested_create_if() {
    // The nested-ternary fix also unblocks the client path (which previously
    // overflowed the stack on the embedded-JSX interpolation slice). It now
    // lowers to nested `_createIf` calls.
    let code =
        client("const A = () => <div>{a ? <p>A</p> : b ? <em>B</em> : <span>C</span>}</div>;");
    assert!(code.contains("_createIf(() => (a)"), "{code}");
    assert!(code.contains("_createIf(() => (b)"), "{code}");
    assert!(!code.contains("ssrRender"), "{code}");
}
