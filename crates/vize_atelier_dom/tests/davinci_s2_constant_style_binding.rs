//! P2-11 installment 94 witness: **`:style` over a constant binding**.
//!
//! The shipped gate for `normalizeStyle` is
//! `is_constant_simple_expression`, not a literal check: its
//! `RuntimeDependencyVisitor` lets through every free identifier that
//! resolves to a *constant* script binding, an allowed global, a runtime
//! helper alias, or a local the expression itself binds. So
//! `:style="theme"` over a `const theme = { … }` skips the helper, the
//! same way an inline object literal does. The port only recognized
//! literals. Compared byte-for-byte with the shipped lane.
//!
//! The rule reads the *prefixed* content on the shipped side, so it only
//! fires where the prefixer leaves a name bare — an inlined render
//! function. The same battery with `inline` off must keep every
//! `_normalizeStyle`.

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
    // Constant script bindings: the helper is skipped.
    ("style_setup_const", r#"<div :style="theme"></div>"#),
    ("style_literal_const", r#"<div :style="LIMIT"></div>"#),
    ("style_external_module", r#"<div :style="imported"></div>"#),
    ("style_const_member", r#"<div :style="theme.box"></div>"#),
    (
        "style_const_object",
        r#"<div :style="{ color: theme.color }"></div>"#,
    ),
    (
        "style_const_array",
        r#"<div :style="[theme, LIMIT]"></div>"#,
    ),
    // Non-constant bindings keep it.
    ("style_setup_ref", r#"<div :style="count"></div>"#),
    ("style_setup_let", r#"<div :style="msg"></div>"#),
    ("style_reactive_const", r#"<div :style="state"></div>"#),
    ("style_prop", r#"<div :style="title"></div>"#),
    ("style_unknown", r#"<div :style="other"></div>"#),
    ("style_vue_global", r#"<div :style="$attrs"></div>"#),
    // Mixed: one dynamic name anywhere makes the whole expression
    // dynamic.
    (
        "style_const_and_ref",
        r#"<div :style="[theme, count]"></div>"#,
    ),
    (
        "style_const_object_dynamic_value",
        r#"<div :style="{ color: count }"></div>"#,
    ),
    // A call on a constant binding is still constant to the visitor;
    // its *result* is what the runtime recomputes.
    ("style_const_call", r#"<div :style="build()"></div>"#),
    ("style_dynamic_call", r#"<div :style="msg()"></div>"#),
    // Locals the expression binds itself. (A `:style` whose value is
    // *only* globals — `Math.random` — is a different gap: the shipped
    // lane reads the same constant predicate from `has_static_props` and
    // hoists the whole props object, which the S2 pass's own staticness
    // rule does not do yet. That is its own installment; this one is the
    // normalize gate.)
    (
        "style_local_arrow",
        r#"<div :style="((x) => x)(theme)"></div>"#,
    ),
    (
        "style_local_shadows_ref",
        r#"<div :style="((count) => count)(theme)"></div>"#,
    ),
    // A `v-for` alias is bound by the render function, not the script.
    (
        "style_v_for_alias",
        r#"<li v-for="i in items" :key="i" :style="i"></li>"#,
    ),
    (
        "style_v_for_const_inside",
        r#"<li v-for="i in items" :key="i" :style="theme"></li>"#,
    ),
    // Literals still take the old path.
    (
        "style_literal_object",
        r#"<div :style="{ color: 'red' }"></div>"#,
    ),
    (
        "style_literal_string",
        r#"<div :style="'color:red'"></div>"#,
    ),
    // A static `style` attribute merges beside the bound one.
    (
        "style_static_and_literal",
        r#"<div style="color:red" :style="{ color: 'blue' }"></div>"#,
    ),
    (
        "style_static_and_ref",
        r#"<div style="color:red" :style="count"></div>"#,
    ),
    (
        "style_static_and_const",
        r#"<div style="color:red" :style="theme"></div>"#,
    ),
    (
        "component_style_static_and_const",
        r#"<MyComp style="color:red" :style="theme" />"#,
    ),
    // Components take the same gate through their own props path.
    ("component_style_const", r#"<MyComp :style="theme" />"#),
    ("component_style_ref", r#"<MyComp :style="count" />"#),
    // `class` has its own helper and is untouched by this rule.
    ("class_const_binding", r#"<div :class="theme"></div>"#),
    // Shadowing, pinned against the shipped visitor rather than against
    // intuition. `RuntimeDependencyVisitor` tracks what a *function*
    // binds, so a name a parameter or a declaration inside one shadows
    // is not a script binding while that shadow is in view, and the
    // expression is constant on the strength of the shadow alone.
    (
        "style_shadowed_by_arrow_param",
        r#"<div :style="items.map(theme => theme.color)"></div>"#,
    ),
    (
        "style_block_scoped_shadow",
        r#"<div :style="(() => { { const theme = count; return theme } })()"></div>"#,
    ),
    (
        "style_loop_scoped_shadow",
        r#"<div :style="(() => { for (const theme of items) { return theme } return count })()"></div>"#,
    ),
    (
        "style_catch_scoped_shadow",
        r#"<div :style="(() => { try { return count } catch (theme) { return theme } })()"></div>"#,
    ),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("theme", BindingType::SetupConst),
        ("build", BindingType::SetupConst),
        ("LIMIT", BindingType::LiteralConst),
        ("imported", BindingType::ExternalModule),
        ("count", BindingType::SetupRef),
        ("msg", BindingType::SetupLet),
        ("state", BindingType::SetupReactiveConst),
        ("title", BindingType::Props),
        ("$attrs", BindingType::VueGlobal),
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
fn constant_style_bindings_match_the_shipped_dom_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// With `inline` off the prefixer turns every script binding into a
/// `$setup.` member, which the shipped rule reads as dynamic — so the
/// same battery must keep every `_normalizeStyle`.
#[test]
fn the_same_battery_normalizes_without_the_inline_option() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, false),
        &CodegenOptions::default(),
        &emit_options(&table, false),
    );
}

/// The helper's presence, pinned on both sides of the option.
#[test]
fn only_a_constant_binding_skips_the_style_helper() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let style_prop = |src: &str, inline: bool| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, inline),
        )
        .expect("constant style witness must emit")
        .assembled()
        .lines()
        .find(|line| line.contains("style:"))
        .unwrap_or_default()
        .trim()
        .to_string()
    };
    assert_eq!(
        style_prop(r#"<div :style="theme"></div>"#, true),
        "style: theme"
    );
    assert_eq!(
        style_prop(r#"<div :style="theme"></div>"#, false),
        "style: _normalizeStyle($setup.theme)"
    );
    assert_eq!(
        style_prop(r#"<div :style="count"></div>"#, true),
        "style: _normalizeStyle(count.value)"
    );
    assert_eq!(
        style_prop(r#"<div :style="[theme, count]"></div>"#, true),
        "style: _normalizeStyle([theme, count.value])"
    );
    assert_eq!(
        style_prop(r#"<li v-for="i in items" :key="i" :style="i"></li>"#, true),
        "style: _normalizeStyle(i)"
    );
}
