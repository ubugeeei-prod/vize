//! P2-11 installment 97 witness: **`is_constant_interpolation` over a
//! text run**.
//!
//! An interpolation that reads a single `LiteralConst` / `SetupConst`
//! binding never updates, so the shipped lane leaves the `TEXT` patch
//! flag off the element — and it asks that of *every* interpolation
//! child, one at a time. S2 folds a mixed text run into one compound
//! op, so the joined source (`"a" + count + "b"`) never names a binding
//! and the question has to be asked per dynamic part instead. Compared
//! byte-for-byte with the shipped lane.

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
    // A lone interpolation: the shape that already worked.
    ("lone_literal_const", "<div>{{ LIMIT }}</div>"),
    ("lone_setup_const", "<div>{{ handler }}</div>"),
    ("lone_ref", "<div>{{ count }}</div>"),
    // A text run around one constant read.
    ("run_literal_const", "<div>a{{ LIMIT }}b</div>"),
    ("run_setup_const", "<div>a{{ handler }}b</div>"),
    ("run_leading_text", "<div>a{{ LIMIT }}</div>"),
    ("run_trailing_text", "<div>{{ LIMIT }}b</div>"),
    // Two constants in one run.
    ("run_two_constants", "<div>{{ LIMIT }} {{ handler }}</div>"),
    (
        "run_two_constants_text",
        "<div>a{{ LIMIT }}b{{ handler }}c</div>",
    ),
    // One dynamic read anywhere keeps the flag.
    ("run_one_ref", "<div>a{{ count }}b</div>"),
    ("run_const_then_ref", "<div>a{{ LIMIT }}b{{ count }}c</div>"),
    ("run_ref_then_const", "<div>a{{ count }}b{{ LIMIT }}c</div>"),
    ("run_let", "<div>a{{ msg }}b</div>"),
    ("run_prop", "<div>a{{ title }}b</div>"),
    ("run_unknown", "<div>a{{ other }}b</div>"),
    ("run_reactive_const", "<div>a{{ state.id }}b</div>"),
    // A member read on a constant is not a bare name, so it is dynamic
    // on both sides.
    ("run_const_member", "<div>a{{ LIMIT.x }}b</div>"),
    ("run_const_call", "<div>a{{ handler() }}b</div>"),
    // Nested elements and siblings around the run.
    (
        "run_inside_nested",
        "<section><div>a{{ LIMIT }}b</div></section>",
    ),
    (
        "run_beside_element",
        "<div><span>x</span>a{{ LIMIT }}b</div>",
    ),
    // Components take the same children path.
    ("component_run", "<MyComp>a{{ LIMIT }}b</MyComp>"),
    ("component_run_ref", "<MyComp>a{{ count }}b</MyComp>"),
    // Slots and loops keep their own scopes; a `v-for` alias is never
    // constant.
    (
        "slot_run",
        "<MyComp><template #x>a{{ LIMIT }}b</template></MyComp>",
    ),
    (
        "v_for_alias_run",
        r#"<li v-for="i in items" :key="i">a{{ i }}b</li>"#,
    ),
    (
        "v_for_const_run",
        r#"<li v-for="i in items" :key="i">a{{ LIMIT }}b</li>"#,
    ),
    // `v-once` / `v-memo` neighbours read the same run.
    ("once_run", "<div v-once>a{{ LIMIT }}b</div>"),
    ("memo_run", r#"<div v-memo="[count]">a{{ LIMIT }}b</div>"#),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("LIMIT", BindingType::LiteralConst),
        ("handler", BindingType::SetupConst),
        ("count", BindingType::SetupRef),
        ("msg", BindingType::SetupLet),
        ("state", BindingType::SetupReactiveConst),
        ("title", BindingType::Props),
        ("items", BindingType::SetupConst),
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
fn constant_text_runs_match_the_shipped_dom_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// The rule only fires where the transform leaves the binding bare, so
/// the same battery must agree with `inline` off — where every read is a
/// `$setup.` member and every run keeps its `TEXT` flag.
#[test]
fn the_same_battery_agrees_without_the_inline_option() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, false),
        &CodegenOptions::default(),
        &emit_options(&table, false),
    );
}

/// The flag itself, pinned on both sides of the option.
#[test]
fn a_run_of_constant_reads_carries_no_text_flag() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let render = |src: &str, inline: bool| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, inline),
        )
        .expect("constant text run witness must emit")
        .assembled()
        .lines()
        .find(|line| line.contains("return "))
        .unwrap_or_default()
        .trim()
        .to_string()
    };
    assert_eq!(
        render("<div>a{{ LIMIT }}b</div>", true),
        "return (_openBlock(), _createElementBlock(\"div\", null, \"a\" + _toDisplayString(LIMIT) + \"b\"))"
    );
    assert_eq!(
        render("<div>a{{ LIMIT }}b</div>", false),
        "return (_openBlock(), _createElementBlock(\"div\", null, \"a\" + _toDisplayString($setup.LIMIT) + \"b\", 1 /* TEXT */))"
    );
    // One dynamic part anywhere in the run keeps the flag.
    assert_eq!(
        render("<div>a{{ LIMIT }}b{{ count }}c</div>", true),
        "return (_openBlock(), _createElementBlock(\"div\", null, \"a\" + _toDisplayString(LIMIT) + \"b\" + _toDisplayString(count.value) + \"c\", 1 /* TEXT */))"
    );
}
