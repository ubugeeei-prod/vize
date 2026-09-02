//! JSX spread-child behavior in opt-in Babel compatibility mode (#3391).

use oxc_allocator::Allocator;
use vize_atelier_jsx::{JsxCompatMode, JsxCompileConfig, JsxLang, compile_jsx, parse_module};
use vize_s0::String;

fn compile(source: &str, compat: JsxCompatMode) -> (vize_s0::String, Vec<String>) {
    let bump = vize_s0::Allocator::new();
    let output = compile_jsx(
        &bump,
        source,
        JsxLang::Jsx,
        &JsxCompileConfig {
            compat,
            ..Default::default()
        },
    );
    (
        output.module_code(),
        output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect(),
    )
}

#[test]
fn spread_children_expand_only_in_babel_vdom_compatibility_mode() {
    let source = "const A = () => <div>{...items}</div>;";
    let (native, native_diagnostics) = compile(source, JsxCompatMode::Native);
    let (babel, babel_diagnostics) = compile(source, JsxCompatMode::Babel);

    assert_eq!(native_diagnostics.len(), 1, "{native_diagnostics:?}");
    assert!(native.contains("_toDisplayString(items)"), "{native}");

    assert!(babel_diagnostics.is_empty(), "{babel_diagnostics:?}");
    assert!(babel.contains("[\n    ...items\n  ]"), "{babel}");
    assert!(!babel.contains("toDisplayString"), "{babel}");

    let allocator = Allocator::default();
    let parsed = parse_module(&allocator, babel.as_str(), JsxLang::Jsx);
    assert!(
        parsed.diagnostics.is_empty(),
        "{:?}\n{babel}",
        parsed.diagnostics
    );
}

#[test]
fn spread_children_keep_order_between_neighboring_vnodes() {
    let source = "const A = () => <div><i/>{...items}<b/></div>;";
    let (babel, diagnostics) = compile(source, JsxCompatMode::Babel);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let before = babel.find("_createElementVNode(\"i\")").expect("i vnode");
    let spread = babel.find("...items").expect("spread child");
    let after = babel.find("_createElementVNode(\"b\")").expect("b vnode");
    assert!(before < spread && spread < after, "{babel}");
}

#[test]
fn component_spread_children_expand_inside_the_default_slot() {
    let source = "const A = () => <B>{...items}</B>;";
    let (babel, diagnostics) = compile(source, JsxCompatMode::Babel);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(babel.contains("default: _withCtx(() => ["), "{babel}");
    assert!(babel.contains("...items"), "{babel}");
    assert!(!babel.contains("toDisplayString"), "{babel}");
}
