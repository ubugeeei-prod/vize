//! P2-11 installment 85 witness: **`prefix_identifiers`** — free
//! identifiers become `_ctx.` accesses exactly as the shipped transform
//! and codegen spell them (scope hygiene for `v-for` aliases and slot
//! params, shorthand expansion, handler wrapping, the codegen-time
//! slot-param strips, dynamic-argument special cases), compared
//! byte-for-byte with the shipped lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::{CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_s1_to_s2::{DomEmitMode, DomEmitOptions};

const BATTERY: &[(&str, &str)] = &[
    ("interpolation", "<div>{{ msg }}</div>"),
    ("interpolation_member", "<div>{{ user.name }}</div>"),
    ("interpolation_global", "<div>{{ Math.max(a, 1) }}</div>"),
    ("interpolation_comment", "<div>{{ a // c }}</div>"),
    ("mixed_text", "<p>Hi {{ name }}!</p>"),
    ("bind_padded", r#"<div :id=" x "></div>"#),
    ("bind_comment", r#"<div :id="x // c"></div>"#),
    ("bind_shorthand_object", r#"<div :foo="{ a, b: c }"></div>"#),
    ("bind_arrow", r#"<div :fn="(e) => go(e, x)"></div>"#),
    (
        "bind_globals",
        r#"<div :a="Math.max(1, y)" :b="$event" :c="_cache" :d="undefined" :e="arguments"></div>"#,
    ),
    (
        "bind_style_class",
        r#"<div :style="{ color: c }" :class="{ a: b }" class="s"></div>"#,
    ),
    (
        "bind_template_literal",
        "<div :title=\"`hi ${name}`\"></div>",
    ),
    ("handler_reference", r#"<div @click="handler"></div>"#),
    ("handler_padded", r#"<div @click=" go "></div>"#),
    ("handler_member", r#"<div @click="obj.method"></div>"#),
    ("handler_inline_update", r#"<div @click="count++"></div>"#),
    ("handler_inline_call", r#"<div @click="fn(x)"></div>"#),
    (
        "handler_arrow",
        r#"<div @focus="() => go(x)" @input="e => set(e)"></div>"#,
    ),
    ("handler_multi_statement", r#"<div @keyup="a; b"></div>"#),
    (
        "handler_trailing_comment",
        "<div @click=\"foo() // c\n\"></div>",
    ),
    (
        "handler_modifiers",
        r#"<div @click.stop="go(x)" @keyup.enter="submit"></div>"#,
    ),
    (
        "handler_assignment",
        r#"<div @click="selected = item"></div>"#,
    ),
    ("v_if", r#"<div v-if="ok">x</div>"#),
    (
        "v_if_chain_keys",
        r#"<div v-if="a" :key="k">x</div><div v-else-if="b" :key="kb">y</div>"#,
    ),
    (
        "v_for_simple",
        r#"<div v-for="item in list" :key="item.id">{{ item.a + x }}</div>"#,
    ),
    (
        "v_for_handler",
        r#"<div v-for="item in list" :key="item.id" @click="go(item, other)"></div>"#,
    ),
    (
        "v_for_object",
        r#"<div v-for="(v, k) in obj" :key="k">{{ v }}{{ k }}{{ obj }}</div>"#,
    ),
    (
        "v_for_destructured",
        r#"<div v-for="({ a }, i) in xs" :key="i">{{ a }}{{ b }}<span :[a]="1" @[b]="a"></span></div>"#,
    ),
    (
        "v_for_nested_source",
        r#"<div v-for="a in list"><span v-for="b in a.items">{{ b }}</span></div>"#,
    ),
    (
        "v_for_numeric",
        r#"<span v-for="n in 3">{{ n + x }}</span>"#,
    ),
    (
        "template_v_for_key",
        r#"<template v-for="i in n" :key="i"><span>{{ i }}</span></template>"#,
    ),
    ("v_model_native", r#"<input v-model="msg">"#),
    ("v_model_padded", r#"<input v-model=" msg ">"#),
    (
        "v_model_component",
        r#"<Foo v-model="val" v-model:x="y" />"#,
    ),
    ("v_model_dynamic_arg", r#"<Foo v-model:[prop]="val" />"#),
    (
        "dynamic_args",
        r#"<div :[key]="v" @[evt]="h" @[e2]="x++" :[k.a]="v"></div>"#,
    ),
    ("dynamic_component", r#"<component :is="tag" />"#),
    (
        "directives",
        r#"<div v-html="h" v-text="t" v-show="s" v-my="d" v-memo="[m]" v-my2:[arg].mod="v"></div>"#,
    ),
    (
        "slot_outlet",
        r#"<slot :name="n" :item="it" v-bind="obj" v-on="hs"></slot>"#,
    ),
    (
        "slot_scoped_default",
        r#"<Foo v-slot="{ item = def }">{{ item }}{{ other }}</Foo>"#,
    ),
    (
        "slot_template_scoped",
        r#"<Foo><template #row="{ r }">{{ r }} {{ q }}</template></Foo>"#,
    ),
    (
        "slot_dynamic_name",
        r#"<Foo><template #[dyn]="p">{{ p }}{{ dyn }}</template></Foo>"#,
    ),
    (
        "slot_scoped_handler",
        r#"<Foo><template #row="{ r }"><button @click="pick(r, x)">{{ r.label }}</button></template></Foo>"#,
    ),
    (
        "slot_scoped_nested_for",
        r#"<Foo><template #row="{ rows }"><span v-for="r in rows" :key="r.id">{{ r.v + y }}</span></template></Foo>"#,
    ),
    ("teleport", r#"<Teleport :to="tgt"><div>x</div></Teleport>"#),
    ("object_bind", r#"<div v-bind="obj" v-on="hs"></div>"#),
    (
        "conditional_slots",
        r#"<Foo><template v-if="ok" #a="{ p }">{{ p }}{{ q }}</template><template v-for="s in slots" #[s.name]>{{ s.x }}{{ y }}</template></Foo>"#,
    ),
    (
        "v_once_memo",
        r#"<div v-once>{{ a }}</div><div v-memo="[b]">{{ c }}</div>"#,
    ),
    ("svg_bind", r#"<svg><circle :r="radius" /></svg>"#),
    ("keyed_block", r#"<div :key="k">{{ x }}</div>"#),
    (
        "template_v_if_class",
        r#"<template v-if="ok"><span :class="c">{{ x }}</span><span>y</span></template>"#,
    ),
];

fn prefixed_options() -> DomCompilerOptions {
    DomCompilerOptions {
        prefix_identifiers: true,
        ..Default::default()
    }
}

const EMIT_PREFIXED: DomEmitOptions<'static> = DomEmitOptions {
    prefix_identifiers: true,
    ..DomEmitOptions::DEFAULT
};

#[test]
fn prefixed_identifiers_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &prefixed_options(),
        &CodegenOptions::default(),
        &EMIT_PREFIXED,
    );
}

#[test]
fn prefixed_identifiers_in_module_mode_match_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &DomCompilerOptions {
            mode: CodegenMode::Module,
            ..prefixed_options()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            ..EMIT_PREFIXED
        },
    );
}

#[test]
fn unprefixed_emit_under_explicit_options_is_unchanged() {
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &DomCompilerOptions::default(),
        &CodegenOptions::default(),
        &DomEmitOptions::DEFAULT,
    );
}
