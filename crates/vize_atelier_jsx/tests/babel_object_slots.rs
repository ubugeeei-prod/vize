//! Babel JSX object-slot compatibility (#3391).
//!
//! The plugin treats a lone component child specially: identifiers and call
//! results may already be a slots object, while every other value becomes the
//! raw default-slot child. `enableObjectSlots: false` disables only the runtime
//! slots-object check. Native, Vapor, and SSR output must remain untouched.

use vize_atelier_jsx::{
    BabelJsxOptions, JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx,
    compile_jsx_with_babel_object_slots,
};
use vize_s0::Allocator;

const LONE_IDENTIFIER_CHILD: &str = "const A = () => <B>{slots}</B>;";

/// The plugin's own `_isSlot` discriminator, emitted once per module.
const IS_SLOT_HELPER: &str = concat!(
    "function _isSlot(s) {\n",
    "return typeof s === 'function' || Object.prototype.toString.call(s) === ",
    "'[object Object]' && !_isVNode(s);\n",
    "}\n",
);

fn compile(
    source: &str,
    compat: JsxCompatMode,
    mode: JsxOutputMode,
    enable_object_slots: bool,
) -> (String, Vec<String>) {
    let bump = Allocator::new();
    let output = compile_jsx_with_babel_object_slots(
        &bump,
        source,
        JsxLang::Tsx,
        &JsxCompileConfig {
            compat,
            default_mode: mode,
            ..Default::default()
        },
        &BabelJsxOptions::default(),
        enable_object_slots,
    );
    (
        output.module_code().to_string(),
        output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.to_string())
            .collect(),
    )
}

fn compile_default(source: &str, compat: JsxCompatMode) -> String {
    let bump = Allocator::new();
    compile_jsx(
        &bump,
        source,
        JsxLang::Tsx,
        &JsxCompileConfig {
            compat,
            ..Default::default()
        },
    )
    .module_code()
    .to_string()
}

#[test]
fn option_is_babel_vdom_only_and_true_is_the_babel_default() {
    // Native output is byte-identical whichever way the option is set, and
    // Babel's own default is `true`, so the plain entry point already agrees.
    let native_default = compile_default(LONE_IDENTIFIER_CHILD, JsxCompatMode::Native);
    for enabled in [true, false] {
        let (native, diagnostics) = compile(
            LONE_IDENTIFIER_CHILD,
            JsxCompatMode::Native,
            JsxOutputMode::Vdom,
            enabled,
        );
        assert_eq!(native, native_default);
        assert_eq!(diagnostics, Vec::<String>::new());
    }

    let (babel, diagnostics) = compile(
        LONE_IDENTIFIER_CHILD,
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        true,
    );
    assert_eq!(diagnostics, Vec::<String>::new());
    assert_eq!(
        babel,
        compile_default(LONE_IDENTIFIER_CHILD, JsxCompatMode::Babel)
    );

    // Vapor has no Babel equivalent, so both settings produce the same
    // unsupported-mode diagnostic and the same Vize-native Vapor module.
    let (vapor_on, vapor_on_diagnostics) = compile(
        LONE_IDENTIFIER_CHILD,
        JsxCompatMode::Babel,
        JsxOutputMode::Vapor,
        true,
    );
    let (vapor_off, vapor_off_diagnostics) = compile(
        LONE_IDENTIFIER_CHILD,
        JsxCompatMode::Babel,
        JsxOutputMode::Vapor,
        false,
    );
    assert_eq!(vapor_on, vapor_off);
    assert_eq!(vapor_on_diagnostics, vapor_off_diagnostics);
    assert_eq!(
        vapor_on_diagnostics,
        vec![
            "compiler.jsxCompat: \"babel\" is not supported with Vapor output: \
             @vue/babel-plugin-jsx has no Vapor equivalent. Use jsxMode \"vdom\" for babel \
             compatibility, or drop jsxCompat to use Vize's own Vapor semantics."
                .to_string()
        ]
    );
}

#[test]
fn identifier_children_switch_between_runtime_slots_and_a_raw_default() {
    let (enabled, diagnostics) = compile(
        LONE_IDENTIFIER_CHILD,
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        true,
    );
    assert_eq!(diagnostics, Vec::<String>::new());
    assert_eq!(
        enabled,
        format!(
            concat!(
                "import {{ resolveComponent as _resolveComponent, openBlock as _openBlock, ",
                "createBlock as _createBlock, isVNode as _isVNode }} from \"vue\"\n",
                "{}",
                "export function render(_ctx, _cache) {{\n",
                "  const _component_B = _resolveComponent(\"B\")\n",
                "  \n",
                "  return (_openBlock(), _createBlock(_component_B, null, ",
                "_isSlot(slots) ? slots : {{ default: () => [slots] }}, ",
                "1024 /* DYNAMIC_SLOTS */))\n",
                "}}",
            ),
            IS_SLOT_HELPER
        )
    );

    let (disabled, diagnostics) = compile(
        LONE_IDENTIFIER_CHILD,
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        false,
    );
    assert_eq!(diagnostics, Vec::<String>::new());
    assert_eq!(
        disabled,
        concat!(
            "import { resolveComponent as _resolveComponent, openBlock as _openBlock, ",
            "createBlock as _createBlock } from \"vue\"\n",
            "export function render(_ctx, _cache) {\n",
            "  const _component_B = _resolveComponent(\"B\")\n",
            "  \n",
            "  return (_openBlock(), _createBlock(_component_B, null, ",
            "{ default: () => [slots] }, 1024 /* DYNAMIC_SLOTS */))\n",
            "}",
        )
    );
}

#[test]
fn call_children_are_wrapped_so_the_call_evaluates_once() {
    let (call, diagnostics) = compile(
        "const A = () => <B>{getSlots()}</B>;",
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        true,
    );
    assert_eq!(diagnostics, Vec::<String>::new());
    assert_eq!(
        call,
        format!(
            concat!(
                "import {{ resolveComponent as _resolveComponent, openBlock as _openBlock, ",
                "createBlock as _createBlock, isVNode as _isVNode }} from \"vue\"\n",
                "{}",
                "export function render(_ctx, _cache) {{\n",
                "  const _component_B = _resolveComponent(\"B\")\n",
                "  \n",
                "  return (_openBlock(), _createBlock(_component_B, null, ",
                "(_slot => _isSlot(_slot) ? _slot : {{ default: () => [_slot] }})(getSlots()), ",
                "1024 /* DYNAMIC_SLOTS */))\n",
                "}}",
            ),
            IS_SLOT_HELPER
        )
    );
}

#[test]
fn non_candidate_children_stay_raw_default_slot_values() {
    // Babel only probes identifiers and calls; everything else is the raw
    // default-slot child, never stringified through `toDisplayString`.
    for (child, expected) in [
        ("state.slots", "{ default: () => [state.slots] }"),
        ("1", "{ default: () => [1] }"),
        ("ok ? a : b", "{ default: () => [ok ? a : b] }"),
        ("(a, b)", "{ default: () => [((a, b))] }"),
    ] {
        let source = format!("const A = () => <B>{{{child}}}</B>;");
        let (output, diagnostics) = compile(
            source.as_str(),
            JsxCompatMode::Babel,
            JsxOutputMode::Vdom,
            true,
        );
        assert_eq!(diagnostics, Vec::<String>::new(), "{child}");
        assert_eq!(
            output,
            format!(
                concat!(
                    "import {{ resolveComponent as _resolveComponent, openBlock as _openBlock, ",
                    "createBlock as _createBlock }} from \"vue\"\n",
                    "export function render(_ctx, _cache) {{\n",
                    "  const _component_B = _resolveComponent(\"B\")\n",
                    "  \n",
                    "  return (_openBlock(), _createBlock(_component_B, null, {}, ",
                    "1024 /* DYNAMIC_SLOTS */))\n",
                    "}}",
                ),
                expected
            ),
            "{child}"
        );
    }
}

#[test]
fn helpers_are_emitted_once_and_explicit_v_slots_keeps_ordinary_children() {
    let (output, diagnostics) = compile(
        concat!(
            "const A = () => <B>{slots}</B>;\n",
            "const C = () => <B>{otherSlots}</B>;\n",
            "const D = () => <B v-slots={forwarded}>{value}</B>;",
        ),
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        true,
    );
    assert_eq!(diagnostics, Vec::<String>::new());
    assert_eq!(
        output,
        format!(
            concat!(
                "import {{ resolveComponent as _resolveComponent, openBlock as _openBlock, ",
                "createBlock as _createBlock, isVNode as _isVNode, withCtx as _withCtx }} ",
                "from \"vue\"\n",
                "{}",
                "export function A(_ctx, _cache) {{\n",
                "  const _component_B = _resolveComponent(\"B\")\n",
                "  \n",
                "  return (_openBlock(), _createBlock(_component_B, null, ",
                "_isSlot(slots) ? slots : {{ default: () => [slots] }}, ",
                "1024 /* DYNAMIC_SLOTS */))\n",
                "}}\n",
                "export function C(_ctx, _cache) {{\n",
                "  const _component_B = _resolveComponent(\"B\")\n",
                "  \n",
                "  return (_openBlock(), _createBlock(_component_B, null, ",
                "_isSlot(otherSlots) ? otherSlots : {{ default: () => [otherSlots] }}, ",
                "1024 /* DYNAMIC_SLOTS */))\n",
                "}}\n",
                "export function D(_ctx, _cache) {{\n",
                "  const _component_B = _resolveComponent(\"B\")\n",
                "  \n",
                "  return (_openBlock(), _createBlock(_component_B, null, {{\n",
                "    default: _withCtx(() => [\n",
                "      value\n",
                "    ]),\n",
                "    ...forwarded\n",
                "  }}, 1024 /* DYNAMIC_SLOTS */))\n",
                "}}",
            ),
            IS_SLOT_HELPER
        )
    );
}

#[test]
fn helper_bindings_are_collision_free_and_typescript_is_stripped() {
    let (output, diagnostics) = compile(
        concat!(
            "const _isSlot = existing; const _isVNode = existingVNode;\n",
            "type Slots = Record<string, unknown>;\n",
            "const A = () => <B>{slots as Slots}</B>;",
        ),
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        true,
    );
    assert_eq!(diagnostics, Vec::<String>::new());
    assert_eq!(
        output,
        concat!(
            "import { resolveComponent as _resolveComponent, openBlock as _openBlock, ",
            "createBlock as _createBlock, isVNode as _isVNode_ } from \"vue\"\n",
            "function _isSlot_(s) {\n",
            "return typeof s === 'function' || Object.prototype.toString.call(s) === ",
            "'[object Object]' && !_isVNode_(s);\n",
            "}\n",
            "export function render(_ctx, _cache) {\n",
            "  const _component_B = _resolveComponent(\"B\")\n",
            "  \n",
            "  return (_openBlock(), _createBlock(_component_B, null, ",
            "_isSlot_(slots) ? slots : { default: () => [slots] }, ",
            "1024 /* DYNAMIC_SLOTS */))\n",
            "}",
        )
    );
}
