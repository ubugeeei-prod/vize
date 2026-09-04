//! P2-11 installment 89 witness: **`inline`**. A `<script setup>` SFC
//! inlines its render function into `setup()`, so the template reads the
//! setup bindings straight from the closure — refs through `.value`,
//! `let`/maybe-ref bindings through `_unref(…)`, props through
//! `__props.` — instead of the `$setup` proxy. Compared byte-for-byte
//! with the shipped lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_s1_to_s2::{DomEmitMode, DomEmitOptions};

const BATTERY: &[(&str, &str)] = &[
    ("setup_ref_read", "<div>{{ count }}</div>"),
    ("setup_ref_member", "<div>{{ count.toFixed(2) }}</div>"),
    ("setup_let_read", "<div>{{ msg }}</div>"),
    ("setup_maybe_ref_read", "<div>{{ theme }}</div>"),
    ("setup_const_read", "<div>{{ handler }}</div>"),
    ("setup_reactive_read", "<div>{{ state.id }}</div>"),
    ("literal_const_read", "<div>{{ LIMIT }}</div>"),
    ("props_read", "<div>{{ title }}</div>"),
    ("unknown_read", "<div>{{ other }}</div>"),
    ("vue_global_read", "<div>{{ $slots.default }}</div>"),
    ("mixed_text", "<p>{{ count }} {{ title }} {{ other }}</p>"),
    ("bind_ref", r#"<div :id="count"></div>"#),
    ("bind_let", r#"<div :id="msg"></div>"#),
    ("bind_props", r#"<div :id="title"></div>"#),
    ("bind_expression", r#"<div :id="count + LIMIT"></div>"#),
    (
        "bind_shorthand_object",
        r#"<div :o="{ count, msg, title, other }"></div>"#,
    ),
    // The arrow keeps a runtime dependency: a dependency-free one is
    // hoisted by the shipped lane through a constness rule S2 does not
    // carry yet, which reproduces with `inline` off.
    (
        "bind_arrow_shadow",
        r#"<div :fn="(count) => other(count)"></div>"#,
    ),
    ("handler_const", r#"<div @click="handler"></div>"#),
    ("handler_ref", r#"<div @click="count"></div>"#),
    ("handler_call", r#"<div @click="handler(count)"></div>"#),
    ("handler_increment", r#"<div @click="count++"></div>"#),
    ("handler_assign_ref", r#"<div @click="count = 1"></div>"#),
    ("handler_assign_let", r#"<div @click="msg = 'x'"></div>"#),
    (
        "handler_statements",
        r#"<div @click="count++; msg = 'x'"></div>"#,
    ),
    ("model_ref", r#"<input v-model="count">"#),
    ("model_let", r#"<input v-model="msg">"#),
    ("model_member", r#"<input v-model="state.name">"#),
    ("model_component", r#"<MyComp v-model="count" />"#),
    ("vif_ref", r#"<p v-if="count">{{ msg }}</p>"#),
    ("vshow_let", r#"<div v-show="msg"></div>"#),
    (
        "vfor_over_ref",
        r#"<li v-for="i in count" :key="i">{{ i }}</li>"#,
    ),
    (
        "vfor_alias_shadows",
        r#"<li v-for="count in items" :key="count">{{ count }} {{ msg }}</li>"#,
    ),
    ("vhtml_let", r#"<div v-html="msg"></div>"#),
    ("vtext_ref", r#"<p v-text="count"></p>"#),
    ("vmemo_ref", r#"<div v-memo="[count]">{{ msg }}</div>"#),
    ("vonce_ref", "<div v-once>{{ count }}</div>"),
    ("component_const", "<MyComp />"),
    ("component_let", "<LetComp />"),
    ("component_ref", "<RefComp />"),
    ("component_external", "<FooBar />"),
    ("component_unknown", "<Other />"),
    ("component_dotted", "<Ns.Item />"),
    ("directive_const", "<div v-focus></div>"),
    ("directive_let", "<div v-my-dir></div>"),
    ("directive_unknown", "<div v-other></div>"),
    (
        "slot_param_shadows",
        r#"<MyComp v-slot="{ count }">{{ count }} {{ msg }}</MyComp>"#,
    ),
    ("slot_outlet", r#"<slot :item="count">{{ msg }}</slot>"#),
    ("dynamic_bind_key", r#"<div :[key]="count"></div>"#),
    ("static_children", "<div><span>a</span><span>b</span></div>"),
    (
        "static_with_dynamic",
        r#"<div><span>a</span><em>{{ count }}</em></div>"#,
    ),
];

fn metadata() -> BindingMetadata {
    support::bindings::script_setup_metadata(&[
        ("count", BindingType::SetupRef),
        ("msg", BindingType::SetupLet),
        ("theme", BindingType::SetupMaybeRef),
        ("handler", BindingType::SetupConst),
        ("state", BindingType::SetupReactiveConst),
        ("LIMIT", BindingType::LiteralConst),
        ("title", BindingType::Props),
        ("$slots", BindingType::VueGlobal),
        ("FooBar", BindingType::ExternalModule),
        ("MyComp", BindingType::SetupConst),
        ("LetComp", BindingType::SetupLet),
        ("RefComp", BindingType::SetupRef),
        ("Ns", BindingType::SetupConst),
        ("vFocus", BindingType::SetupConst),
        ("vMyDir", BindingType::SetupLet),
        ("items", BindingType::SetupConst),
        ("key", BindingType::SetupRef),
    ])
}

#[test]
fn inline_mode_matches_the_shipped_dom_lane() {
    let metadata = metadata();
    let table = support::bindings::binding_table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            inline: true,
            binding_metadata: Some(metadata.clone()),
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            prefix_identifiers: true,
            inline: true,
            bindings: Some(&table),
            ..DomEmitOptions::DEFAULT
        },
    );
}

/// The distinctive inline spellings, pinned: a lane that ignored the
/// option would still pass the dual run above if the shipped side
/// regressed with it.
#[test]
fn inline_spellings_are_pinned() {
    let metadata = metadata();
    let table = support::bindings::binding_table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let options = DomEmitOptions {
        mode: DomEmitMode::Module,
        prefix_identifiers: true,
        inline: true,
        bindings: Some(&table),
        ..DomEmitOptions::DEFAULT
    };
    let body = |src: &str| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &options,
        )
        .expect("inline witness must emit")
        .assembled()
        .lines()
        .find(|line| line.contains("return "))
        .unwrap_or_default()
        .trim()
        .to_string()
    };
    // A ref reads through `.value`, a `let` through `_unref`, a prop off
    // the setup-local `__props`, and an unknown name still through `_ctx`.
    assert_eq!(
        body("<div>{{ count }} {{ msg }} {{ title }} {{ other }}</div>"),
        "return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(count.value) + \" \" + _toDisplayString(_unref(msg)) + \" \" + _toDisplayString(__props.title) + \" \" + _toDisplayString(_ctx.other), 1 /* TEXT */))"
    );
    // A shorthand object expands rather than becoming `{ count.value }`.
    assert_eq!(
        body(r#"<div :o="{ count, msg }"></div>"#),
        "return (_openBlock(), _createElementBlock(\"div\", { o: { count: count.value, msg: _unref(msg) } }, null, 8 /* PROPS */, [\"o\"]))"
    );
    // A component binding reads the closure directly, through `_unref`
    // when the script may rebind it.
    assert_eq!(
        body("<MyComp />"),
        "return (_openBlock(), _createBlock(MyComp))"
    );
    assert_eq!(
        body("<LetComp />"),
        "return (_openBlock(), _createBlock(_unref(LetComp)))"
    );
    // A constant binding read is not a patch target at all.
    assert_eq!(
        body("<div>{{ handler }}</div>"),
        "return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(handler)))"
    );
}
