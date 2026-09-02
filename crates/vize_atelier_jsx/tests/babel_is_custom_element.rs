//! Babel-compatible `isCustomElement` classification (#3391).

use std::sync::atomic::{AtomicUsize, Ordering};

use vize_atelier_jsx::{
    BabelIsCustomElement, BabelJsxCustomizations, BabelJsxOptions, JsxCompatMode, JsxCompileConfig,
    JsxLang, JsxOutputMode, compile_jsx, compile_jsx_with_babel_customizations,
};
use vize_s0::{Allocator, FxHashSet, String};

static PREDICATE_CALLS: AtomicUsize = AtomicUsize::new(0);

fn babel_vdom() -> JsxCompileConfig {
    JsxCompileConfig {
        compat: JsxCompatMode::Babel,
        ..Default::default()
    }
}

fn compile(
    source: &str,
    config: &JsxCompileConfig,
    is_custom_element: Option<&BabelIsCustomElement>,
) -> (String, std::vec::Vec<String>) {
    let bump = Allocator::new();
    let output = match is_custom_element {
        Some(is_custom_element) => compile_jsx_with_babel_customizations(
            &bump,
            source,
            JsxLang::Jsx,
            config,
            &BabelJsxOptions::default(),
            BabelJsxCustomizations {
                is_custom_element: Some(is_custom_element),
                ..Default::default()
            },
        ),
        None => compile_jsx(&bump, source, JsxLang::Jsx, config),
    };
    let diagnostics = output
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.clone())
        .collect();
    (output.module_code(), diagnostics)
}

#[test]
fn matching_unbound_tag_uses_a_string_vnode_tag() {
    let is_custom_element = |tag: &str| tag == "MyEl";
    let (module, diagnostics) = compile(
        "const A = () => <MyEl foo={1}/>;",
        &babel_vdom(),
        Some(&is_custom_element),
    );

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(
        module.contains("_createElementBlock(\"MyEl\", { foo: 1 })"),
        "{module}"
    );
    assert!(!module.contains("_resolveComponent"), "{module}");
}

#[test]
fn matching_hyphenated_tag_stays_an_intrinsic_element() {
    // A hyphenated tag is the shape the element transform would otherwise
    // promote to a component, so it is the case that proves the lowerer's
    // verdict survives the transform.
    let is_custom_element = |tag: &str| tag == "my-el";
    let (custom, diagnostics) = compile(
        "const A = () => <my-el foo={1}/>;",
        &babel_vdom(),
        Some(&is_custom_element),
    );

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(
        custom.contains("_createElementBlock(\"my-el\", { foo: 1 })"),
        "{custom}"
    );
    assert!(!custom.contains("_resolveComponent"), "{custom}");

    let (baseline, _) = compile("const A = () => <my-el foo={1}/>;", &babel_vdom(), None);
    assert!(
        baseline.contains("_resolveComponent(\"my-el\")"),
        "{baseline}"
    );
}

#[test]
fn predicate_is_inert_outside_babel_vdom() {
    let source = "const A = () => <MyEl foo={1}/>;";
    let is_custom_element = |tag: &str| {
        PREDICATE_CALLS.fetch_add(1, Ordering::Relaxed);
        tag == "MyEl"
    };

    for config in [
        JsxCompileConfig::default(),
        JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ssr: true,
            ..Default::default()
        },
        JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            default_mode: JsxOutputMode::Vapor,
            ..Default::default()
        },
    ] {
        PREDICATE_CALLS.store(0, Ordering::Relaxed);
        assert_eq!(
            compile(source, &config, Some(&is_custom_element)),
            compile(source, &config, None)
        );
        assert_eq!(PREDICATE_CALLS.load(Ordering::Relaxed), 0);
    }
}

#[test]
fn unmatched_predicate_preserves_the_babel_baseline() {
    let source = "const A = () => <MyEl foo={1}/>;";
    let is_custom_element = |tag: &str| tag == "OtherEl";
    assert_eq!(
        compile(source, &babel_vdom(), Some(&is_custom_element)),
        compile(source, &babel_vdom(), None)
    );
}

#[test]
fn lexical_bindings_win_before_a_matching_predicate() {
    let is_custom_element = |tag: &str| tag == "MyEl";
    for source in [
        "import MyEl from 'my-el'; const A = () => <MyEl foo={1}/>;",
        "const MyEl = Other; const A = () => <MyEl foo={1}/>;",
        "let MyEl = Other; const A = () => <MyEl foo={1}/>;",
        "var MyEl = Other; const A = () => <MyEl foo={1}/>;",
        "const A = (MyEl) => <MyEl foo={1}/>;",
    ] {
        let (module, diagnostics) = compile(source, &babel_vdom(), Some(&is_custom_element));
        assert!(diagnostics.is_empty(), "{source}\n{diagnostics:?}");
        assert!(
            module.contains("_resolveDynamicComponent(MyEl)"),
            "{source}\n{module}"
        );
        assert!(!module.contains("_resolveComponent(\"MyEl\")"), "{module}");
        assert!(!module.contains("_createElementBlock(\"MyEl\""), "{module}");
    }

    let source = "const A = () => <MyEl/>; const B = (MyEl) => <MyEl/>;";
    let (module, diagnostics) = compile(source, &babel_vdom(), Some(&is_custom_element));
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(module.contains("_createElementBlock(\"MyEl\""), "{module}");
    assert!(
        module.contains("_resolveDynamicComponent(MyEl)"),
        "{module}"
    );
}

#[test]
fn custom_element_classification_controls_children_and_v_model() {
    let is_my_el = |tag: &str| tag == "MyEl";
    let (component, _) = compile("const A = () => <MyEl>x</MyEl>;", &babel_vdom(), None);
    let (custom, diagnostics) = compile(
        "const A = () => <MyEl>x</MyEl>;",
        &babel_vdom(),
        Some(&is_my_el),
    );
    assert!(
        component.contains("default: _withCtx(() => ["),
        "{component}"
    );
    assert!(
        custom.contains("_createElementBlock(\"MyEl\", null, \"x\")"),
        "{custom}"
    );
    assert!(!custom.contains("_withCtx"), "{custom}");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    let is_dashed = |tag: &str| tag == "my-el";
    let (custom, diagnostics) = compile(
        "const A = () => <my-el v-model={value}/>;",
        &babel_vdom(),
        Some(&is_dashed),
    );
    assert!(
        custom.contains("_createElementBlock(\"my-el\", {"),
        "{custom}"
    );
    assert!(custom.contains("\"onUpdate:modelValue\""), "{custom}");
    assert!(!custom.contains("_resolveComponent"), "{custom}");
    assert!(!custom.contains("\"modelValue\""), "{custom}");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn predicate_skips_non_identifier_and_builtin_tag_shapes() {
    let source = concat!(
        "const A = () => <div/>;",
        "const B = () => <Ns.Item/>;",
        "const C = () => <svg:circle/>;",
        "const D = () => <Fragment/>;",
    );
    let is_custom_element = |_: &str| true;
    assert_eq!(
        compile(source, &babel_vdom(), Some(&is_custom_element)),
        compile(source, &babel_vdom(), None)
    );
}

#[test]
fn captured_predicate_composes_with_other_babel_options() {
    let mut tags = FxHashSet::default();
    tags.insert(String::from("MyEl"));
    let is_custom_element = move |tag: &str| tags.contains(tag);
    let bump = Allocator::new();
    let output = compile_jsx_with_babel_customizations(
        &bump,
        "const A = () => <MyEl class=\"a\" {...p} class={c} on={{ click: h }}/>;",
        JsxLang::Jsx,
        &babel_vdom(),
        &BabelJsxOptions { transform_on: true },
        BabelJsxCustomizations {
            pragma: Some("factory.h"),
            merge_props: false,
            is_custom_element: Some(&is_custom_element),
            ..Default::default()
        },
    );
    let module = output.module_code();

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(module.contains("factory.h(\"MyEl\", {"), "{module}");
    assert!(module.contains("...p"), "{module}");
    assert!(module.contains("_transformOn({ click: h })"), "{module}");
    assert!(!module.contains("_mergeProps"), "{module}");
    assert!(!module.contains("_resolveComponent"), "{module}");
}
