//! Primitive `v-slots` values in Babel VDOM compatibility mode (#3391).
//!
//! Babel forwards `v-slots={1}` as the vnode children argument. Vize adopts
//! that observable behavior only for opt-in VDOM compatibility: native mode
//! keeps its strict slots-object diagnostic, while Vapor and SSR do not silently
//! accept a VDOM-only representation.

use vize_atelier_jsx::{JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx};
use vize_carton::Bump;

const SOURCE: &str = "const A = () => <B v-slots={1}/>;";

#[test]
fn babel_compat_forwards_a_numeric_value_only_for_vdom() {
    let compile = |compat| {
        let bump = Bump::new();
        let out = compile_jsx(
            &bump,
            SOURCE,
            JsxLang::Jsx,
            &JsxCompileConfig {
                compat,
                ..Default::default()
            },
        );
        let diagnostics = out
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str().to_string())
            .collect::<Vec<_>>();
        (out.module_code().to_string(), diagnostics)
    };

    let (native, native_diagnostics) = compile(JsxCompatMode::Native);
    assert_eq!(
        native_diagnostics,
        vec![
            "v-slots value `1` is not a slots object: babel forwards it as the component's \
             children, which leaves the component with no slots. Write the slots inline, e.g. \
             v-slots={{ default: () => <div/> }}, or forward a slots object, e.g. \
             v-slots={slots}."
                .to_string()
        ]
    );
    assert_eq!(
        native,
        concat!(
            "import { resolveComponent as _resolveComponent, openBlock as _openBlock, ",
            "createBlock as _createBlock } from \"vue\"\n",
            "export function render(_ctx, _cache) {\n",
            "  const _component_B = _resolveComponent(\"B\")\n",
            "  \n",
            "  return (_openBlock(), _createBlock(_component_B))\n",
            "}",
        )
    );

    let (babel, babel_diagnostics) = compile(JsxCompatMode::Babel);
    assert_eq!(babel_diagnostics, Vec::<String>::new());
    assert_eq!(
        babel,
        concat!(
            "import { resolveComponent as _resolveComponent, openBlock as _openBlock, ",
            "createBlock as _createBlock } from \"vue\"\n",
            "export function render(_ctx, _cache) {\n",
            "  const _component_B = _resolveComponent(\"B\")\n",
            "  \n",
            "  return (_openBlock(), _createBlock(_component_B, null, 1, ",
            "1024 /* DYNAMIC_SLOTS */))\n",
            "}",
        )
    );
}

#[test]
fn babel_compat_forwards_a_static_template_but_not_an_interpolated_one() {
    let compile = |source: &str| {
        let bump = Bump::new();
        let out = compile_jsx(
            &bump,
            source,
            JsxLang::Jsx,
            &JsxCompileConfig {
                compat: JsxCompatMode::Babel,
                ..Default::default()
            },
        );
        let diagnostics = out
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str().to_string())
            .collect::<Vec<_>>();
        (out.module_code().to_string(), diagnostics)
    };

    // A template literal with no substitutions is a primitive: its source is
    // already valid JavaScript, so babel's "forward it as children" is
    // reproducible verbatim.
    let (static_template, static_diagnostics) = compile("const A = () => <B v-slots={`t`}/>;");
    assert_eq!(static_diagnostics, Vec::<String>::new());
    assert!(
        static_template.contains("_createBlock(_component_B, null, `t`, 1024 /* DYNAMIC_SLOTS */)"),
        "{static_template}"
    );

    // An interpolated one stays rejected: its substitutions can hold JSX that
    // still needs lowering, so the source cannot be forwarded verbatim.
    let (interpolated, interpolated_diagnostics) =
        compile("const A = () => <B v-slots={`t${<i/>}`}/>;");
    assert_eq!(
        interpolated_diagnostics,
        vec![
            "v-slots value ``t${<i/>}`` is not a slots object: babel forwards it as the \
             component's children, which leaves the component with no slots. Write the slots \
             inline, e.g. v-slots={{ default: () => <div/> }}, or forward a slots object, e.g. \
             v-slots={slots}."
                .to_string()
        ]
    );
    assert!(!interpolated.contains("DYNAMIC_SLOTS"), "{interpolated}");
}

#[test]
fn compatibility_does_not_leak_into_vapor_or_ssr() {
    let compile = |default_mode, ssr| {
        let bump = Bump::new();
        let out = compile_jsx(
            &bump,
            SOURCE,
            JsxLang::Jsx,
            &JsxCompileConfig {
                default_mode,
                compat: JsxCompatMode::Babel,
                ssr,
                ..Default::default()
            },
        );
        let diagnostics = out
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str().to_string())
            .collect::<Vec<_>>();
        (out.module_code().to_string(), diagnostics)
    };

    let (vapor, vapor_diagnostics) = compile(JsxOutputMode::Vapor, false);
    assert_eq!(
        vapor_diagnostics,
        vec![
            "v-slots value `1` is not a slots object: babel forwards it as the component's \
             children, which leaves the component with no slots. Write the slots inline, e.g. \
             v-slots={{ default: () => <div/> }}, or forward a slots object, e.g. \
             v-slots={slots}."
                .to_string(),
            "compiler.jsxCompat: \"babel\" is not supported with Vapor output: \
             @vue/babel-plugin-jsx has no Vapor equivalent. Use jsxMode \"vdom\" for babel \
             compatibility, or drop jsxCompat to use Vize's own Vapor semantics."
                .to_string(),
        ]
    );
    assert!(!vapor.contains("DYNAMIC_SLOTS"), "{vapor}");

    let (ssr, ssr_diagnostics) = compile(JsxOutputMode::Vdom, true);
    assert_eq!(
        ssr_diagnostics,
        vec![
            "v-slots forwards a slots object the compiler cannot see inside, which SSR output \
             cannot express: the server renderer inlines each slot's content. Write the slots \
             inline, e.g. v-slots={{ default: () => <div/> }}, or render this component on the \
             client."
                .to_string()
        ]
    );
    assert!(!ssr.contains("DYNAMIC_SLOTS"), "{ssr}");
}
