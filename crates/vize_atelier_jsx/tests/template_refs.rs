//! Template refs authored in JSX reach the runtime ref setter intact.
#![expect(
    clippy::disallowed_types,
    reason = "tests assert by panicking; fixtures use std strings"
)]

use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx};
use vize_l0::Allocator;

fn component_code(src: &str, default_mode: JsxOutputMode) -> std::string::String {
    let bump = Allocator::new();
    let config = JsxCompileConfig {
        default_mode,
        ..Default::default()
    };
    let out = compile_jsx(&bump, src, JsxLang::Jsx, &config);
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected exactly one component");
    out.components[0].code().into()
}

/// A bare `ref_for` is the boolean flag that turns array collection on; an
/// empty-string value after the implicit flag would turn it back off.
#[test]
fn bare_ref_for_keeps_array_collection_in_vdom() {
    let code = component_code(
        "const App = () => { const els = ref([]); return () => <ul>{items.map((i) => <li key={i} ref={els} ref_for>{i}</li>)}</ul>; };",
        JsxOutputMode::Vdom,
    );
    assert_eq!(code.matches("ref_for: \"\"").count(), 0, "{code}");
    // In an object literal the last duplicate key wins.
    let last_flag = code
        .rfind("ref_for: ")
        .and_then(|start| code.get(start..start + "ref_for: true".len()));
    assert_eq!(last_flag, Some("ref_for: true"), "{code}");
}

/// A JSX Vapor render is a closure invoked without an instance context, so the
/// ref setter cannot come from `_ctx`.
#[test]
fn vapor_refs_create_their_own_setter() {
    let code = component_code(
        "const el = ref();\nconst App = () => <div><p ref={el}/><i ref=\"named\"/></div>;",
        JsxOutputMode::Vapor,
    );
    let setters: Vec<&str> = code
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.split_once(" = ")
                .is_some_and(|(head, _)| head == "const _setRef")
        })
        .collect();
    assert_eq!(
        setters,
        ["const _setRef = _createTemplateRefSetter()"],
        "{code}"
    );
}

/// Outside a loop a bare `ref_for` is still the authored array flag, so the
/// Vapor setter receives it.
#[test]
fn bare_ref_for_outside_a_loop_sets_the_vapor_flag() {
    let code = component_code(
        "const els = ref([]);\nconst App = () => <div><li ref={els} ref_for /></div>;",
        JsxOutputMode::Vapor,
    );
    let calls: Vec<&str> = code
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.split_once('(')
                .is_some_and(|(head, _)| head == "_setRef")
        })
        .collect();
    assert_eq!(calls.len(), 1, "{code}");
    assert_eq!(
        calls
            .first()
            .and_then(|call| call.rsplit_once(", "))
            .map(|(_, flag)| flag),
        Some("true)"),
        "{code}"
    );
}
