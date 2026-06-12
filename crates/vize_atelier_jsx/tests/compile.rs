//! Mode-aware JSX/TSX compilation via [`compile_jsx`] (#1496).
//!
//! These cover the dispatcher in `src/compile.rs`: how the configured default
//! mode and per-component `"use vue:vapor"` / `"use vue:vdom"` directives route
//! each render root to the VDOM or Vapor backend, including a single module that
//! mixes both.

use vize_atelier_jsx::{
    JsxCompileConfig, JsxCompileOutput, JsxComponent, JsxLang, JsxOutputMode, compile_jsx,
    resolve_mode,
};
use vize_carton::Bump;

/// Compile JSX with the given config, asserting it produced no errors. The
/// returned output owns its components, so the arena can be dropped here.
fn compile(src: &str, config: &JsxCompileConfig) -> JsxCompileOutput {
    let bump = Bump::new();
    let out = compile_jsx(&bump, src, JsxLang::Jsx, config);
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    out
}

#[test]
fn resolve_mode_prefers_component_directive_over_default() {
    // An explicit per-component mode always wins.
    assert_eq!(
        resolve_mode(Some(JsxOutputMode::Vapor), JsxOutputMode::Vdom),
        JsxOutputMode::Vapor
    );
    assert_eq!(
        resolve_mode(Some(JsxOutputMode::Vdom), JsxOutputMode::Vapor),
        JsxOutputMode::Vdom
    );
    // No directive falls back to the configured default.
    assert_eq!(
        resolve_mode(None, JsxOutputMode::Vapor),
        JsxOutputMode::Vapor
    );
    assert_eq!(resolve_mode(None, JsxOutputMode::Vdom), JsxOutputMode::Vdom);
}

#[test]
fn default_config_routes_undirected_component_to_vdom() {
    // `JsxOutputMode::Vdom` is the derived default, matching Vue's renderer.
    let out = compile("const App = () => <div/>;", &JsxCompileConfig::default());
    assert_eq!(out.components.len(), 1);
    let component = &out.components[0];
    assert_eq!(component.component_name(), Some("App"));
    assert_eq!(component.mode(), JsxOutputMode::Vdom);
    assert!(
        component.code().contains("_createElementBlock(\"div\")"),
        "{}",
        component.code()
    );
}

#[test]
fn default_mode_can_be_overridden_to_vapor() {
    let config = JsxCompileConfig {
        default_mode: JsxOutputMode::Vapor,
        ..Default::default()
    };
    let out = compile("const App = () => <div/>;", &config);
    assert_eq!(out.components.len(), 1);
    let component = &out.components[0];
    assert_eq!(component.mode(), JsxOutputMode::Vapor);
    // Vapor emits a hoisted template rather than a VDOM block call.
    assert!(
        component.code().contains("template("),
        "{}",
        component.code()
    );
}

#[test]
fn vapor_directive_overrides_vdom_default() {
    let src = "const Fast = () => { \"use vue:vapor\"; return <div/>; };";
    let out = compile(src, &JsxCompileConfig::default());
    assert_eq!(out.components.len(), 1);
    assert_eq!(out.components[0].mode(), JsxOutputMode::Vapor);
}

#[test]
fn vdom_directive_overrides_vapor_default() {
    let config = JsxCompileConfig {
        default_mode: JsxOutputMode::Vapor,
        ..Default::default()
    };
    let src = "function Slow() { \"use vue:vdom\"; return <div/>; }";
    let out = compile(src, &config);
    assert_eq!(out.components.len(), 1);
    assert_eq!(out.components[0].mode(), JsxOutputMode::Vdom);
    assert!(
        out.components[0].code().contains("_createElementBlock"),
        "{}",
        out.components[0].code()
    );
}

#[test]
fn one_module_can_mix_vdom_and_vapor_components() {
    // `A` opts into Vapor via directive; `B` takes the Vdom default. Both are
    // lowered and analyzed once, then routed to their respective backends.
    let src = "const A = () => { \"use vue:vapor\"; return <a/>; };\nconst B = () => <b/>;";
    let out = compile(src, &JsxCompileConfig::default());
    assert_eq!(
        out.components.len(),
        2,
        "expected two components in source order"
    );

    assert_eq!(out.components[0].component_name(), Some("A"));
    assert_eq!(out.components[0].mode(), JsxOutputMode::Vapor);
    assert!(
        out.components[0].code().contains("template("),
        "{}",
        out.components[0].code()
    );

    assert_eq!(out.components[1].component_name(), Some("B"));
    assert_eq!(out.components[1].mode(), JsxOutputMode::Vdom);
    assert!(
        out.components[1]
            .code()
            .contains("_createElementBlock(\"b\")"),
        "{}",
        out.components[1].code()
    );
}

#[test]
fn jsx_component_variant_matches_reported_mode() {
    let src = "const A = () => { \"use vue:vapor\"; return <a/>; };\nconst B = () => <b/>;";
    let out = compile(src, &JsxCompileConfig::default());
    assert_eq!(out.components.len(), 2);

    match &out.components[0] {
        JsxComponent::Vapor(_) => {}
        JsxComponent::Dom(_) => panic!("component A should route to Vapor"),
    }
    assert_eq!(out.components[0].mode(), JsxOutputMode::Vapor);

    match &out.components[1] {
        JsxComponent::Dom(_) => {}
        JsxComponent::Vapor(_) => panic!("component B should route to Vdom"),
    }
    assert_eq!(out.components[1].mode(), JsxOutputMode::Vdom);
}

#[test]
fn empty_module_yields_no_components_and_no_errors() {
    let out = compile("const x = 1;", &JsxCompileConfig::default());
    assert!(out.components.is_empty(), "expected no render roots");
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
}

#[test]
fn vapor_default_config_with_vdom_directive_override_in_one_module() {
    // #1496 acceptance, end to end: a Vapor *global default* (as set by
    // `compiler.jsxMode: "vapor"`) routes undirected components to Vapor, while a
    // `"use vue:vdom"` directive overrides one component back to VDOM — all in a
    // single module that is lowered and analyzed once.
    let config = JsxCompileConfig {
        default_mode: JsxOutputMode::Vapor,
        ..Default::default()
    };
    let src = "const Default = () => <a/>;\n\
               const Overridden = () => { \"use vue:vdom\"; return <b/>; };";
    let bump = Bump::new();
    let out = compile_jsx(&bump, src, JsxLang::Jsx, &config);
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 2);

    // Undirected component takes the Vapor default.
    assert_eq!(out.components[0].component_name(), Some("Default"));
    assert_eq!(out.components[0].mode(), JsxOutputMode::Vapor);
    assert!(
        out.components[0].code().contains("template("),
        "{}",
        out.components[0].code()
    );

    // The `"use vue:vdom"` directive overrides the default back to VDOM.
    assert_eq!(out.components[1].component_name(), Some("Overridden"));
    assert_eq!(out.components[1].mode(), JsxOutputMode::Vdom);
    assert!(
        out.components[1]
            .code()
            .contains("_createElementBlock(\"b\")"),
        "{}",
        out.components[1].code()
    );
}

#[test]
fn invalid_directive_produces_a_diagnostic() {
    // #1496 acceptance: a malformed `"use vue:"` directive is surfaced as an
    // error diagnostic rather than silently ignored.
    let bump = Bump::new();
    let src = "const App = () => { \"use vue:vapour\"; return <div/>; };";
    let out = compile_jsx(&bump, src, JsxLang::Jsx, &JsxCompileConfig::default());
    assert!(
        out.has_errors(),
        "expected a diagnostic for the typo'd directive"
    );
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.is_error() && d.message.as_str().contains("use vue:vapour")),
        "diagnostics: {:?}",
        out.diagnostics
    );
}
