//! JSX/TSX -> Vue Vapor **SSR** render codegen (#1533, first cut).
//!
//! When [`VaporCompileOptions::ssr`] is set, a JSX/Vapor component is
//! server-rendered: [`vize_atelier_ssr`]'s `ssrRender` codegen is reused to emit
//! an HTML-string render function (`_push(`…`)`) instead of the client Vapor IR
//! pipeline. These tests pin the implemented first-cut subset — static elements,
//! static + dynamic attributes, and text interpolation — and guard that the
//! normal client Vapor output is unchanged when SSR is off.

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
