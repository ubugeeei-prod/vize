//! P2-11 installment 88 witness: **`component_name`**. An SFC knows its
//! own name (its file stem), and a component tag that resolves to it is a
//! self-reference — the shipped lane asks the runtime to resolve it as
//! one, `_resolveComponent("Foo", true)`, so a recursive component finds
//! itself. Compared byte-for-byte with the shipped lane.

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
    ("exact", "<div><Foo /></div>"),
    ("kebab_of_own_name", "<div><foo-bar /></div>"),
    ("camel_of_own_name", "<div><fooBar /></div>"),
    ("other_component", "<div><Other /></div>"),
    ("mixed", "<div><Foo /><Other /></div>"),
    ("nested_self", "<Foo><Foo /></Foo>"),
    ("self_in_v_if", r#"<div><Foo v-if="ok" /></div>"#),
    (
        "self_in_v_for",
        r#"<div><Foo v-for="i in n" :key="i" /></div>"#,
    ),
    ("self_with_props", r#"<Foo :depth="n" @done="ok" />"#),
    ("dotted_not_self", "<div><Foo.Bar /></div>"),
    ("builtin_untouched", "<Transition><Foo /></Transition>"),
    ("dynamic_untouched", r#"<component :is="Foo" />"#),
];

fn shipped(name: &str, mode: CodegenMode) -> DomCompilerOptions {
    DomCompilerOptions {
        mode,
        component_name: Some(name.into()),
        ..Default::default()
    }
}

#[test]
fn component_name_in_function_mode_matches_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &shipped("FooBar", CodegenMode::Function),
        &CodegenOptions::default(),
        &DomEmitOptions {
            component_name: Some("FooBar"),
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn component_name_in_module_mode_matches_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &shipped("FooBar", CodegenMode::Module),
        &CodegenOptions::default(),
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            component_name: Some("FooBar"),
            ..DomEmitOptions::DEFAULT
        },
    );
}

/// A name that matches nothing in the battery leaves every resolution
/// alone — the flag is the *only* thing the option changes.
#[test]
fn an_unrelated_component_name_changes_nothing() {
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &shipped("Unrelated", CodegenMode::Function),
        &CodegenOptions::default(),
        &DomEmitOptions {
            component_name: Some("Unrelated"),
            ..DomEmitOptions::DEFAULT
        },
    );
}

/// The exact spelling, so a lane that dropped the flag on both sides
/// could not pass the dual run above unnoticed.
#[test]
fn the_self_reference_flag_is_pinned() {
    let allocator = vize_s0::Allocator::new();
    let emit = |src: &str, own: Option<&str>| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &DomEmitOptions {
                component_name: own,
                ..DomEmitOptions::DEFAULT
            },
        )
        .expect("component-name witness must emit")
        .assembled()
    };
    let asset = |src: &str, own: Option<&str>| {
        emit(src, own)
            .lines()
            .find(|line| line.contains("_resolveComponent("))
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    assert_eq!(
        asset("<div><Foo /></div>", Some("Foo")),
        "const _component_Foo = _resolveComponent(\"Foo\", true)"
    );
    assert_eq!(
        asset("<div><foo-bar /></div>", Some("FooBar")),
        "const _component_foo_bar = _resolveComponent(\"foo-bar\", true)"
    );
    assert_eq!(
        asset("<div><Foo /></div>", Some("Other")),
        "const _component_Foo = _resolveComponent(\"Foo\")"
    );
    assert_eq!(
        asset("<div><Foo /></div>", None),
        "const _component_Foo = _resolveComponent(\"Foo\")"
    );
}
