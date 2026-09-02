//! `v-model` array-form argument validation (#3466, #3391).
//!
//! A non-literal argument needs computed prop keys and update-listener names.
//! Opt-in Babel VDOM mode emits those for a **component**, matching
//! `@vue/babel-plugin-jsx`. Everywhere else — native mode, Vapor, SSR, and any
//! plain element — lowering rejects the input instead of silently binding
//! `modelValue` and changing the component contract.

use vize_atelier_jsx::{
    JsxCompatMode, JsxCompileConfig, JsxLang, VdomCompileOptions, compile_jsx, compile_to_vdom,
    lower_source,
};
use vize_s0::Allocator;

const SOURCE: &str = "const A = () => <B v-model={[foo, bar]}/>;";
const ELEMENT_ARG: &str = "const A = () => <input v-model:foo={val}/>;";
const ELEMENT_ARG_MODIFIER: &str = "const A = () => <input v-model:foo_trim={val}/>;";
const COMPONENT_ARG_MODIFIER: &str = "const A = () => <B v-model:foo_trim={val}/>;";
const DYNAMIC_ARGUMENT_ERROR: &str =
    "v-model argument `bar` must be a string literal; dynamic arguments are not supported.";
const REJECTED_ELEMENT_MODULE: &str = concat!(
    "import { openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\"\n",
    "export function render(_ctx, _cache) {\n",
    "  return (_openBlock(), _createElementBlock(\"input\"))\n",
    "}",
);

#[test]
fn babel_compat_plain_element_static_arguments_match_the_oracle() {
    let cases = [
        (
            ELEMENT_ARG,
            concat!(
                "import { vModelText as _vModelText, withDirectives as _withDirectives, openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\"\n",
                "export function render(_ctx, _cache) {\n",
                "  return _withDirectives((_openBlock(), _createElementBlock(\"input\", {\n",
                "    \"onUpdate:foo\": $event => ((val) = $event)\n",
                "  }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onUpdate:foo\"])), [\n",
                "    [_vModelText, val, \"foo\"]\n",
                "  ])\n",
                "}",
            ),
        ),
        (
            ELEMENT_ARG_MODIFIER,
            concat!(
                "import { vModelText as _vModelText, withDirectives as _withDirectives, openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\"\n",
                "export function render(_ctx, _cache) {\n",
                "  return _withDirectives((_openBlock(), _createElementBlock(\"input\", {\n",
                "    \"onUpdate:foo\": $event => ((val) = $event)\n",
                "  }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onUpdate:foo\"])), [\n",
                "    [\n",
                "      _vModelText,\n",
                "      val,\n",
                "      \"foo\",\n",
                "      { trim: true }\n",
                "    ]\n",
                "  ])\n",
                "}",
            ),
        ),
    ];

    for (source, expected) in cases {
        let bump = Allocator::new();
        let output = compile_jsx(
            &bump,
            source,
            JsxLang::Jsx,
            &JsxCompileConfig {
                compat: JsxCompatMode::Babel,
                ..Default::default()
            },
        );

        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        assert_eq!(output.module_code(), expected);
    }
}

#[test]
fn native_plain_element_argument_rejection_is_unchanged() {
    for source in [ELEMENT_ARG, ELEMENT_ARG_MODIFIER] {
        let bump = Allocator::new();
        let output = compile_jsx(&bump, source, JsxLang::Jsx, &JsxCompileConfig::default());

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].message.as_str(),
            "v-model argument is not supported on plain elements."
        );
        assert_eq!(output.module_code(), REJECTED_ELEMENT_MODULE);
    }
}

#[test]
fn babel_compat_component_argument_behavior_is_unchanged() {
    let bump = Allocator::new();
    let output = compile_jsx(
        &bump,
        COMPONENT_ARG_MODIFIER,
        JsxLang::Jsx,
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(
        output.module_code(),
        concat!(
            "import { resolveComponent as _resolveComponent, openBlock as _openBlock, createBlock as _createBlock } from \"vue\"\n",
            "export function render(_ctx, _cache) {\n",
            "  const _component_B = _resolveComponent(\"B\")\n",
            "  \n",
            "  return (_openBlock(), _createBlock(_component_B, {\n",
            "    foo_trim: val,\n",
            "    \"onUpdate:foo_trim\": $event => ((val) = $event)\n",
            "  }, null, 8 /* PROPS */, [\"foo_trim\", \"onUpdate:foo_trim\"]))\n",
            "}",
        )
    );
}

#[test]
fn babel_compat_dynamic_component_argument_emits_computed_prop_keys() {
    // Babel emits `{[bar]: foo, ["onUpdate:" + bar]: $event => foo = $event}`.
    // Vize reaches the same props through its dynamic-prop path, so the
    // argument is never `_ctx.`-prefixed: a JSX component closes over module
    // scope, not a render context.
    let bump = Allocator::new();
    let output = compile_jsx(
        &bump,
        SOURCE,
        JsxLang::Jsx,
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(
        output.module_code(),
        concat!(
            "import { resolveComponent as _resolveComponent, normalizeProps as _normalizeProps, openBlock as _openBlock, createBlock as _createBlock } from \"vue\"\n",
            "export function render(_ctx, _cache) {\n",
            "  const _component_B = _resolveComponent(\"B\")\n",
            "  \n",
            "  return (_openBlock(), _createBlock(_component_B, _normalizeProps({ [bar]: foo,\n",
            "  [\"onUpdate:\" + bar]: $event => ((foo) = $event) }), null, 16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}

#[test]
fn babel_compat_dynamic_component_argument_carries_modifiers_and_member_paths() {
    for (source, key) in [
        (
            "const A = () => <B v-model={[foo, bar, ['trim']]}/>;",
            concat!(
                "{ [bar]: foo,\n",
                "  [\"onUpdate:\" + bar]: $event => ((foo) = $event),\n",
                "  [bar + \"Modifiers\"]: { trim: true } }",
            ),
        ),
        (
            "const A = () => <B v-model={[foo, a.b]}/>;",
            concat!(
                "{ [a.b]: foo,\n",
                "  [\"onUpdate:\" + a.b]: $event => ((foo) = $event) }",
            ),
        ),
    ] {
        let bump = Allocator::new();
        let output = compile_jsx(
            &bump,
            source,
            JsxLang::Jsx,
            &JsxCompileConfig {
                compat: JsxCompatMode::Babel,
                ..Default::default()
            },
        );
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        assert_eq!(
            output.module_code(),
            format!(
                concat!(
                    "import {{ resolveComponent as _resolveComponent, normalizeProps as _normalizeProps, openBlock as _openBlock, createBlock as _createBlock }} from \"vue\"\n",
                    "export function render(_ctx, _cache) {{\n",
                    "  const _component_B = _resolveComponent(\"B\")\n",
                    "  \n",
                    "  return (_openBlock(), _createBlock(_component_B, _normalizeProps({}), null, 16 /* FULL_PROPS */))\n",
                    "}}",
                ),
                key
            ),
            "{source}"
        );
    }
}

#[test]
fn babel_compat_dynamic_argument_stays_rejected_off_the_vdom_component_lane() {
    // A plain element has no computed-prop shape to emit into, and compat mode
    // is a VDOM-only contract, so Vapor and SSR keep the native rejection.
    let element = "const A = () => <input v-model={[foo, bar]}/>;";
    let bump = Allocator::new();
    let output = compile_jsx(
        &bump,
        element,
        JsxLang::Jsx,
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
    );
    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.to_string())
            .collect::<Vec<_>>(),
        vec![DYNAMIC_ARGUMENT_ERROR.to_string()]
    );
    assert_eq!(output.module_code(), REJECTED_ELEMENT_MODULE);

    let bump = Allocator::new();
    let ssr = compile_jsx(
        &bump,
        SOURCE,
        JsxLang::Jsx,
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ssr: true,
            ..Default::default()
        },
    );
    assert_eq!(
        ssr.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.to_string())
            .collect::<Vec<_>>(),
        vec![DYNAMIC_ARGUMENT_ERROR.to_string()]
    );
}

#[test]
fn dynamic_array_argument_is_rejected_without_model_value_fallback() {
    let bump = Allocator::new();
    let lowered = lower_source(&bump, bump.as_oxc(), SOURCE, JsxLang::Jsx);
    let errors: Vec<_> = lowered
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.is_error())
        .collect();

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message.as_str(), DYNAMIC_ARGUMENT_ERROR);
    assert_eq!(
        &SOURCE[errors[0].start as usize..errors[0].end as usize],
        "bar"
    );

    let bump = Allocator::new();
    let compiled = compile_to_vdom(&bump, SOURCE, JsxLang::Jsx, VdomCompileOptions::default());
    assert!(compiled.has_errors());
    assert_eq!(compiled.components.len(), 1);
    assert_eq!(
        compiled.components[0].code.as_str(),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B))\n}"
    );
}
