//! Forwarding an opaque slots object from `v-slots={slots}` (#3467).
//!
//! `v-slots={{ … }}` expands into synthetic `<template v-slot:name>` children
//! (#3418, `v_slots.rs`). `v-slots={slots}` cannot: the value only exists at
//! runtime, so there are no entries to expand. It lowers to a relief `slots`
//! directive that VDOM emits as a spread and Vapor passes as a raw slot source.
//!
//! Both babel shapes are asserted on the **complete** generated render function,
//! against the recorded `@vue/babel-plugin-jsx` 2.0.1 output in the corpus rows
//! `slots/v_slots_only`, `slots/v_slots_with_children` and
//! `optimize/v_slots_stability`:
//!
//! ```js
//! const A = () => _createVNode(_resolveComponent("B"), null, slots);
//! const C = () => _createVNode(_resolveComponent("B"), null, {
//!   default: () => [_createVNode("div", null, [_createTextVNode("A")])],
//!   ...slots
//! });
//! ```
//!
//! # Why no `_` flag and why `1024 /* DYNAMIC_SLOTS */`
//!
//! Babel emits no slot-stability flag beside a forwarded object, even under
//! `optimize: true`, and neither does Vize. Only the no-`_` path runs Vue's
//! `normalizeObjectSlots`, which binds raw entries to the owning instance and
//! passes already-`withCtx`-wrapped ones through untouched via `rawSlot._n`.
//! `_: 2 /* DYNAMIC */` would `extend` without normalizing; `_: 1 /* STABLE */`
//! would stop the child re-rendering when the forwarded slots change. Babel's
//! unoptimized vnodes get that update for free (`shouldUpdateComponent` forces
//! it for any children); Vize's are always optimized, so the vnode carries
//! `1024 /* DYNAMIC_SLOTS */` to force it.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

#[expect(dead_code, reason = "shared test helpers; each binary uses a subset")]
mod common;

use vize_atelier_jsx::{
    JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, SsrCompileOptions,
    VaporCompileOptions, VdomCompileOptions, compile_jsx, compile_to_ssr, compile_to_vapor,
    compile_to_vdom,
};
use vize_l0::Allocator;

const VAPOR_RUNTIME_CASES: [(&str, &str); 7] = [
    ("bare", "const A = () => <B v-slots={slots}/>;"),
    (
        "mixed",
        "const A = () => <B v-slots={getSlots()}><div>A</div><template v-slot:named><i>N</i></template></B>;",
    ),
    ("member", "const A = () => <B v-slots={owner.slots}/>;"),
    (
        "conditional",
        "const A = () => <B v-slots={choose ? left : right}/>;",
    ),
    (
        "nested",
        "const A = () => <Outer><B v-slots={slots}/></Outer>;",
    ),
    (
        "dynamic-component",
        "const A = () => <view.Component v-slots={slots}/>;",
    ),
    ("alias", "const A = () => <B vSlots={slots}/>;"),
];

fn render_code(source: &str) -> String {
    let bump = Allocator::new();
    let out = compile_to_vdom(&bump, source, JsxLang::Jsx, VdomCompileOptions::default());
    assert_eq!(
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str().to_string())
            .collect::<Vec<_>>(),
        Vec::<String>::new(),
        "forwarding a slots object to VDOM is supported and must not diagnose"
    );
    out.components
        .into_iter()
        .map(|component| component.code.as_str().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_forwarded_slots_object_is_the_children_argument() {
    // Corpus rows `slots/v_slots_only` and `optimize/v_slots_stability`: babel
    // emits `_createVNode(_resolveComponent("B"), null, slots)` for both, with
    // no `_` flag under either option set.
    assert_eq!(
        render_code("const A = () => <B v-slots={slots}/>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, slots, \
         1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn element_children_become_the_default_slot_and_the_spread_closes_the_object() {
    // Corpus row `slots/v_slots_with_children`: babel emits
    // `{default: () => [...], ...slots}` — authored slots first, spread last, so
    // a forwarded entry overrides an authored one of the same name.
    assert_eq!(
        render_code("const A = () => <B v-slots={slots}><div>A</div></B>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         default: _withCtx(() => [\n      \
         _createElementVNode(\"div\", null, \"A\")\n    \
         ]),\n    \
         ...slots\n  \
         }, 1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn named_slot_templates_keep_the_spread_last() {
    assert_eq!(
        render_code("const A = () => <B v-slots={slots}><template v-slot:foo>x</template></B>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         foo: _withCtx(() => [\n      \
         _createTextVNode(\"x\")\n    \
         ]),\n    \
         ...slots\n  \
         }, 1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn whitespace_only_children_do_not_build_an_object() {
    // Whitespace is not a default slot, so this is still the bare-value shape.
    assert_eq!(
        render_code("const A = () => <B v-slots={slots}>{' '}</B>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, slots, \
         1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn any_opaque_expression_is_forwarded_verbatim() {
    // Parentheses and TS wrappers are transparent; what is emitted is the
    // expression underneath, so no TS-only syntax reaches the JavaScript.
    for (source, forwarded) in [
        ("<B v-slots={p.slots}/>", "p.slots"),
        ("<B v-slots={(slots)}/>", "slots"),
        ("<B v-slots={getSlots()}/>", "getSlots()"),
        ("<B v-slots={c ? a : b}/>", "c ? a : b"),
    ] {
        assert_eq!(
            render_code(&format!("const A = () => {source};")),
            format!(
                "export function render(_ctx, _cache) {{\n  \
                 const _component_B = _resolveComponent(\"B\")\n  \n  \
                 return (_openBlock(), _createBlock(_component_B, null, {forwarded}, \
                 1024 /* DYNAMIC_SLOTS */))\n}}"
            ),
            "{source}"
        );
    }
}

#[test]
fn a_forwarded_slots_object_survives_v_for() {
    // A component with no children of its own still needs the slots argument
    // emitted inside `renderList`, where the children slot is otherwise skipped.
    assert_eq!(
        render_code("const A = () => <ul>{items.map(i => <B key={i} v-slots={slots}/>)}</ul>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createElementBlock(\"ul\", null, [\n    \
         (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (i) => {\n      \
         return (_openBlock(), _createBlock(_component_B, {\n        \
         key: i\n      \
         }, slots, 1024 /* DYNAMIC_SLOTS */))\n    \
         }), 128 /* KEYED_FRAGMENT */))\n  \
         ]))\n}"
    );
}

#[test]
fn v_slots_never_becomes_a_resolved_directive() {
    // The #3418 regression: an unrecognized `v-*` fell through to the custom
    // directive path. `slots` is a compiler built-in, so it contributes no prop,
    // no `withDirectives` wrapper and no `resolveDirective("slots")`.
    let bump = Allocator::new();
    let root = common::lower_one(&bump, "const A = () => <B class=\"c\" v-slots={slots}/>;");
    let element = common::root_element(&root);
    assert_eq!(element.props.len(), 2);
    assert_eq!(common::as_attribute(&element.props[0]).name, "class");
    let forwarded =
        common::find_directive(element, "slots").expect("v-slots lowers to a directive");
    assert!(forwarded.arg.is_none());
    let exp = forwarded.exp.as_ref().expect("forwarded expression");
    assert_eq!(common::simple_content(exp), "slots");
    assert!(!common::is_static(exp));
}

#[test]
fn vapor_output_forwards_the_runtime_slots_object() {
    let bump = Allocator::new();
    let out = compile_to_vapor(
        &bump,
        "const A = () => <B v-slots={slots}/>;",
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
    assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1);
    assert_eq!(
        out.components[0].code,
        "import { resolveComponent as _resolveComponent, createComponentWithFallback as \
         _createComponentWithFallback } from 'vue';\n\n\
         export function render(_ctx) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \
         const n0 = _createComponentWithFallback(_component_B, null, slots, true)\n  \
         return n0\n}\n"
    );
}

#[test]
fn vapor_authored_slots_precede_the_forwarded_collision_source() {
    let bump = Allocator::new();
    let out = compile_to_vapor(
        &bump,
        "const A = () => <B v-slots={getSlots()}><div>A</div><template v-slot:named><i>N</i></template></B>;",
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
    assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    let code = out.components[0].code.as_str();
    assert_eq!(
        code,
        r#"import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback, template as _template } from 'vue';
const t0 = _template("<i>N</i>")
const t1 = _template("<div>A</div>")

export function render(_ctx) {
  const _component_B = _resolveComponent("B")
  const n3 = _createComponentWithFallback(_component_B, null, {
    "named": () => {
      const n0 = t0()
      return n0
    },
    "default": () => {
      const n2 = t1()
      return n2
    },
    $: [
      getSlots()
    ]
  }, true)
  return n3
}
"#
    );
}

#[test]
fn vapor_runtime_slots_capture_all_complete_modules() {
    let packets = VAPOR_RUNTIME_CASES
        .into_iter()
        .map(|(id, source)| {
            let allocator = Allocator::new();
            let output = compile_to_vapor(
                &allocator,
                source,
                JsxLang::Jsx,
                VaporCompileOptions::default(),
            );
            assert!(
                output.diagnostics.is_empty(),
                "{id}: {:?}",
                output.diagnostics
            );
            assert_eq!(output.components.len(), 1, "{id}");
            let component = output.components.first().expect("one Vapor component");
            assert_eq!(component.mode, JsxOutputMode::Vapor, "{id}");
            serde_json::json!({ "id": id, "source": source, "code": component.code.as_str() })
        })
        .collect::<Vec<_>>();
    assert_eq!(packets.len(), VAPOR_RUNTIME_CASES.len());
    if let Some(path) = std::env::var_os("VIZE_JSX_VAPOR_SLOTS_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&packets).expect("whole client Vapor module packets"),
        )
        .expect("retain fresh JSX client Vapor modules");
    }
}

#[test]
fn ssr_output_forwards_the_typed_slots_object() {
    let bump = Allocator::new();
    let out = compile_to_ssr(
        &bump,
        "const A = () => <B v-slots={slots}/>;",
        JsxLang::Jsx,
        SsrCompileOptions::default(),
    );
    assert_eq!(out.diagnostics.len(), 0);
    assert_eq!(out.components.len(), 1);
    assert_eq!(
        out.components
            .first()
            .expect("one SSR component")
            .code
            .as_str(),
        r#"import { ssrRenderComponent as _ssrRenderComponent } from "@vue/server-renderer"
import { resolveComponent as _resolveComponent } from "vue"

function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(_ssrRenderComponent(_resolveComponent("B"), _attrs, slots, _parent))
}
"#
    );
}

/// Compare complete authored modules and diagnostics. The hyphenated forms are
/// independently pinned by the existing exact render tests and Babel recording.
/// Diagnostic source slices also prove the shorter alias retains correct ranges.
fn assert_camel_alias(source: &str, config: &JsxCompileConfig, errors: usize) {
    let camel = source.replace("v-slots", "vSlots");
    assert_ne!(camel, source);
    let bump = Allocator::new();
    let expected = compile_jsx(&bump, source, JsxLang::Jsx, config);
    let actual = compile_jsx(&bump, &camel, JsxLang::Jsx, config);
    assert_eq!(actual.module_code(), expected.module_code(), "{source}");
    assert_eq!(
        expected.diagnostics.iter().filter(|d| d.is_error()).count(),
        errors,
        "{source}: {:?}",
        expected.diagnostics
    );
    let diagnostics = |input: &str, output: &vize_atelier_jsx::JsxCompileOutput| {
        output
            .diagnostics
            .iter()
            .map(|d| {
                let range = input
                    .get(d.start as usize..d.end as usize)
                    .expect("diagnostic must name a valid original source range");
                (
                    d.severity,
                    d.message.to_string(),
                    range.replace("vSlots", "v-slots"),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        diagnostics(&camel, &actual),
        diagnostics(source, &expected),
        "{source}"
    );
}

#[test]
fn camel_slots_forwarding_keeps_whole_modules_and_backend_diagnostics() {
    for source in [
        "const A = () => <B v-slots={slots}/>;",
        "const A = () => <B v-slots={slots}><div>A</div></B>;",
        "const A = () => <B v-slots:default={slots}/>;",
        "const A = () => <ul>{items.map(i => <B key={i} v-slots={slots}/>)}</ul>;",
    ] {
        for config in [
            JsxCompileConfig::default(),
            JsxCompileConfig {
                compat: JsxCompatMode::Babel,
                ..Default::default()
            },
            JsxCompileConfig {
                default_mode: JsxOutputMode::Vapor,
                ..Default::default()
            },
            JsxCompileConfig {
                ssr: true,
                ..Default::default()
            },
        ] {
            assert_camel_alias(source, &config, 0);
        }
    }
}

#[test]
fn camel_inline_slots_compile_on_every_backend() {
    for source in [
        "const A = () => <B v-slots={{default: () => <i/>, bar: () => <b/>}}/>;",
        "const A = () => <B v-slots={{bar: () => <b/>}}><div>A</div></B>;",
    ] {
        for config in [
            JsxCompileConfig::default(),
            JsxCompileConfig {
                compat: JsxCompatMode::Babel,
                ..Default::default()
            },
            JsxCompileConfig {
                default_mode: JsxOutputMode::Vapor,
                ..Default::default()
            },
            JsxCompileConfig {
                ssr: true,
                ..Default::default()
            },
        ] {
            assert_camel_alias(source, &config, 0);
        }
    }
}

#[test]
fn camel_slots_values_keep_existing_validation_and_source_ranges() {
    for source in [
        "const A = () => <B v-slots/>;",
        "const A = () => <B v-slots={1}/>;",
        "const A = () => <B v-slots={() => <i/>}/>;",
        "const A = () => <div v-slots={{default: () => <i/>}}/>;",
        "const A = () => <B v-slots={{default: () => <i/>}}><div>A</div></B>;",
    ] {
        assert_camel_alias(source, &JsxCompileConfig::default(), 1);
    }
}

#[test]
fn mixed_slots_aliases_are_one_duplicate_builtin() {
    let expected = vec!["v-slots is given more than once on this element; babel keeps only the last one, so merge them into a single slots object.".to_string()];
    for source in [
        "const A = () => <B vSlots={a} v-slots={b}/>;",
        "const A = () => <B v-slots={a} vSlots={b}/>;",
        "const A = () => <B vSlots={a} vSlots={b}/>;",
    ] {
        let bump = Allocator::new();
        let output = compile_jsx(&bump, source, JsxLang::Jsx, &JsxCompileConfig::default());
        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|d| d.message.to_string())
                .collect::<Vec<_>>(),
            expected,
            "{source}"
        );
        let diagnostic = &output.diagnostics[0];
        let second = source.split_once("={a} ").expect("second attribute").1;
        let second = second.strip_suffix("/>;").expect("component end");
        assert_eq!(
            source.get(diagnostic.start as usize..diagnostic.end as usize),
            Some(second)
        );
        assert!(diagnostic.is_error());
    }
}

#[test]
fn slots_alias_lookalikes_remain_ordinary_props() {
    for name in ["vslots", "VSlots", "vSlot", "vSlotsLike", "vSlots_foo"] {
        let bump = Allocator::new();
        let source = format!("const A = () => <B {name}={{slots}}/>;");
        let root = common::lower_one(&bump, &source);
        let element = common::root_element(&root);
        assert_eq!(element.props.len(), 1, "{name}");
        let prop = common::as_directive(&element.props[0]);
        assert_eq!(prop.name, "bind", "{name}");
        assert_eq!(
            common::simple_content(prop.arg.as_ref().expect("prop name")),
            name
        );
        assert_eq!(
            common::simple_content(prop.exp.as_ref().expect("prop value")),
            "slots"
        );
    }
}

/// The original namespace spelling was accepted by legacy lowering, which
/// discarded its argument. This preserves that behavior, not slot selection.
#[test]
fn legacy_namespace_spellings_forward_the_same_whole_ssr_module() {
    let allocator = Allocator::new();
    let expected = compile_to_ssr(
        &allocator,
        "const A = () => <B v-slots={slots}/>;",
        JsxLang::Jsx,
        SsrCompileOptions::default(),
    );
    assert!(expected.diagnostics.is_empty());
    assert_eq!(expected.components.len(), 1);
    let expected = expected
        .components
        .first()
        .expect("golden-pinned bare module");
    for source in [
        "const A = () => <B v-slots:default={slots}/>;",
        "const A = () => <B vSlots:default={slots}/>;",
    ] {
        let output = compile_to_ssr(
            &allocator,
            source,
            JsxLang::Jsx,
            SsrCompileOptions::default(),
        );
        assert!(output.diagnostics.is_empty(), "{source}");
        assert_eq!(output.components.len(), 1, "{source}");
        assert_eq!(
            output.components.first().expect("one SSR module").code,
            expected.code,
            "{source}"
        );
    }
}
