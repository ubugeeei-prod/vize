//! P2-11 installment 98 witness: **when a cached element's props object
//! breaks over lines**.
//!
//! The shipped `genObjectExpression` writes an object multiline on two
//! arms: more than one property, *or* a property whose value is not a
//! `SimpleExpression`. `class` and `style` reach codegen as the objects
//! `transformElement` normalized them into, and `v-text` as a
//! `toDisplayString` call, so a lone one of those still breaks.
//!
//! This lane's normal props printer already carries both arms. The
//! cached/hoisted printer — the one that renders a constant element into
//! `_cache[n] || (_cache[n] = …)` — carried only the first, so a cached
//! `<span :style="{ … }">` came out on one line where the shipped lane
//! broke it. Compared byte-for-byte with the shipped lane.

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

/// Each case pairs a dynamic sibling — which keeps the tree from being
/// hoisted whole — with a constant element the inline lane caches.
const BATTERY: &[(&str, &str)] = &[
    // One property whose value is an object: the second arm alone.
    (
        "cached_style_object",
        r#"<div><Comp :format="fmt"/><span :style="{ marginLeft: 8 }">t</span></div>"#,
    ),
    (
        "cached_class_array",
        r#"<div><Comp :format="fmt"/><span :class="['a', 'b']">t</span></div>"#,
    ),
    (
        "cached_class_object",
        r#"<div><Comp :format="fmt"/><span :class="{ a: true }">t</span></div>"#,
    ),
    // The first arm on its own, and both together.
    (
        "cached_two_simple_props",
        r#"<div><Comp :format="fmt"/><span :id="'a'" :title="'b'">t</span></div>"#,
    ),
    (
        "cached_style_and_simple",
        r#"<div><Comp :format="fmt"/><span :style="{ marginLeft: 8 }" :id="'a'">t</span></div>"#,
    ),
    // Neither arm: a lone simple value stays on one line.
    (
        "cached_one_simple_prop",
        r#"<div><Comp :format="fmt"/><span :id="'a'">t</span></div>"#,
    ),
    // Static attributes take the other cached-props path entirely.
    (
        "cached_static_attr",
        r#"<div><Comp :format="fmt"/><span id="a">t</span></div>"#,
    ),
    (
        "cached_static_style_attr",
        r#"<div><Comp :format="fmt"/><span style="margin-left: 8px">t</span></div>"#,
    ),
    // `v-text` is the third non-simple value.
    (
        "cached_v_text",
        r#"<div><Comp :format="fmt"/><span v-text="'t'"></span></div>"#,
    ),
    // Nested inside a cached subtree rather than at its root.
    (
        "cached_nested_style",
        r#"<div><Comp :format="fmt"/><p><span :style="{ marginLeft: 8 }">t</span></p></div>"#,
    ),
    // The same shapes on a root the inline lane hoists rather than caches.
    (
        "root_style_object",
        r#"<span :style="{ marginLeft: 8 }">t</span>"#,
    ),
    (
        "root_two_simple_props",
        r#"<span :id="'a'" :title="'b'">t</span>"#,
    ),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("fmt", BindingType::SetupRef),
        ("Comp", BindingType::SetupConst),
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
fn cached_props_objects_break_over_lines_like_the_shipped_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// `inline` is what turns these constant elements into cached ones, so
/// the same battery has to agree with it off, where they stay ordinary
/// hoists.
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

/// The layout itself, pinned.
#[test]
fn a_lone_style_property_breaks_the_cached_object() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let span_block = |src: &str, take: usize| {
        let assembled = vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, true),
        )
        .expect("cached props witness must emit")
        .assembled();
        let lines = assembled
            .lines()
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>();
        let start = lines
            .iter()
            .position(|line| line.contains("\"span\""))
            .expect("the witness template emits a span");
        lines[start..]
            .iter()
            .take(take)
            .cloned()
            .collect::<Vec<_>>()
    };
    // One object-valued property, so the second arm alone breaks it.
    assert_eq!(
        span_block(
            r#"<div><Comp :format="fmt"/><span :style="{ marginLeft: 8 }">t</span></div>"#,
            3
        ),
        vec![
            "_cache[0] || (_cache[0] = _createElementVNode(\"span\", {".to_string(),
            "style: { marginLeft: 8 }".to_string(),
            "}, \"t\", -1 /* CACHED */))".to_string(),
        ]
    );
    // One simple value, so neither arm applies and it stays on one line.
    assert_eq!(
        span_block(r#"<div><Comp :format="fmt"/><span :id="'a'">t</span></div>"#, 1),
        vec![
            "_cache[0] || (_cache[0] = _createElementVNode(\"span\", { id: 'a' }, \"t\", -1 /* CACHED */))"
                .to_string(),
        ]
    );
}
