//! `v-slots` values that `@vue/babel-plugin-jsx` forwards rather than expands
//! (#3391, oracle row `errors/v_slots_not_object`).
//!
//! Babel never validates a `v-slots` value: it hands whatever it finds to
//! `createVNode` as the children argument, so `<B v-slots={1}/>` compiles to
//! `createVNode(B, null, 1)`. Vize rejects that by default, because for a
//! component a slots object is either an object literal to expand or an opaque
//! expression to forward — never a primitive — and silently rendering nothing is
//! the failure shape #3418 exists to remove.
//!
//! Opt-in Babel VDOM mode reproduces the pass-through for the literals whose
//! source text is already valid JavaScript in children position. Everything
//! babel would forward as a *container* (arrays, interpolated template literals,
//! raw JSX, functions, sequences) stays diagnosed rather than emitted as a
//! malformed module.

use vize_atelier_jsx::{JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx};
use vize_s0::Allocator;

/// The diagnostic every non-slots-object `v-slots` value produces.
fn not_a_slots_object(value: &str) -> String {
    format!(
        "v-slots value `{value}` is not a slots object: babel forwards it as the component's \
         children, which leaves the component with no slots. Write the slots inline, e.g. \
         v-slots={{{{ default: () => <div/> }}}}, or forward a slots object, e.g. \
         v-slots={{slots}}."
    )
}

/// A `B` component render whose children argument is `children`, or a bare
/// `createBlock(B)` when the value was dropped.
fn render_module(children: Option<&str>) -> String {
    let call = match children {
        Some(children) => {
            format!("_createBlock(_component_B, null, {children}, 1024 /* DYNAMIC_SLOTS */)")
        }
        None => "_createBlock(_component_B)".to_string(),
    };
    format!(
        concat!(
            "import {{ resolveComponent as _resolveComponent, openBlock as _openBlock, ",
            "createBlock as _createBlock }} from \"vue\"\n",
            "export function render(_ctx, _cache) {{\n",
            "  const _component_B = _resolveComponent(\"B\")\n",
            "  \n",
            "  return (_openBlock(), {call})\n",
            "}}",
        ),
        call = call
    )
}

fn compile_with_config(source: &str, config: &JsxCompileConfig) -> (String, Vec<String>) {
    let bump = Allocator::new();
    let out = compile_jsx(&bump, source, JsxLang::Jsx, config);
    (
        out.module_code().to_string(),
        out.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.to_string())
            .collect(),
    )
}

fn compile(source: &str, compat: JsxCompatMode, mode: JsxOutputMode) -> (String, Vec<String>) {
    compile_with_config(
        source,
        &JsxCompileConfig {
            compat,
            default_mode: mode,
            ..Default::default()
        },
    )
}

/// Every self-contained literal spelling babel passes straight through.
const FORWARDABLE_LITERALS: [&str; 7] = ["1", "true", "null", "\"text\"", "1n", "/re/g", "`text`"];

#[test]
fn self_contained_v_slots_literals_are_forwarded_in_babel_vdom() {
    for value in FORWARDABLE_LITERALS {
        let source = format!("const A = () => <B v-slots={{{value}}}/>;");
        let (module, diagnostics) =
            compile(source.as_str(), JsxCompatMode::Babel, JsxOutputMode::Vdom);
        assert_eq!(diagnostics, Vec::<String>::new(), "{value}");
        assert_eq!(module, render_module(Some(value)), "{value}");
    }
}

#[test]
fn native_mode_still_rejects_every_forwardable_v_slots_literal() {
    // The switch is opt-in: default Vize output and diagnostics are unchanged.
    for value in FORWARDABLE_LITERALS {
        let source = format!("const A = () => <B v-slots={{{value}}}/>;");
        let (module, diagnostics) =
            compile(source.as_str(), JsxCompatMode::Native, JsxOutputMode::Vdom);
        assert_eq!(diagnostics, vec![not_a_slots_object(value)], "{value}");
        assert_eq!(module, render_module(None), "{value}");
    }
}

#[test]
fn babel_vapor_output_keeps_rejecting_forwardable_v_slots_literals() {
    // Compat mode is a VDOM-only contract, so the Vapor request is diagnosed and
    // the value is still rejected rather than half-applied.
    let expected_module = concat!(
        "import { resolveComponent as _resolveComponent, createComponentWithFallback as ",
        "_createComponentWithFallback } from 'vue';\n",
        "\n",
        "export function render(_ctx) {\n",
        "  const _component_B = _resolveComponent(\"B\")\n",
        "  const n0 = _createComponentWithFallback(_component_B, null, null, true)\n",
        "  return n0\n",
        "}\n",
    );
    for value in ["1", "`text`"] {
        let source = format!("const A = () => <B v-slots={{{value}}}/>;");
        let (module, diagnostics) =
            compile(source.as_str(), JsxCompatMode::Babel, JsxOutputMode::Vapor);
        assert_eq!(
            diagnostics,
            vec![
                not_a_slots_object(value),
                "compiler.jsxCompat: \"babel\" is not supported with Vapor output: \
                 @vue/babel-plugin-jsx has no Vapor equivalent. Use jsxMode \"vdom\" for babel \
                 compatibility, or drop jsxCompat to use Vize's own Vapor semantics."
                    .to_string()
            ],
            "{value}"
        );
        assert_eq!(module, expected_module, "{value}");
    }
}

#[test]
fn babel_ssr_output_keeps_rejecting_forwardable_v_slots_literals() {
    let config = JsxCompileConfig {
        compat: JsxCompatMode::Babel,
        ssr: true,
        ..Default::default()
    };
    let (bare_module, bare_diagnostics) = compile_with_config("const A = () => <B/>;", &config);
    assert_eq!(bare_diagnostics, Vec::<String>::new());

    for value in ["1", "`text`"] {
        let source = format!("const A = () => <B v-slots={{{value}}}/>;");
        let (module, diagnostics) = compile_with_config(source.as_str(), &config);
        assert_eq!(diagnostics, vec![not_a_slots_object(value)], "{value}");
        assert_eq!(module, bare_module, "{value}");
    }
}

#[test]
fn container_and_function_v_slots_values_stay_diagnosed_in_babel_vdom() {
    // These are the shapes whose source text is not a self-contained value:
    // arrays and interpolated template literals may hold nested JSX, raw JSX is
    // a vnode, a lone function is the default slot, and the comma operator
    // changes meaning when spliced into an argument list. Forwarding them
    // verbatim would emit a module that does not mean what babel's does, so they
    // keep their error.
    for (value, message) in [
        ("[a, b]", not_a_slots_object("[a, b]")),
        ("<i/>", not_a_slots_object("<i/>")),
        ("`t${x}`", not_a_slots_object("`t${x}`")),
        ("(a, b)", not_a_slots_object("a, b")),
        (
            "() => <i/>",
            "v-slots value `() => <i/>` is a function, not a slots object: a lone function is \
             the default slot, so a spread of it contributes nothing. Write the slots inline, \
             e.g. v-slots={{ default: () => <div/> }}, or forward a slots object, e.g. \
             v-slots={slots}."
                .to_string(),
        ),
    ] {
        let source = format!("const A = () => <B v-slots={{{value}}}/>;");
        let (module, diagnostics) =
            compile(source.as_str(), JsxCompatMode::Babel, JsxOutputMode::Vdom);
        assert_eq!(diagnostics, vec![message], "{value}");
        assert_eq!(module, render_module(None), "{value}");
    }
}

#[test]
fn a_quoted_v_slots_attribute_is_rejected_in_both_modes() {
    // `v-slots="str"` is an attribute string, not an expression: babel's own
    // reading of it is a plain string child, but Vize has no oracle row pinning
    // that, so the existing diagnostic stands in both modes.
    for compat in [JsxCompatMode::Native, JsxCompatMode::Babel] {
        let (module, diagnostics) = compile(
            "const A = () => <B v-slots=\"str\"/>;",
            compat,
            JsxOutputMode::Vdom,
        );
        assert_eq!(diagnostics, vec![not_a_slots_object("\"str\"")]);
        assert_eq!(module, render_module(None));
    }
}
