//! Vue 2 v-on event-modifier sugar codegen parity + zero-cost dialect gating.
//!
//! Compiled only under `--features legacy`; the whole file is gated so the
//! default Vue 3 build never sees it. Asserts that:
//!
//! - under dialect V2 (and V2.7), `@keyup.13="x"` lowers to the same key-name
//!   guard Vue 3 emits for `@keyup.enter` (`_withKeys(..., ["enter"])`), and
//!   `@click.native="x"` on a component drops the removed `.native` modifier;
//! - under the default dialect V3, the *same* numeric keycode stays a raw
//!   `["13"]` guard — proving the feature is inert for Vue 3 even in a
//!   `legacy`-enabled build.
#![cfg(feature = "legacy")]
// Integration test: plain `std::string::String` / `{out}` formatting is fine
// here (the crate's internal `vize_carton::String` rule does not apply to an
// out-of-crate test harness).
#![allow(clippy::disallowed_types, clippy::disallowed_macros)]

use vize_atelier_core::{CodegenOptions, TransformOptions, codegen, lane, parser};
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
    lane::transform(&allocator, &mut root, transform_opts, None);

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

// --- Numeric keycode modifiers (`@keyup.13` -> `@keyup.enter`) ---------------

#[test]
fn v2_numeric_keycode_maps_to_key_name() {
    // `@keyup.13` must lower exactly like `@keyup.enter` does under any dialect.
    let v2_13 = compile(r#"<input @keyup.13="onEnter" />"#, VueVersion::V2);
    let v2_enter = compile(r#"<input @keyup.enter="onEnter" />"#, VueVersion::V2);
    assert!(v2_13.contains(r#"_withKeys"#), "{v2_13}");
    assert!(v2_13.contains(r#"["enter"]"#), "{v2_13}");
    assert!(!v2_13.contains(r#"["13"]"#), "{v2_13}");
    assert_eq!(v2_13, v2_enter);
}

#[test]
fn v2_maps_all_builtin_numeric_keycodes() {
    for (code, name) in [
        ("9", "tab"),
        ("13", "enter"),
        ("27", "esc"),
        ("32", "space"),
        ("37", "left"),
        ("38", "up"),
        ("39", "right"),
        ("40", "down"),
        ("8", "delete"),
        ("46", "delete"),
    ] {
        let src = format!(r#"<input @keyup.{code}="onKey" />"#);
        let out = compile(&src, VueVersion::V2);
        let expected = format!(r#"["{name}"]"#);
        assert!(out.contains(&expected), "keycode {code}: {out}");
    }
}

#[test]
fn v2_unmapped_numeric_keycode_is_left_as_is() {
    // `65` (the `a` key) has no Vue 2 built-in alias, so it stays verbatim.
    let out = compile(r#"<input @keyup.65="onA" />"#, VueVersion::V2);
    assert!(out.contains(r#"["65"]"#), "{out}");
}

#[test]
fn v2_7_shares_the_v2_event_dialect() {
    let out = compile(r#"<input @keyup.13="onEnter" />"#, VueVersion::V2_7);
    assert!(out.contains(r#"["enter"]"#), "{out}");
}

// --- `.native` modifier ------------------------------------------------------

#[test]
fn v2_native_on_component_is_stripped_to_plain_listener() {
    // `@click.native` on a component drops `.native`; the result is a plain
    // `onClick` handler with no leftover modifier guard.
    let out = compile(r#"<MyComp @click.native="onClick" />"#, VueVersion::V2);
    assert!(out.contains("onClick"), "{out}");
    assert!(!out.contains("native"), "{out}");
    // `.native` is not a key/system modifier, so no guard helper is emitted.
    assert!(!out.contains("_withModifiers"), "{out}");
    assert!(!out.contains("_withKeys"), "{out}");
    // The plain `@click` equivalent compiles to the same handler prop.
    let plain = compile(r#"<MyComp @click="onClick" />"#, VueVersion::V2);
    assert_eq!(out, plain);
}

#[test]
fn v2_native_with_other_modifiers_keeps_the_rest() {
    // `.native` is removed but a co-located `.stop` survives as a guard.
    let out = compile(r#"<MyComp @click.native.stop="onClick" />"#, VueVersion::V2);
    assert!(out.contains("_withModifiers"), "{out}");
    assert!(out.contains(r#"["stop"]"#), "{out}");
    assert!(!out.contains("native"), "{out}");
}

// --- Zero-cost: dialect V3 leaves the modifiers untouched --------------------

#[test]
fn v3_default_dialect_keeps_numeric_keycode_raw() {
    // The same input the V2 test maps to `enter` must stay a raw `["13"]`
    // guard under the default Vue 3 dialect — even in this `legacy` build.
    let out = compile(r#"<input @keyup.13="onEnter" />"#, VueVersion::V3);
    assert!(out.contains(r#"["13"]"#), "{out}");
    assert!(!out.contains(r#"["enter"]"#), "{out}");
}

#[test]
fn v2_and_v3_outputs_diverge_only_for_numeric_keycodes() {
    // A keycode-free, `.native`-free template must produce identical output
    // under V2 and V3, demonstrating the dialect only changes the two sugars.
    let v2 = compile(r#"<input @keyup.enter="onEnter" />"#, VueVersion::V2);
    let v3 = compile(r#"<input @keyup.enter="onEnter" />"#, VueVersion::V3);
    assert_eq!(v2, v3);
}

#[test]
fn v3_native_on_component_keeps_the_modifier_in_the_ast() {
    // Under V3 the `.native` modifier is retained on the directive. The
    // generated handler value is the same (`.native` is a codegen no-op), but
    // the retained modifier flips the props object to the multi-line "has
    // modifiers" form, so the V2-stripped output (compact, single-line —
    // byte-identical to plain `@click`) genuinely diverges from V3 here. This
    // pins the observable effect of the V2 strip.
    let v3 = compile(r#"<MyComp @click.native="onClick" />"#, VueVersion::V3);
    let v3_plain = compile(r#"<MyComp @click="onClick" />"#, VueVersion::V3);
    let v2 = compile(r#"<MyComp @click.native="onClick" />"#, VueVersion::V2);
    // V3 keeps `.native` and so does NOT collapse to the plain-`@click` form;
    // V2 strips it and does.
    assert_ne!(v3, v3_plain);
    assert_eq!(v2, v3_plain);
}
