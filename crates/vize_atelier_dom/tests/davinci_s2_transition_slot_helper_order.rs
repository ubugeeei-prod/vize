//! P2-11 installment 100 witness: **where the explicit-`Transition`-slot
//! arm registers its helpers**.
//!
//! A component whose default children carry a `TransitionGroup` and whose
//! named slot carries an explicit `<Transition>` needs `renderList`,
//! `withCtx`, `Transition` and `vShow` — and the shipped transform
//! registers each at the op that carries it, deep inside the slot bodies.
//! This lane preferred all four on the *carrier* itself, before the walk
//! descended, which put them ahead of anything a descendant registers.
//!
//! Nothing observed it until `inline` produced `_unref`: `unref` is
//! registered at the op whose expression needs it, so a deep interpolation
//! reading a maybe-ref lands *between* the carrier and its descendants —
//! before those four in the shipped lane, after them here. The four are
//! now preferred once the walk has descended. Compared byte-for-byte with
//! the shipped lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_s0::FxHashMap;
use vize_s1_to_s2::{BindingKind, BindingTable, DomEmitMode, DomEmitOptions};

const BATTERY: &[(&str, &str)] = &[
    // The shape that reorders: an implicit `TransitionGroup` beside an
    // explicit `<Transition>` slot, with a maybe-ref read between them.
    (
        "transition_slot_with_group_and_unref",
        r#"<MyComp><TransitionGroup><p v-for="i in items" :key="i">{{ maybe.a }}</p></TransitionGroup><template #footer><Transition><span v-show="open">f</span></Transition></template></MyComp>"#,
    ),
    // The maybe-ref read ahead of the group, so `unref` precedes all four.
    (
        "unref_before_transition_group",
        r#"<MyComp><b>{{ maybe.a }}</b><TransitionGroup><p v-for="i in items" :key="i">{{ i }}</p></TransitionGroup><template #footer><Transition><span v-show="open">f</span></Transition></template></MyComp>"#,
    ),
    // No maybe-ref at all: the four keep their order among themselves.
    (
        "transition_slot_with_group_no_unref",
        r#"<MyComp><TransitionGroup><p v-for="i in items" :key="i">{{ i }}</p></TransitionGroup><template #footer><Transition><span v-show="open">f</span></Transition></template></MyComp>"#,
    ),
    // The arm's two halves on their own — neither fires it.
    (
        "transition_slot_without_group",
        r#"<MyComp><b>{{ maybe.a }}</b><template #footer><Transition><span v-show="open">f</span></Transition></template></MyComp>"#,
    ),
    (
        "group_without_transition_slot",
        r#"<MyComp><TransitionGroup><p v-for="i in items" :key="i">{{ maybe.a }}</p></TransitionGroup></MyComp>"#,
    ),
    // A named slot carrying the group instead of the default children.
    (
        "group_in_named_slot",
        r#"<MyComp><template #body><TransitionGroup><p v-for="i in items" :key="i">{{ maybe.a }}</p></TransitionGroup></template><template #footer><Transition><span v-show="open">f</span></Transition></template></MyComp>"#,
    ),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("items", BindingType::SetupRef),
        ("open", BindingType::SetupRef),
        ("maybe", BindingType::SetupMaybeRef),
        ("MyComp", BindingType::SetupConst),
    ];
    let mut bindings = FxHashMap::default();
    for (name, kind) in entries {
        bindings.insert((*name).into(), *kind);
    }
    BindingMetadata {
        bindings,
        props_aliases: FxHashMap::default(),
        is_script_setup: true,
    }
}

fn binding_kind(kind: BindingType) -> BindingKind {
    match kind {
        BindingType::SetupLet => BindingKind::SetupLet,
        BindingType::SetupMaybeRef => BindingKind::SetupMaybeRef,
        BindingType::SetupRef => BindingKind::SetupRef,
        BindingType::SetupReactiveConst => BindingKind::SetupReactiveConst,
        BindingType::SetupConst => BindingKind::SetupConst,
        BindingType::Props => BindingKind::Props,
        BindingType::PropsAliased => BindingKind::PropsAliased,
        BindingType::Data => BindingKind::Data,
        BindingType::Options => BindingKind::Options,
        BindingType::LiteralConst => BindingKind::LiteralConst,
        BindingType::JsGlobalUniversal => BindingKind::JsGlobalUniversal,
        BindingType::JsGlobalBrowser => BindingKind::JsGlobalBrowser,
        BindingType::JsGlobalNode => BindingKind::JsGlobalNode,
        BindingType::JsGlobalDeno => BindingKind::JsGlobalDeno,
        BindingType::JsGlobalBun => BindingKind::JsGlobalBun,
        BindingType::VueGlobal => BindingKind::VueGlobal,
        BindingType::ExternalModule => BindingKind::ExternalModule,
    }
}

fn table(metadata: &BindingMetadata) -> BindingTable {
    BindingTable::new(
        metadata
            .bindings
            .iter()
            .map(|(name, kind)| (name.as_str(), binding_kind(*kind))),
        [],
        metadata.is_script_setup,
    )
}

fn dom_options(metadata: &BindingMetadata, inline: bool) -> DomCompilerOptions {
    DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        inline,
        binding_metadata: Some(metadata.clone()),
        ..Default::default()
    }
}

fn emit_options(table: &BindingTable, inline: bool) -> DomEmitOptions<'_> {
    DomEmitOptions {
        mode: DomEmitMode::Module,
        prefix_identifiers: true,
        inline,
        bindings: Some(table),
        ..DomEmitOptions::DEFAULT
    }
}

#[test]
fn transition_slot_helpers_register_where_the_shipped_lane_registers_them() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// Without `inline` there is no `_unref` to sit between the carrier and
/// its descendants, so the same battery has to keep agreeing — the arm's
/// helpers move, and nothing else may.
#[test]
fn the_same_battery_agrees_without_inline() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, false),
        &CodegenOptions::default(),
        &emit_options(&table, false),
    );
}

/// The rule itself, pinned in both directions: `unref` registers at the
/// op whose expression needs it, in source order, so it follows
/// `renderList` when the read sits inside the loop and precedes the
/// carrier's four when it is authored ahead of them.
#[test]
fn unref_registers_in_source_order_against_the_slot_helpers() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let order = |src: &str| {
        let assembled = vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, true),
        )
        .expect("transition slot witness must emit")
        .assembled();
        let line = assembled
            .lines()
            .next()
            .expect("the module preamble is the first line")
            .to_string();
        let aliases = line
            .split(", ")
            .filter_map(|part| part.split(" as ").nth(1))
            .map(|alias| alias.trim_end_matches(" } from \"vue\"").to_string())
            .collect::<Vec<_>>();
        let at = |name: &str| aliases.iter().position(|alias| alias == name);
        (at("_unref"), at("_renderList"))
    };
    // The maybe-ref read sits *inside* the `v-for`, so the loop's own
    // `renderList` is registered first.
    let (unref, render_list) = order(BATTERY[0].1);
    assert_eq!(
        (unref.is_some(), render_list.is_some(), unref > render_list),
        (true, true, true)
    );
    // Authored ahead of the group, the same read registers first — the
    // ordering the carrier's four used to jump ahead of.
    let (unref, render_list) = order(BATTERY[1].1);
    assert_eq!(
        (unref.is_some(), render_list.is_some(), unref < render_list),
        (true, true, true)
    );
}
