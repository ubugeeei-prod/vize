//! Lowercase tag classification in opt-in Babel compatibility mode (#3391).

use vize_atelier_jsx::{JsxCompatMode, JsxCompileConfig, JsxLang, compile_jsx};
use vize_s0::Allocator;

fn compile_module(source: &str, compat: JsxCompatMode) -> vize_s0::String {
    let bump = Allocator::new();
    compile_jsx(
        &bump,
        source,
        JsxLang::Jsx,
        &JsxCompileConfig {
            compat,
            ..Default::default()
        },
    )
    .module_code()
}

#[test]
fn non_html_non_svg_lowercase_tags_resolve_only_when_opted_in() {
    for (tag, source, native_call, resolution, component_call) in [
        (
            "foo",
            "const A = () => <foo/>;",
            "_createElementBlock(\"foo\")",
            "_resolveComponent(\"foo\")",
            "_createBlock(_component_foo)",
        ),
        (
            "mi",
            "const A = () => <mi/>;",
            "_createElementBlock(\"mi\")",
            "_resolveComponent(\"mi\")",
            "_createBlock(_component_mi)",
        ),
    ] {
        let native = compile_module(source, JsxCompatMode::Native);
        let babel = compile_module(source, JsxCompatMode::Babel);

        assert!(native.contains(native_call), "{native}");
        assert!(babel.contains(resolution), "{babel}");
        assert!(babel.contains(component_call), "{babel}");
        assert_ne!(native, babel, "{tag}");
    }
}

#[test]
fn dashed_lowercase_is_classified_before_slot_lowering() {
    let source = "const A = () => <my-el>{{ default: () => <span/> }}</my-el>;";
    let native = compile_module(source, JsxCompatMode::Native);
    let babel = compile_module(source, JsxCompatMode::Babel);

    assert!(native.contains("_toDisplayString({"), "{native}");
    assert!(babel.contains("default: _withCtx(() => ["), "{babel}");
    assert!(!babel.contains("_toDisplayString({"), "{babel}");
    assert_ne!(native, babel);
}

#[test]
fn html_svg_and_known_namespaces_stay_intrinsic() {
    for (tag, source, element_call) in [
        (
            "div",
            "const A = () => <div/>;",
            "_createElementBlock(\"div\")",
        ),
        (
            "circle",
            "const A = () => <circle/>;",
            "_createElementBlock(\"circle\")",
        ),
        (
            "svg:circle",
            "const A = () => <svg:circle/>;",
            "_createElementBlock(\"svg:circle\")",
        ),
        (
            "math:mi",
            "const A = () => <math:mi/>;",
            "_createElementBlock(\"math:mi\")",
        ),
    ] {
        let native = compile_module(source, JsxCompatMode::Native);
        let babel = compile_module(source, JsxCompatMode::Babel);

        assert_eq!(native, babel, "{tag}");
        assert!(babel.contains(element_call), "{babel}");
        assert!(!babel.contains("_resolveComponent"), "{babel}");
    }
}
