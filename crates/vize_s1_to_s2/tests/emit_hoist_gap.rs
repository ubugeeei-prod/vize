//! P3-17: the inline root props-hoist gap is refused, never guessed.
//!
//! The shipped lane hoists a root props surface of an inlined render
//! function when every value passes `is_constant_simple_expression` — which
//! admits a value reading only locals it binds itself (`(v) => v.id`). The S2
//! hoist pass classifies such a value as dynamic, so S2 would spell the props
//! inline. Measured on the production-path parity oracle
//! (`vize_atelier_sfc/tests/davinci_production_reach.rs`), the emitter now
//! refuses exactly the root surfaces of an inline render on which the two
//! classifiers disagree, and keeps emitting everything else.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
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
    ("items", BindingType::SetupRef, BindingKind::SetupRef),
    ("draft", BindingType::SetupLet, BindingKind::SetupLet),
];

fn s2(source: &str, inline: bool) -> Result<String, EmitError> {
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
            bindings: Some(&table),
            ..DomEmitOptions::DEFAULT
        },
    )
    .map(|emit| emit.assembled().to_string())
}

fn shipped(source: &str, inline: bool) -> String {
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
            binding_metadata: Some(metadata),
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{source:?}: {errors:?}");
    format!("{}\n{}", old.preamble, old.code)
}

fn refused(source: &str) -> Reason {
    match s2(source, true) {
        Err(EmitError::Unsupported(refusal)) => refusal.reason,
        other => panic!("{source:?} must refuse on the inline lane, got {other:?}"),
    }
}

#[test]
fn inline_root_surfaces_the_shipped_lane_hoists_are_refused() {
    for source in [
        r#"<Card :format="(row) => row.name" />"#,
        r#"<Card title="a" :format="(v) => v.toFixed(2)" /><Card :format="(v) => v" />"#,
        r#"<div :format="(v) => v.toFixed(2)" class="a"></div>"#,
    ] {
        assert_ne!(
            shipped(source, true).find("_hoisted_1"),
            None,
            "{source:?}: the shipped inline lane hoists this surface"
        );
        assert_eq!(refused(source), Reason::HoistConstantGap, "{source:?}");
    }
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
fn unref_across_reordered_slot_objects_is_refused_on_the_inline_lane() {
    // Named slot templates print ahead of the default content they follow
    // in source; `_unref`'s registration point then differs from the
    // shipped transform's, so the inline lane refuses the template.
    let source = r#"<Card><p v-if="items">{{ draft }}</p><template #footer><i>{{ draft }}</i></template></Card>"#;
    assert_eq!(refused(source), Reason::UnrefAcrossReorderedSlots);
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
