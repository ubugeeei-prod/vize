//! The opt-in `@vue/babel-plugin-jsx` compatibility switch (#3391).
//!
//! These tests pin the switch's *contract* rather than any particular compat
//! behavior: the mode is off by default, turning it on never changes VDOM
//! output yet (behaviors land one PR per inventory row), and asking for it under
//! Vapor output is rejected with a diagnostic rather than silently ignored.
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
fn babel_compat_vdom_output_is_unchanged_for_now() {
    // No inventory row has been closed yet, so compat mode is still a no-op on
    // VDOM output. This test is what the first behavioral PR flips: when a
    // divergence is closed, these two stop being equal and the assertion here
    // becomes the pin for the new compat-only output.
    assert_eq!(
        module_code(JsxCompatMode::Babel, JsxOutputMode::Vdom),
        module_code(JsxCompatMode::Native, JsxOutputMode::Vdom)
    );
    assert_eq!(
        diagnostics(JsxCompatMode::Babel, JsxOutputMode::Vdom),
        Vec::<String>::new()
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
