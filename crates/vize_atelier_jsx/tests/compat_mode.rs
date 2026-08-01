//! The opt-in `@vue/babel-plugin-jsx` compatibility switch (#3391).
//!
//! These tests pin the switch's contract and each compatibility behavior: the
//! mode is off by default, behaviors land one inventory row at a time, and
//! asking for it under Vapor output is rejected rather than silently ignored.
//!
//! The "default output is unchanged" test is the important one — flipping the
//! default would be a silent compatibility break for every existing Vize user.

use vize_atelier_jsx::{JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx};
use vize_carton::Bump;

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
    let bump = Bump::new();
    let config = JsxCompileConfig {
        default_mode: mode,
        compat,
        ..Default::default()
    };
    let out = compile_jsx(&bump, SOURCE, JsxLang::Jsx, &config);
    out.module_code().to_string()
}

fn diagnostics(compat: JsxCompatMode, mode: JsxOutputMode) -> Vec<String> {
    let bump = Bump::new();
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
    let bump = Bump::new();
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
        let bump = Bump::new();
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
