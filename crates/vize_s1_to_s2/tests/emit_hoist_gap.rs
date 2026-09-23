//! P3-17: inline root props that the shipped lane hoists must hoist here.
//!
//! `is_constant_simple_expression` admits a value whose identifiers are
//! locals it binds (`(v) => v.id`). The S2 hoist pass now admits that same
//! class, so the inline root arm spells `_hoisted_N` on both lanes. A value
//! the pass still under-classifies (a local mixed with a free name) stays
//! refused rather than guessed.

#![expect(clippy::panic, reason = "tests assert by panicking")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_legacy_with_options};
use vize_s0::Allocator;
use vize_s1_to_s2::{
    BindingKind, BindingTable, DomEmitMode, DomEmitOptions, EmitError, LegacyCaps,
    UnsupportedReason as Reason, emit_dom_source_with_options,
};

const BINDINGS: &[(&str, BindingType, BindingKind)] = &[
    ("Card", BindingType::SetupConst, BindingKind::SetupConst),
    (
        "Form",
        BindingType::SetupMaybeRef,
        BindingKind::SetupMaybeRef,
    ),
    (
        "Link",
        BindingType::SetupMaybeRef,
        BindingKind::SetupMaybeRef,
    ),
    ("items", BindingType::SetupRef, BindingKind::SetupRef),
    ("draft", BindingType::SetupLet, BindingKind::SetupLet),
];

fn s2(source: &str, inline: bool) -> Result<String, EmitError> {
    s2_with_ts(source, inline, false)
}

fn s2_with_ts(source: &str, inline: bool, is_ts: bool) -> Result<String, EmitError> {
    let table = BindingTable::new(
        BINDINGS.iter().map(|(name, _, kind)| (*name, *kind)),
        [],
        true,
    );
    let allocator = Allocator::new();
    emit_dom_source_with_options(
        &allocator,
        source,
        LegacyCaps::VUE3,
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            prefix_identifiers: true,
            inline,
            cache_handlers: inline,
            is_ts,
            bindings: Some(&table),
            ..DomEmitOptions::DEFAULT
        },
    )
    .map(|emit| emit.assembled().to_string())
}

fn shipped(source: &str, inline: bool) -> String {
    shipped_with_ts(source, inline, false)
}

fn shipped_with_ts(source: &str, inline: bool, is_ts: bool) -> String {
    let mut metadata = BindingMetadata {
        is_script_setup: true,
        ..Default::default()
    };
    for (name, kind, _) in BINDINGS {
        metadata.bindings.insert((*name).into(), *kind);
    }
    let allocator = Allocator::new();
    let (_, errors, old) = compile_template_legacy_with_options(
        &allocator,
        source,
        DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            inline,
            cache_handlers: inline,
            is_ts,
            binding_metadata: Some(metadata),
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{source:?}: {errors:?}");
    format!("{}\n{}", old.preamble, old.code)
}

#[test]
fn typed_component_callback_props_match_shipped_hoists() {
    let cases = [
        // The shipped hoist check still sees the typed local declaration in
        // the callback body, so this nested Form keeps its props inline.
        r#"<div><Form action="/users" method="post" :optimistic="(props, formData) => { const name: string = formData.name; return { ...props, name } }" :transform="(data) => { const email: string = data.email; return { email } }"><input name="name" /><input name="email" /></Form></div>"#,
        // A typed parameter is erased before the shipped constant walk; its
        // self-bound name and allowed global together permit the root hoist.
        r#"<Form action="/form-component/view-transition" method="post" :options="{ viewTransition: (transition: ViewTransition) => { transition.ready.then(() => console.log('ready')) } }"><button type="submit">Submit</button></Form>"#,
        r#"<Link href="/view-transition/page-b" :view-transition="(transition: ViewTransition) => { transition.ready.then(() => console.log('ready')) }">Link to Page B</Link>"#,
    ];
    for source in cases {
        let legacy = shipped_with_ts(source, true, true);
        let emitted = s2_with_ts(source, true, true)
            .unwrap_or_else(|error| panic!("{source:?} must emit, got {error:?}"));
        assert_eq!(emitted, legacy, "{source:?}");
    }
}

#[test]
fn typed_parameter_and_local_callback_props_match_shipped_hoists() {
    let source = r#"<Form :handler="(x: number) => { const y: number = x; return y }" />"#;
    let legacy = shipped_with_ts(source, true, true);
    let emitted = s2_with_ts(source, true, true).expect("S2 emit");
    assert_eq!(emitted, legacy, "{source:?}");
}

#[test]
fn conditional_named_slot_outlet_props_match_shipped_hoists() {
    let source = r#"<Tabs><template #apiTab><div class="py-2"><slot name="api" /></div></template><template v-if="showInstallation" #creditsTab><div class="py-2"><slot name="credits" /></div></template></Tabs>"#;
    let legacy = shipped(source, true);
    let emitted = s2(source, true).unwrap_or_else(|error| panic!("{source:?}: {error:?}"));
    assert_eq!(emitted, legacy, "{source:?}");
}

fn refused(source: &str) -> Reason {
    match s2(source, true) {
        Err(EmitError::Unsupported(refusal)) => refusal.reason,
        other => panic!("{source:?} must refuse on the inline lane, got {other:?}"),
    }
}

#[test]
fn inline_root_surfaces_the_shipped_lane_hoists_match() {
    let cases = [
        (
            r#"<Card :format="(row) => row.name" />"#,
            "import { openBlock as _openBlock, createBlock as _createBlock } from \"vue\"\n\nconst _hoisted_1 = { format: (row) => row.name }\n\nexport function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (_openBlock(), _createBlock(Card, _hoisted_1, null, 8 /* PROPS */, [\"format\"]))\n}",
        ),
        (
            r#"<Card title="a" :format="(v) => v.toFixed(2)" /><Card :format="(v) => v" />"#,
            "import { createVNode as _createVNode, openBlock as _openBlock, createElementBlock as _createElementBlock, Fragment as _Fragment } from \"vue\"\n\nconst _hoisted_1 = { title: \"a\", format: (v) => v.toFixed(2) }\nconst _hoisted_2 = { format: (v) => v }\n\nexport function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (_openBlock(), _createElementBlock(_Fragment, null, [\n    _createVNode(Card, _hoisted_1, null, 8 /* PROPS */, [\"format\"]),\n    _createVNode(Card, _hoisted_2, null, 8 /* PROPS */, [\"format\"])\n  ], 64 /* STABLE_FRAGMENT */))\n}",
        ),
        (
            r#"<div :format="(v) => v.toFixed(2)" class="a"></div>"#,
            "import { openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\"\n\nconst _hoisted_1 = { format: (v) => v.toFixed(2), class: \"a\" }\n\nexport function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1, null, 8 /* PROPS */, [\"format\"]))\n}",
        ),
    ];
    for (source, expected) in cases {
        let legacy = shipped(source, true);
        assert_eq!(legacy, expected, "{source:?}");
        let emitted =
            s2(source, true).unwrap_or_else(|error| panic!("{source:?} must emit, got {error:?}"));
        assert_eq!(emitted, legacy, "{source:?}");
    }
    // A local mixed with a free name is still under-classified, so the
    // inline lane refuses instead of spelling a different props object.
    let mixed = r#"<Card :format="(row) => Math.max(row, 1)" />"#;
    assert_eq!(
        shipped(mixed, true),
        "import { openBlock as _openBlock, createBlock as _createBlock } from \"vue\"\n\nconst _hoisted_1 = { format: (row) => Math.max(row, 1) }\n\nexport function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (_openBlock(), _createBlock(Card, _hoisted_1, null, 8 /* PROPS */, [\"format\"]))\n}"
    );
    assert_eq!(refused(mixed), Reason::HoistConstantGap);
}

#[test]
fn surfaces_outside_the_gap_keep_emitting_with_shipped_parity() {
    for (source, inline) in [
        // Non-inline lanes keep their corpus-proven output.
        (r#"<Card :format="(row) => row.name" />"#, false),
        // A dynamic sibling value blocks the shipped hoist too.
        (
            r#"<Card :format="(row) => row.name" :items="items" />"#,
            true,
        ),
        // Nested surfaces are outside the inline root arm.
        (
            r#"<section><Card :format="(row) => row.name" /></section>"#,
            true,
        ),
        // A value reading a free name was never constant.
        (r#"<Card :format="(row) => items[row]" />"#, true),
    ] {
        let emitted = s2(source, inline)
            .unwrap_or_else(|error| panic!("{source:?} (inline={inline}): {error:?}"));
        assert_eq!(
            emitted,
            shipped(source, inline),
            "{source:?} (inline={inline})"
        );
    }
}

#[test]
fn unref_across_reordered_slot_objects_matches_the_shipped_lane() {
    // Named slot templates print ahead of the default content they follow
    // in source. `_unref` still registers at the first authored read.
    let source = r#"<Card><p v-if="items">{{ draft }}</p><template #footer><i>{{ draft }}</i></template></Card>"#;
    let emitted = s2(source, true).unwrap_or_else(|error| panic!("{source:?}: {error:?}"));
    assert_eq!(emitted, shipped(source, true), "{source:?}");
    // Without `_unref`, or without a named template, the order is shared.
    for source in [
        r#"<Card><p v-if="items">{{ items }}</p><template #footer><i>{{ items }}</i></template></Card>"#,
        r#"<Card><p>{{ draft }}</p><template #footer><i>{{ draft }}</i></template></Card>"#,
        r#"<Card><p>{{ draft }}</p></Card>"#,
    ] {
        let emitted = s2(source, true).unwrap_or_else(|error| panic!("{source:?}: {error:?}"));
        assert_eq!(emitted, shipped(source, true), "{source:?}");
    }
}
