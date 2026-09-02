//! Babel-compatible custom vnode factory coverage (#3391).

use oxc_allocator::Allocator;
use vize_atelier_jsx::{
    BabelJsxOptions, JsxCompatMode, JsxCompileConfig, JsxCompileOutput, JsxLang, JsxOutputMode,
    VdomCompileOptions, compile_jsx_with_babel_options, compile_jsx_with_babel_pragma,
    parse_module,
};

fn compile(
    bump: &vize_s0::Allocator,
    source: &str,
    config: &JsxCompileConfig,
    pragma: Option<&str>,
) -> JsxCompileOutput {
    compile_jsx_with_babel_pragma(
        bump,
        source,
        JsxLang::Jsx,
        config,
        &BabelJsxOptions::default(),
        pragma,
    )
}

#[test]
fn pragma_is_empty_by_default_and_inert_outside_babel_vdom() {
    let source = "const A = () => <div><span/></div>;";
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
        let bump = vize_s0::Allocator::new();
        let baseline = compile_jsx_with_babel_options(
            &bump,
            source,
            JsxLang::Jsx,
            &config,
            &BabelJsxOptions::default(),
        );
        let with_pragma = compile(&bump, source, &config, Some("h"));
        assert_eq!(with_pragma.module_code(), baseline.module_code());
        assert_eq!(with_pragma.diagnostics, baseline.diagnostics);
    }

    let config = JsxCompileConfig {
        compat: JsxCompatMode::Babel,
        ..Default::default()
    };
    let bump = vize_s0::Allocator::new();
    let baseline = compile(&bump, source, &config, None);
    for near_miss in [Some(""), Some("  \n")] {
        let output = compile(&bump, source, &config, near_miss);
        assert_eq!(output.module_code(), baseline.module_code());
        assert!(output.diagnostics.is_empty());
    }
}

#[test]
fn babel_pragma_routes_every_vnode_shape_through_the_custom_factory() {
    let source = concat!(
        "const A = () => <div><span/><Comp/></div>;",
        "const B = () => <>{ok ? <i/> : <b/>}</>;",
    );
    let bump = vize_s0::Allocator::new();
    let output = compile(
        &bump,
        source,
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
        Some("factory.h"),
    );
    let module = output.module_code();

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(module.matches("factory.h(").count() >= 5, "{module}");
    for default_helper in [
        "_createVNode",
        "_createElementVNode",
        "_createBlock",
        "_createElementBlock",
    ] {
        assert!(
            !module.contains(default_helper),
            "{default_helper}: {module}"
        );
    }
    assert_eq!(module.matches("const _openBlock = () => {}").count(), 1);

    let allocator = Allocator::default();
    let parsed = parse_module(&allocator, module.as_str(), JsxLang::Jsx);
    assert!(
        parsed.diagnostics.is_empty(),
        "{:?}\n{module}",
        parsed.diagnostics
    );
}

#[test]
fn babel_pragma_keeps_callee_precedence_for_non_identifier_expressions() {
    // The pragma accepts any valid JavaScript expression, so anything that is
    // not a plain identifier or dotted member chain has to stay parenthesized
    // to remain the callee of the vnode call.
    for (pragma, callee) in [
        ("left || right", "(left || right\n)("),
        ("cond ? a : b", "(cond ? a : b\n)("),
        ("(setup(), h)", "((setup(), h)\n)("),
        (
            "left || right // fallback",
            "(left || right // fallback\n)(",
        ),
    ] {
        let bump = vize_s0::Allocator::new();
        let output = compile(
            &bump,
            "const A = () => <div>{ok ? <i/> : <b/>}</div>;",
            &JsxCompileConfig {
                compat: JsxCompatMode::Babel,
                ..Default::default()
            },
            Some(pragma),
        );
        let module = output.module_code();

        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        assert!(module.contains(callee), "{pragma}: {module}");
        let allocator = Allocator::default();
        let parsed = parse_module(&allocator, module.as_str(), JsxLang::Jsx);
        assert!(
            parsed.diagnostics.is_empty(),
            "{pragma}: {:?}\n{module}",
            parsed.diagnostics
        );
    }
}

#[test]
fn babel_pragma_removes_the_vue_import_when_no_other_helper_is_needed() {
    let bump = vize_s0::Allocator::new();
    let output = compile(
        &bump,
        "const A = () => <div/>;",
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
        Some("h"),
    );
    let module = output.module_code();
    assert!(module.contains("h(\"div\")"), "{module}");
    assert!(!module.contains("from \"vue\""), "{module}");
    assert!(!module.contains("createVNode"), "{module}");
}

#[test]
fn babel_pragma_composes_with_transform_on_and_preserves_source_maps() {
    let bump = vize_s0::Allocator::new();
    let output = compile_jsx_with_babel_pragma(
        &bump,
        "const A = () => <button on={{ click: handler }}/>;",
        JsxLang::Jsx,
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            vdom: VdomCompileOptions {
                source_map: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &BabelJsxOptions { transform_on: true },
        Some("h"),
    );
    let module = output.module_code();
    assert!(module.contains("h(\"button\""), "{module}");
    assert!(
        module.contains("_transformOn({ click: handler })"),
        "{module}"
    );
    assert_eq!(
        module.matches("@vue/babel-helper-vue-transform-on").count(),
        1
    );
    let map = output
        .source_map()
        .expect("pragma retains the generated source map");
    let value: serde_json::Value = serde_json::from_str(map).expect("valid source-map JSON");
    assert_eq!(value["version"], 3);
}

#[test]
fn babel_pragma_does_not_leak_into_a_vapor_component_override() {
    let bump = vize_s0::Allocator::new();
    let output = compile(
        &bump,
        concat!(
            "const A = () => <div/>;",
            "const B = () => { 'use vue:vapor'; return <span/>; };",
        ),
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
        Some("h"),
    );
    let module = output.module_code();
    assert!(module.contains("h(\"div\")"), "{module}");
    assert!(!module.contains("h(\"span\")"), "{module}");
    assert_eq!(output.diagnostics.len(), 1);
    assert!(output.diagnostics[0].message.contains("Vapor output"));
}

#[test]
fn invalid_pragma_is_diagnosed_without_emitting_broken_javascript() {
    let bump = vize_s0::Allocator::new();
    let output = compile(
        &bump,
        "const A = () => <div/>;",
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
        Some("h; unexpected"),
    );
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].message,
        "Babel JSX pragma must be a valid JavaScript expression"
    );
    let module = output.module_code();
    assert!(module.contains("_createElementBlock"), "{module}");
    let allocator = Allocator::default();
    assert!(
        parse_module(&allocator, module.as_str(), JsxLang::Jsx)
            .diagnostics
            .is_empty()
    );
}
