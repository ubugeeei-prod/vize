//! The opt-in `@vue/babel-plugin-jsx` compatibility switch (#3391).
//!
//! These tests pin the switch's contract and each compatibility behavior: the
//! mode is off by default, behaviors land one inventory row at a time, and
//! asking for it under Vapor output is rejected rather than silently ignored.
//!
//! The "default output is unchanged" test is the important one — flipping the
//! default would be a silent compatibility break for every existing Vize user.

use oxc_allocator::Allocator;
use vize_atelier_jsx::{
    BabelJsxOptions, JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx,
    compile_jsx_with_babel_options, parse_module,
};

/// A representative module touching elements, props, interpolation, control
/// flow, directives, and slots, so "output is unchanged" is a broad claim rather
/// than a single-node one.
const SOURCE: &str = concat!(
    "const A = () => <div class={c} id=\"x\">{count}</div>;\n",
    "const B = () => <ul>{items.map((i) => <li key={i}>{i}</li>)}</ul>;\n",
    "const C = () => <input v-model={val}/>;\n",
    "const D = () => <Comp>{{ header: () => <h1>h</h1> }}</Comp>;\n",
);

fn module_code(compat: JsxCompatMode, mode: JsxOutputMode) -> String {
    compile_module(SOURCE, compat, mode)
}

fn compile_module(source: &str, compat: JsxCompatMode, mode: JsxOutputMode) -> String {
    let bump = vize_s0::Allocator::new();
    let config = JsxCompileConfig {
        default_mode: mode,
        compat,
        ..Default::default()
    };
    let out = compile_jsx(&bump, source, JsxLang::Jsx, &config);
    out.module_code().to_string()
}

fn diagnostics(compat: JsxCompatMode, mode: JsxOutputMode) -> Vec<String> {
    let bump = vize_s0::Allocator::new();
    let config = JsxCompileConfig {
        default_mode: mode,
        compat,
        ..Default::default()
    };
    let out = compile_jsx(&bump, SOURCE, JsxLang::Jsx, &config);
    out.diagnostics
        .iter()
        .map(|diagnostic| format!("{:?}: {}", diagnostic.severity, diagnostic.message))
        .collect()
}

fn compile_with_transform_on(source: &str, compat: JsxCompatMode, mode: JsxOutputMode) -> String {
    let bump = vize_s0::Allocator::new();
    compile_jsx_with_babel_options(
        &bump,
        source,
        JsxLang::Jsx,
        &JsxCompileConfig {
            default_mode: mode,
            compat,
            ..Default::default()
        },
        &BabelJsxOptions { transform_on: true },
    )
    .module_code()
    .to_string()
}

#[test]
fn compat_is_off_by_default() {
    assert_eq!(
        JsxCompileConfig::default().compat,
        JsxCompatMode::Native,
        "turning compat on by default would silently change output for every existing project"
    );
}

#[test]
fn default_config_output_equals_explicit_native() {
    // The switch must be inert unless asked for: `Default::default()` and an
    // explicit `Native` must produce the same module, byte for byte.
    let bump = vize_s0::Allocator::new();
    let implicit = compile_jsx(&bump, SOURCE, JsxLang::Jsx, &JsxCompileConfig::default());
    assert_eq!(
        implicit.module_code().to_string(),
        module_code(JsxCompatMode::Native, JsxOutputMode::Vdom)
    );
    assert!(implicit.diagnostics.is_empty());
}

#[test]
fn babel_compat_vdom_remains_error_free() {
    assert_eq!(
        diagnostics(JsxCompatMode::Babel, JsxOutputMode::Vdom),
        Vec::<String>::new()
    );
}

#[test]
fn babel_compat_emits_true_for_a_valueless_attribute_only_when_opted_in() {
    let compile = |compat| {
        let bump = vize_s0::Allocator::new();
        compile_jsx(
            &bump,
            "const A = () => <input disabled/>;",
            JsxLang::Jsx,
            &JsxCompileConfig {
                compat,
                ..Default::default()
            },
        )
        .module_code()
        .to_string()
    };

    let native = compile(JsxCompatMode::Native);
    let babel = compile(JsxCompatMode::Babel);
    assert!(native.contains("{ disabled: \"\" }"), "{native}");
    assert!(babel.contains("{ disabled: true }"), "{babel}");
    assert_ne!(native, babel);
}

#[test]
fn babel_compat_assigns_v_text_raw_only_when_opted_in() {
    let source = "const A = () => <div v-text={value}/>;";
    let native = compile_module(source, JsxCompatMode::Native, JsxOutputMode::Vdom);
    let babel = compile_module(source, JsxCompatMode::Babel, JsxOutputMode::Vdom);

    assert!(
        native.contains("textContent: _toDisplayString(value)"),
        "{native}"
    );
    assert!(
        native.contains("toDisplayString as _toDisplayString"),
        "{native}"
    );
    assert!(babel.contains("textContent: value"), "{babel}");
    assert!(!babel.contains("toDisplayString"), "{babel}");
    assert_ne!(native, babel);
}

#[test]
fn babel_compat_rewrites_xlink_href_across_prop_shapes_only_when_opted_in() {
    let source = concat!(
        "const A = () => <svg>",
        "<use xlinkHref=\"#a\"/>",
        "<use xlinkHref={href}/>",
        "<use xlink:href=\"#b\"/>",
        "</svg>;\n",
        "const C = () => <Comp xlinkHref=\"#c\"/>;\n",
        "const B = () => <svg>{ok ? <use xlinkHref=\"#d\"/> : ",
        "items.map(id => <use xlinkHref={id}/>)}</svg>;",
    );
    let native = compile_module(source, JsxCompatMode::Native, JsxOutputMode::Vdom);
    let babel = compile_module(source, JsxCompatMode::Babel, JsxOutputMode::Vdom);

    assert!(native.contains("{ xlinkHref: \"#a\" }"), "{native}");
    assert!(native.contains("{ xlinkHref: href }"), "{native}");
    assert!(native.contains("{ \"xlink:href\": \"#b\" }"), "{native}");
    assert!(!babel.contains("xlinkHref"), "{babel}");
    assert!(babel.contains("{ \"xlink:href\": \"#a\" }"), "{babel}");
    assert!(babel.contains("{ \"xlink:href\": href }"), "{babel}");
    assert!(babel.contains("{ \"xlink:href\": \"#b\" }"), "{babel}");
    assert!(babel.contains("{ \"xlink:href\": \"#c\" }"), "{babel}");
    assert!(babel.contains("\"xlink:href\": \"#d\""), "{babel}");
    assert!(babel.contains("\"xlink:href\": id"), "{babel}");
    assert_ne!(native, babel);
}

#[test]
fn babel_compat_keeps_lone_element_expressions_raw_without_a_text_flag() {
    let source = concat!(
        "const A = () => <div>{t}</div>;\n",
        "const B = () => <div class={c} id={i}>{t}</div>;",
    );
    let bump = vize_s0::Allocator::new();
    let implicit_native = compile_jsx(&bump, source, JsxLang::Jsx, &JsxCompileConfig::default())
        .module_code()
        .to_string();
    let explicit_native = compile_module(source, JsxCompatMode::Native, JsxOutputMode::Vdom);
    let babel = compile_module(source, JsxCompatMode::Babel, JsxOutputMode::Vdom);

    assert_eq!(implicit_native, explicit_native);
    assert_eq!(explicit_native.matches("_toDisplayString(t)").count(), 2);
    assert!(
        explicit_native.contains("1 /* TEXT */"),
        "{explicit_native}"
    );
    assert!(
        explicit_native.contains("11 /* TEXT, CLASS, PROPS */, [\"id\"]"),
        "{explicit_native}"
    );

    assert_eq!(babel.matches("[\n    t\n  ]").count(), 2, "{babel}");
    assert!(!babel.contains("toDisplayString"), "{babel}");
    assert!(!babel.contains("1 /* TEXT */"), "{babel}");
    assert!(babel.contains("10 /* CLASS, PROPS */, [\"id\"]"), "{babel}");

    let allocator = Allocator::default();
    let parsed = parse_module(&allocator, &babel, JsxLang::Jsx);
    assert!(
        parsed.diagnostics.is_empty(),
        "{:?}\n{babel}",
        parsed.diagnostics
    );
}

#[test]
fn transform_on_is_off_by_default_and_inert_in_native_mode() {
    let source = "const A = () => <button on={{ click: h }}/>;";
    let native = compile_module(source, JsxCompatMode::Native, JsxOutputMode::Vdom);
    let native_with_option =
        compile_with_transform_on(source, JsxCompatMode::Native, JsxOutputMode::Vdom);
    let babel_without_option = compile_module(source, JsxCompatMode::Babel, JsxOutputMode::Vdom);

    assert!(!BabelJsxOptions::default().transform_on);
    assert_eq!(native_with_option, native);
    assert!(
        !native.contains("@vue/babel-helper-vue-transform-on"),
        "{native}"
    );
    assert!(
        babel_without_option.contains("on: { click: h }"),
        "{babel_without_option}"
    );
    assert!(
        !babel_without_option.contains("@vue/babel-helper-vue-transform-on"),
        "{babel_without_option}"
    );
}

#[test]
fn babel_transform_on_wraps_exact_on_props_in_authored_merge_order() {
    let source = concat!(
        "const A = () => <button id=\"first\" ",
        "on={{ click: h }} {...attrs} nativeOn={native} ",
        "only={other} nativeon={lower} onClick={direct} on-call={dashed}/>;",
    );
    let output = compile_with_transform_on(source, JsxCompatMode::Babel, JsxOutputMode::Vdom);

    assert_eq!(
        output.matches("@vue/babel-helper-vue-transform-on").count(),
        1
    );
    assert_eq!(output.matches("_transformOn(").count(), 2, "{output}");
    assert!(output.contains("_transformOn({ click: h })"), "{output}");
    assert!(output.contains("_transformOn(native)"), "{output}");
    assert!(output.contains("only: other"), "{output}");
    assert!(output.contains("nativeon: lower"), "{output}");
    assert!(output.contains("onClick: direct"), "{output}");
    assert!(output.contains("\"on-call\": dashed"), "{output}");

    let first = output.find("id: \"first\"").unwrap();
    let on = output.find("_transformOn({ click: h })").unwrap();
    let spread = output.find("attrs").unwrap();
    let native_on = output.find("_transformOn(native)").unwrap();
    let near_misses = output.find("only: other").unwrap();
    assert!(first < on && on < spread && spread < native_on && native_on < near_misses);
}

#[test]
fn babel_transform_on_keeps_generated_modules_parseable_and_helper_bindings_unique() {
    let source = concat!(
        "const _transformOn = existing; ",
        "const A = () => <button on/>; ",
        "const B = () => <button nativeOn={{}}/>; ",
        "const C = () => <button on=\"tap\"/>; ",
        "const D = () => <button on={}/>;",
    );
    let output = compile_with_transform_on(source, JsxCompatMode::Babel, JsxOutputMode::Vdom);

    assert!(
        output.contains("import _transformOn_ from \"@vue/babel-helper-vue-transform-on\""),
        "{output}"
    );
    assert!(output.contains("_transformOn_(true)"), "{output}");
    assert!(output.contains("_transformOn_({})"), "{output}");
    assert!(output.contains("_transformOn_(\"tap\")"), "{output}");
    assert_eq!(
        output.matches("@vue/babel-helper-vue-transform-on").count(),
        1
    );

    let allocator = Allocator::default();
    let parsed = parse_module(&allocator, &output, JsxLang::Jsx);
    assert!(
        parsed.diagnostics.is_empty(),
        "{:?}\n{output}",
        parsed.diagnostics
    );
}

#[test]
fn babel_transform_on_does_not_leak_into_vapor_output() {
    let source = "const A = () => { 'use vue:vapor'; return <button on={{ click: h }}/>; };";
    let output = compile_with_transform_on(source, JsxCompatMode::Babel, JsxOutputMode::Vdom);
    assert!(!output.contains("_transformOn"), "{output}");
    assert!(
        !output.contains("@vue/babel-helper-vue-transform-on"),
        "{output}"
    );
}

#[test]
fn babel_compat_under_vapor_is_diagnosed_once_per_component() {
    // `@vue/babel-plugin-jsx` has no Vapor output shape, so the combination is
    // rejected rather than quietly producing Vize-shaped Vapor code. One
    // diagnostic per render root: the conflict applies to each component.
    let expected: Vec<String> = (0..4)
        .map(|_| {
            "Error: compiler.jsxCompat: \"babel\" is not supported with Vapor output: \
             @vue/babel-plugin-jsx has no Vapor equivalent. Use jsxMode \"vdom\" for \
             babel compatibility, or drop jsxCompat to use Vize's own Vapor semantics."
                .to_string()
        })
        .collect();
    assert_eq!(
        diagnostics(JsxCompatMode::Babel, JsxOutputMode::Vapor),
        expected
    );
}

#[test]
fn native_under_vapor_is_not_diagnosed() {
    // Guards the diagnostic against firing on the default configuration.
    assert_eq!(
        diagnostics(JsxCompatMode::Native, JsxOutputMode::Vapor),
        Vec::<String>::new()
    );
}
