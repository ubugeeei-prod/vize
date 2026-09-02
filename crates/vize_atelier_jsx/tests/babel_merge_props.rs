//! Babel JSX `mergeProps` option compatibility.

use oxc_allocator::Allocator;
use vize_atelier_jsx::{
    BabelJsxOptions, JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx,
    compile_jsx_with_babel_merge_props, compile_jsx_with_babel_pragma_and_merge_props,
    parse_module,
};

fn compile_module(source: &str, compat: JsxCompatMode, mode: JsxOutputMode) -> String {
    let bump = vize_s0::Allocator::new();
    let config = JsxCompileConfig {
        default_mode: mode,
        compat,
        ..Default::default()
    };
    compile_jsx(&bump, source, JsxLang::Jsx, &config)
        .module_code()
        .to_string()
}

fn compile_with_merge_props(
    source: &str,
    compat: JsxCompatMode,
    mode: JsxOutputMode,
    merge_props: bool,
) -> (String, Vec<String>) {
    let bump = vize_s0::Allocator::new();
    let out = compile_jsx_with_babel_merge_props(
        &bump,
        source,
        JsxLang::Jsx,
        &JsxCompileConfig {
            default_mode: mode,
            compat,
            ..Default::default()
        },
        &BabelJsxOptions::default(),
        merge_props,
    );
    let diagnostics = out
        .diagnostics
        .iter()
        .map(|diagnostic| format!("{:?}: {}", diagnostic.severity, diagnostic.message))
        .collect();
    (out.module_code().to_string(), diagnostics)
}

#[test]
fn false_uses_object_spread_and_javascript_overwrite_order() {
    let target = "const A = () => <div class=\"a\" {...p} class={c}/>;";
    let (first, diagnostics) =
        compile_with_merge_props(target, JsxCompatMode::Babel, JsxOutputMode::Vdom, false);
    let (second, second_diagnostics) =
        compile_with_merge_props(target, JsxCompatMode::Babel, JsxOutputMode::Vdom, false);

    assert_eq!(first, second, "the option must be deterministic");
    assert_eq!(diagnostics, second_diagnostics);
    assert!(diagnostics.is_empty(), "{diagnostics:?}\n{first}");
    assert!(!first.contains("_mergeProps"), "{first}");
    assert!(!first.contains("_normalizeClass"), "{first}");
    let static_class = first.find("class: \"a\"").unwrap();
    let spread = first.find("...p").unwrap();
    let dynamic_class = first.find("class: c").unwrap();
    assert!(static_class < spread && spread < dynamic_class, "{first}");

    let (spread_only, diagnostics) = compile_with_merge_props(
        "const A = () => <div {...p}/>;",
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        false,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}\n{spread_only}");
    assert!(
        spread_only.contains("createElementBlock(\"div\", p, null"),
        "{spread_only}"
    );
    assert!(!spread_only.contains("...p"), "{spread_only}");

    let (multiple, diagnostics) = compile_with_merge_props(
        "const A = () => <Comp {...a} id={x} {...b}/>;",
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        false,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}\n{multiple}");
    let first_spread = multiple.find("...a").unwrap();
    let middle = multiple.find("id: x").unwrap();
    let second_spread = multiple.find("...b").unwrap();
    assert!(
        first_spread < middle && middle < second_spread,
        "{multiple}"
    );

    let (parenthesized, diagnostics) = compile_with_merge_props(
        "const A = () => <button {...(ok ? a : b)} id=\"x\"/>;",
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        false,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}\n{parenthesized}");
    assert!(parenthesized.contains("...(ok ? a : b)"), "{parenthesized}");
    assert_parseable(&parenthesized);

    let (duplicates, diagnostics) = compile_with_merge_props(
        concat!(
            "const A = () => <button class=\"a\" class={c}/>;\n",
            "const B = () => <button id=\"a\" id=\"b\"/>;\n",
            "const C = () => <button onClick={a} onClick={b}/>;",
        ),
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        false,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}\n{duplicates}");
    assert_eq!(
        duplicates.matches("class: \"a\"").count(),
        1,
        "{duplicates}"
    );
    assert_eq!(duplicates.matches("class: c").count(), 1, "{duplicates}");
    assert!(duplicates.contains("id: \"a\""), "{duplicates}");
    assert!(duplicates.contains("id: \"b\""), "{duplicates}");
    assert_eq!(duplicates.matches("onClick:").count(), 2, "{duplicates}");
    assert!(!duplicates.contains("onClick: ["), "{duplicates}");
    assert_parseable(&duplicates);
}

#[test]
fn option_is_inert_by_default_in_native_vapor_and_ssr() {
    let source = "const A = () => <div class=\"a\" {...p} class={c}/>;";

    let babel_default = compile_module(source, JsxCompatMode::Babel, JsxOutputMode::Vdom);
    let (babel_true, diagnostics) =
        compile_with_merge_props(source, JsxCompatMode::Babel, JsxOutputMode::Vdom, true);
    assert!(diagnostics.is_empty());
    assert_eq!(babel_true, babel_default);
    assert!(babel_default.contains("_mergeProps"), "{babel_default}");

    let native = compile_module(source, JsxCompatMode::Native, JsxOutputMode::Vdom);
    let (native_false, diagnostics) =
        compile_with_merge_props(source, JsxCompatMode::Native, JsxOutputMode::Vdom, false);
    assert!(diagnostics.is_empty());
    assert_eq!(native_false, native);

    let (vapor_true, vapor_true_diagnostics) =
        compile_with_merge_props(source, JsxCompatMode::Babel, JsxOutputMode::Vapor, true);
    let (vapor_false, vapor_false_diagnostics) =
        compile_with_merge_props(source, JsxCompatMode::Babel, JsxOutputMode::Vapor, false);
    assert_eq!(vapor_false, vapor_true);
    assert_eq!(vapor_false_diagnostics, vapor_true_diagnostics);
    assert_eq!(vapor_false_diagnostics.len(), 1);

    let compile_ssr = |merge_props| {
        let bump = vize_s0::Allocator::new();
        let out = compile_jsx_with_babel_merge_props(
            &bump,
            source,
            JsxLang::Jsx,
            &JsxCompileConfig {
                compat: JsxCompatMode::Babel,
                ssr: true,
                ..Default::default()
            },
            &BabelJsxOptions::default(),
            merge_props,
        );
        (
            out.module_code().to_string(),
            out.diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.to_string())
                .collect::<Vec<_>>(),
        )
    };
    assert_eq!(compile_ssr(false), compile_ssr(true));
}

#[test]
fn false_composes_with_a_custom_pragma() {
    let bump = vize_s0::Allocator::new();
    let out = compile_jsx_with_babel_pragma_and_merge_props(
        &bump,
        "const A = () => <div class=\"a\" {...p} class={c}/>;",
        JsxLang::Jsx,
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
        &BabelJsxOptions::default(),
        Some("h"),
        false,
    );
    let output = out.module_code();
    assert!(
        out.diagnostics.is_empty(),
        "{:?}\n{output}",
        out.diagnostics
    );
    assert!(!output.contains("from \"vue\""), "{output}");
    assert!(!output.contains("_mergeProps"), "{output}");
    assert!(output.contains("h(\"div\", {"), "{output}");
    assert!(output.contains("...p"), "{output}");
}

fn assert_parseable(output: &str) {
    let allocator = Allocator::default();
    let parsed = parse_module(&allocator, output, JsxLang::Jsx);
    assert!(
        parsed.diagnostics.is_empty(),
        "{:?}\n{output}",
        parsed.diagnostics
    );
}
