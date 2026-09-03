//! P2-11 installment 84 witness: **module mode** (`CodegenMode::Module`),
//! the first production-option surface — `import { … } from "vue"`,
//! `export function render(_ctx, _cache)`, hoists after the import block —
//! plus custom runtime module / global names, compared byte-for-byte with
//! the shipped lane over a family-spanning battery.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::{CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_s0::String;
use vize_s1_to_s2::{DomEmitMode, DomEmitOptions};

const BATTERY: &[(&str, &str)] = &[
    ("empty", ""),
    ("empty_div", "<div></div>"),
    ("static_text", "<div>hello</div>"),
    (
        "static_attrs",
        r#"<div id="app" class="container">static</div>"#,
    ),
    ("interpolation", "{{ msg }}"),
    ("interpolation_in_element", "<div>{{ msg }}</div>"),
    ("mixed_text", "<p>Hi {{ name }}!</p>"),
    (
        "static_child_hoist",
        r#"<div><span class="x">hi</span>{{ msg }}</div>"#,
    ),
    (
        "nested_static_subtree_hoist",
        r#"<div><div class="a"><span>a</span><span>b</span></div>{{ msg }}</div>"#,
    ),
    ("dynamic_class", r#"<div :class="cls"></div>"#),
    (
        "static_and_dynamic_class",
        r#"<div class="base" :class="cls"></div>"#,
    ),
    ("dynamic_style", r#"<div :style="s"></div>"#),
    (
        "static_and_dynamic_style",
        r#"<div style="color: red" :style="s"></div>"#,
    ),
    ("click_handler", r#"<div @click="handler"></div>"#),
    ("inline_click", r#"<div @click="count++"></div>"#),
    ("click_stop", r#"<div @click.stop="handler"></div>"#),
    ("keyup_enter", r#"<div @keyup.enter="handler"></div>"#),
    ("object_bind", r#"<div v-bind="obj"></div>"#),
    (
        "object_bind_with_attrs",
        r#"<div id="x" v-bind="obj"></div>"#,
    ),
    ("object_on", r#"<div v-on="handlers"></div>"#),
    ("v_if", r#"<div v-if="ok">hello</div>"#),
    (
        "v_if_else_chain",
        r#"<div v-if="a">1</div><div v-else-if="b">2</div><div v-else>3</div>"#,
    ),
    (
        "keyed_v_for",
        r#"<div v-for="item in list" :key="item.id">{{ item }}</div>"#,
    ),
    ("v_for_numeric", r#"<span v-for="n in 3">{{ n }}</span>"#),
    ("component", "<Foo />"),
    ("component_with_props", r#"<Foo id="x" :bar="baz" />"#),
    ("component_default_slot", "<Foo>hello</Foo>"),
    (
        "component_named_scoped_slot",
        r#"<Foo><template #item="{ item }"><span>{{ item }}</span></template></Foo>"#,
    ),
    (
        "component_conditional_slot",
        r#"<Foo><template v-if="ok" #a>a</template></Foo>"#,
    ),
    ("dynamic_component", r#"<component :is="tag" />"#),
    ("slot_outlet", "<slot />"),
    (
        "named_slot_outlet_with_fallback",
        "<slot name=\"x\">fallback</slot>",
    ),
    ("v_model_text", r#"<input v-model="msg" />"#),
    (
        "v_model_checkbox",
        r#"<input type="checkbox" v-model="ok" />"#,
    ),
    ("v_model_component", r#"<Foo v-model="val" />"#),
    ("v_show", r#"<div v-show="ok"></div>"#),
    ("custom_directive", r#"<div v-focus></div>"#),
    (
        "custom_directive_value",
        r#"<div v-my-dir:arg.mod="val"></div>"#,
    ),
    ("v_html", r#"<div v-html="raw"></div>"#),
    ("v_text", r#"<div v-text="txt"></div>"#),
    ("v_once", r#"<div v-once>{{ msg }}</div>"#),
    ("v_memo", r#"<div v-memo="[a]">{{ msg }}</div>"#),
    ("template_ref", r#"<div ref="el"></div>"#),
    ("teleport", r#"<Teleport to="body"><div>x</div></Teleport>"#),
    ("keep_alive", "<KeepAlive><Foo /></KeepAlive>"),
    (
        "transition",
        "<Transition><div v-if=\"ok\">x</div></Transition>",
    ),
    ("multi_root_fragment", "<div>a</div><div>b</div>"),
    ("comment_root", "<!-- c --><div>x</div>"),
    ("svg_foreign", "<svg><circle r=\"1\" /></svg>"),
    (
        "template_v_if_fragment",
        r#"<template v-if="ok"><span>a</span><span>b</span></template>"#,
    ),
    (
        "template_v_for_fragment",
        r#"<template v-for="i in n"><span>{{ i }}</span></template>"#,
    ),
];

fn module_options() -> DomCompilerOptions {
    DomCompilerOptions {
        mode: CodegenMode::Module,
        ..Default::default()
    }
}

#[test]
fn module_mode_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &module_options(),
        &CodegenOptions::default(),
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn function_mode_under_explicit_options_matches_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &DomCompilerOptions::default(),
        &CodegenOptions::default(),
        &DomEmitOptions::DEFAULT,
    );
}

#[test]
fn custom_runtime_module_name_matches_the_shipped_dom_lane() {
    let codegen = CodegenOptions {
        runtime_module_name: String::from("@scope/vue-runtime"),
        ..Default::default()
    };
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &module_options(),
        &codegen,
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            runtime_module_name: "@scope/vue-runtime",
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn custom_runtime_global_name_matches_the_shipped_dom_lane() {
    let codegen = CodegenOptions {
        runtime_global_name: String::from("VueRuntime"),
        ..Default::default()
    };
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &DomCompilerOptions::default(),
        &codegen,
        &DomEmitOptions {
            runtime_global_name: "VueRuntime",
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn module_mode_preamble_assets_and_signature_take_the_module_shape() {
    let allocator = vize_s0::Allocator::new();
    let emit = vize_s1_to_s2::emit_dom_source_with_options(
        &allocator,
        r#"<Foo v-focus @click="go"><template #item="{ item }">{{ item }}</template></Foo>"#,
        vize_s1_to_s2::LegacyCaps::VUE3,
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            ..DomEmitOptions::DEFAULT
        },
    )
    .expect("module-mode emit");
    assert_eq!(
        emit.preamble.as_str(),
        "import { resolveDirective as _resolveDirective, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, openBlock as _openBlock, createBlock as _createBlock, createTextVNode as _createTextVNode, withCtx as _withCtx } from \"vue\"\n"
    );
    assert_eq!(
        emit.code.as_str(),
        concat!(
            "export function render(_ctx, _cache) {\n",
            "  const _component_Foo = _resolveComponent(\"Foo\")\n",
            "  const _directive_focus = _resolveDirective(\"focus\")\n",
            "  \n",
            "  return _withDirectives((_openBlock(), _createBlock(_component_Foo, { onClick: go }, {\n",
            "    item: _withCtx(({ item }) => [\n",
            "      _createTextVNode(_toDisplayString(item), 1 /* TEXT */)\n",
            "    ]),\n",
            "    _: 1 /* STABLE */\n",
            "  }, 8 /* PROPS */, [\"onClick\"])), [\n",
            "    [_directive_focus]\n",
            "  ])\n",
            "}"
        )
    );
}
