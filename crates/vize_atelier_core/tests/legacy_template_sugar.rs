//! Vue 2 template-sugar codegen parity + zero-cost dialect gating.
//!
//! Compiled only under `--features legacy`; the whole file is gated so the
//! default Vue 3 build never sees it. Asserts that:
//!
//! - under dialect V2 (and V2.7), `.sync` lowers to the same output as an
//!   explicit `@update:*` listener;
//! - legacy scoped-slot attributes (`slot-scope` / `scope`) lower to the same
//!   output as their `v-slot` equivalents; and
//! - under the default dialect V3, the same legacy input stays inert even in a
//!   `legacy`-enabled build.
#![cfg(feature = "legacy")]
// Integration test: plain `std::string::String` / `{out}` formatting is fine
// here (the crate's internal `vize_carton::String` rule does not apply to an
// out-of-crate test harness).
#![allow(clippy::disallowed_types, clippy::disallowed_macros)]

use vize_atelier_core::{CodegenOptions, TransformOptions, codegen, parser, transform};
use vize_carton::config::VueVersion;

fn compile(input: &str, dialect: VueVersion) -> String {
    let allocator = bumpalo::Bump::new();
    let (mut root, errors) = parser::parse(&allocator, input);
    assert!(errors.is_empty(), "parse errors: {errors:?}");

    let transform_opts = TransformOptions {
        prefix_identifiers: true,
        dialect,
        ..Default::default()
    };
    transform::transform(&allocator, &mut root, transform_opts, None);

    let codegen_opts = CodegenOptions {
        prefix_identifiers: true,
        ..Default::default()
    };
    let result = codegen::generate(&root, codegen_opts);
    let mut out = String::with_capacity(result.preamble.len() + result.code.len() + 1);
    out.push_str(result.preamble.as_str());
    out.push('\n');
    out.push_str(result.code.as_str());
    out
}

// --- `.sync` (`:foo.sync` -> `:foo` + `@update:foo`) ------------------------

#[test]
fn v2_sync_matches_explicit_update_listener() {
    let sync = compile(r#"<MyComp :foo.sync="bar" />"#, VueVersion::V2);
    let explicit = compile(
        r#"<MyComp :foo="bar" @update:foo="$event => ((bar) = $event)" />"#,
        VueVersion::V2,
    );

    assert!(sync.contains(r#""onUpdate:foo""#), "{sync}");
    assert_eq!(sync, explicit);
}

#[test]
fn v2_sync_preserves_other_bind_modifiers() {
    let sync = compile(r#"<MyComp :foo.sync.camel="bar" />"#, VueVersion::V2);
    let explicit = compile(
        r#"<MyComp :foo.camel="bar" @update:foo="$event => ((bar) = $event)" />"#,
        VueVersion::V2,
    );

    assert!(sync.contains(r#""onUpdate:foo""#), "{sync}");
    assert_eq!(sync, explicit);
}

#[test]
fn v2_7_shares_the_v2_sync_dialect() {
    let out = compile(r#"<MyComp :foo.sync="bar" />"#, VueVersion::V2_7);
    assert!(out.contains(r#""onUpdate:foo""#), "{out}");
}

#[test]
fn v3_default_dialect_does_not_synthesize_sync_listener() {
    let out = compile(r#"<MyComp :foo.sync="bar" />"#, VueVersion::V3);
    assert!(!out.contains("onUpdate:foo"), "{out}");
}

// --- `slot-scope` / `scope` (`template` attrs -> `v-slot`) ------------------

#[test]
fn v2_named_slot_scope_matches_v_slot() {
    let legacy = compile(
        r#"<MyComp><template slot="header" slot-scope="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V2,
    );
    let modern = compile(
        r#"<MyComp><template #header="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V2,
    );

    assert!(legacy.contains("header:"), "{legacy}");
    assert_eq!(legacy, modern);
}

#[test]
fn v2_default_slot_scope_matches_v_slot_default() {
    let legacy = compile(
        r#"<MyComp><template slot-scope="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V2,
    );
    let modern = compile(
        r#"<MyComp><template #default="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V2,
    );

    assert!(legacy.contains("default:"), "{legacy}");
    assert_eq!(legacy, modern);
}

#[test]
fn v2_scope_alias_matches_v_slot_default() {
    let legacy = compile(
        r#"<MyComp><template scope="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V2,
    );
    let modern = compile(
        r#"<MyComp><template #default="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V2,
    );

    assert!(legacy.contains("default:"), "{legacy}");
    assert_eq!(legacy, modern);
}

#[test]
fn v2_7_shares_the_v2_scoped_slot_dialect() {
    let legacy = compile(
        r#"<MyComp><template slot="header" slot-scope="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V2_7,
    );
    let modern = compile(
        r#"<MyComp><template #header="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V2_7,
    );

    assert_eq!(legacy, modern);
}

#[test]
fn v3_default_dialect_keeps_scoped_slot_attrs_inert() {
    let legacy = compile(
        r#"<MyComp><template slot="header" slot-scope="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V3,
    );
    let modern = compile(
        r#"<MyComp><template #header="props">{{ props.title }}</template></MyComp>"#,
        VueVersion::V3,
    );

    assert_ne!(legacy, modern);
}

#[test]
fn v2_and_v3_outputs_match_without_template_sugar() {
    let src = r#"<MyComp :foo="bar" @update:foo="onUpdate"><template #header="props">{{ props.title }}</template></MyComp>"#;
    let v2 = compile(src, VueVersion::V2);
    let v3 = compile(src, VueVersion::V3);
    assert_eq!(v2, v3);
}
