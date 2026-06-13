//! Mode-aware JSX/TSX compilation via [`compile_jsx`] (#1496).

use std::fmt::Write as _;
use vize_atelier_jsx::{
    JsxCompileConfig, JsxCompileOutput, JsxComponent, JsxLang, JsxOutputMode, compile_jsx,
    resolve_mode,
};
use vize_carton::Bump;

fn compile(src: &str, config: &JsxCompileConfig) -> JsxCompileOutput {
    let bump = Bump::new();
    let out = compile_jsx(&bump, src, JsxLang::Jsx, config);
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    out
}

fn component_matrix(out: &JsxCompileOutput) -> std::string::String {
    let mut snapshot = std::string::String::new();
    for (index, component) in out.components.iter().enumerate() {
        if index > 0 {
            snapshot.push_str("\n\n");
        }
        writeln!(snapshot, "## component {index}").unwrap();
        writeln!(snapshot, "name: {:?}", component.component_name()).unwrap();
        writeln!(snapshot, "mode: {:?}", component.mode()).unwrap();
        writeln!(snapshot, "variant: {}", component_variant(component)).unwrap();
        writeln!(snapshot, "code:").unwrap();
        snapshot.push_str(component.code());
    }
    snapshot
}

fn component_variant(component: &JsxComponent) -> &'static str {
    match component {
        JsxComponent::Dom(_) => "vdom",
        JsxComponent::Vapor(_) => "vapor",
    }
}

#[test]
fn resolve_mode_prefers_component_directive_over_default() {
    assert_eq!(
        resolve_mode(Some(JsxOutputMode::Vapor), JsxOutputMode::Vdom),
        JsxOutputMode::Vapor
    );
    assert_eq!(
        resolve_mode(Some(JsxOutputMode::Vdom), JsxOutputMode::Vapor),
        JsxOutputMode::Vdom
    );
    assert_eq!(
        resolve_mode(None, JsxOutputMode::Vapor),
        JsxOutputMode::Vapor
    );
    assert_eq!(resolve_mode(None, JsxOutputMode::Vdom), JsxOutputMode::Vdom);
}

#[test]
fn default_config_routes_undirected_component_to_vdom() {
    let out = compile("const App = () => <div/>;", &JsxCompileConfig::default());

    assert_eq!(out.components.len(), 1);
    insta::assert_snapshot!(component_matrix(&out));
}

#[test]
fn default_mode_can_be_overridden_to_vapor() {
    let config = JsxCompileConfig {
        default_mode: JsxOutputMode::Vapor,
        ..Default::default()
    };
    let out = compile("const App = () => <div/>;", &config);

    assert_eq!(out.components.len(), 1);
    insta::assert_snapshot!(component_matrix(&out));
}

#[test]
fn component_directives_override_defaults() {
    let vdom_default = compile(
        "const Fast = () => { \"use vue:vapor\"; return <div/>; };",
        &JsxCompileConfig::default(),
    );
    let vapor_default = compile(
        "function Slow() { \"use vue:vdom\"; return <div/>; }",
        &JsxCompileConfig {
            default_mode: JsxOutputMode::Vapor,
            ..Default::default()
        },
    );

    assert_eq!(vdom_default.components[0].mode(), JsxOutputMode::Vapor);
    assert_eq!(vapor_default.components[0].mode(), JsxOutputMode::Vdom);
    insta::assert_snapshot!(format!(
        "## vapor override\n{}\n\n## vdom override\n{}",
        component_matrix(&vdom_default),
        component_matrix(&vapor_default)
    ));
}

#[test]
fn one_module_can_mix_vdom_and_vapor_components() {
    let src = "const A = () => { \"use vue:vapor\"; return <a/>; };\nconst B = () => <b/>;";
    let out = compile(src, &JsxCompileConfig::default());

    assert_eq!(out.components.len(), 2);
    insta::assert_snapshot!(component_matrix(&out));
}

#[test]
fn empty_module_yields_no_components_and_no_errors() {
    let out = compile("const x = 1;", &JsxCompileConfig::default());

    assert!(out.components.is_empty(), "expected no render roots");
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
}

#[test]
fn vapor_default_with_vdom_directive_override_in_one_module() {
    let config = JsxCompileConfig {
        default_mode: JsxOutputMode::Vapor,
        ..Default::default()
    };
    let src = "const Default = () => <a/>;\n\
               const Overridden = () => { \"use vue:vdom\"; return <b/>; };";
    let out = compile(src, &config);

    assert_eq!(out.components.len(), 2);
    insta::assert_snapshot!(component_matrix(&out));
}

#[test]
fn invalid_directive_produces_a_diagnostic() {
    let bump = Bump::new();
    let src = "const App = () => { \"use vue:vapour\"; return <div/>; };";
    let out = compile_jsx(&bump, src, JsxLang::Jsx, &JsxCompileConfig::default());

    assert!(
        out.has_errors(),
        "expected a diagnostic for the typo'd directive"
    );
    insta::assert_debug_snapshot!(out.diagnostics);
}
