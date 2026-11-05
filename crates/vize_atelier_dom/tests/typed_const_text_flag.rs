//! Inline `is_ts` setup-const interpolations (#11580).
//!
//! `pick<Item>` and `pick as T` erase to `pick`, but Vue's
//! `processExpression` gives `CAN_SKIP_PATCH` only to an authored simple
//! identifier. The compat lane must keep the `TEXT` flag, matching L2.

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode};
use vize_atelier_dom::{
    DomCompilerOptions, compile_template_legacy_with_options, compile_template_with_options,
};
use vize_l0::{Allocator, FxHashMap};

/// `(name, source, keeps the TEXT flag when inline)`. A bare setup const
/// skips the flag only while the name stays the authored identifier.
const CASES: &[(&str, &str, bool)] = &[
    ("instantiation", "<div>{{ pick<Item> }}</div>", true),
    ("assertion", "<div>{{ pick as T }}</div>", true),
    ("bare", "<div>{{ pick }}</div>", false),
];

fn metadata() -> BindingMetadata {
    let mut bindings = FxHashMap::default();
    bindings.insert("pick".into(), BindingType::SetupConst);
    BindingMetadata {
        bindings,
        props_aliases: FxHashMap::default(),
        is_script_setup: true,
    }
}

fn options(inline: bool) -> DomCompilerOptions {
    DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        is_ts: true,
        inline,
        binding_metadata: Some(metadata()),
        ..DomCompilerOptions::default()
    }
}

fn render(source: &str, inline: bool, legacy: bool) -> (vize_l0::String, vize_l0::String) {
    let allocator = Allocator::new();
    let options = options(inline);
    let (errors, result) = if legacy {
        let (_, errors, result) = compile_template_legacy_with_options(&allocator, source, options);
        (errors, result)
    } else {
        let (_, errors, result) = compile_template_with_options(&allocator, source, options);
        (errors, result)
    };
    assert!(
        errors.is_empty(),
        "{source} (inline: {inline}, legacy: {legacy}): {errors:?}"
    );
    (result.preamble, result.code)
}

// The bare source is an independent legacy control, not a typed-output snapshot.
// Both full outputs were captured on the f697 production source. Type erasure
// preserves their renderer and preamble; authored type syntax additionally keeps
// TEXT in inline mode. Non-inline mode already keeps TEXT for the bare control.
// Vue 3.5.44 independently confirms that inline bare setup consts omit TEXT;
// its wrapper, parameter spelling, and non-inline bare flag differ from Vize.
const EXPECTED_PREAMBLE: &str = "import { toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\"\n";

fn expected_code(inline: bool, keeps_flag: bool) -> &'static str {
    match (inline, keeps_flag) {
        (true, false) => {
            r#"export function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("div", null, _toDisplayString(pick)))
}"#
        }
        (true, true) => {
            r#"export function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("div", null, _toDisplayString(pick), 1 /* TEXT */))
}"#
        }
        (false, _) => {
            r#"export function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("div", null, _toDisplayString($setup.pick), 1 /* TEXT */))
}"#
        }
    }
}

#[test]
fn typed_setup_const_keeps_the_text_flag_on_both_lanes() {
    for inline in [true, false] {
        for (name, source, keeps_flag_when_inline) in CASES {
            let keeps_flag = !inline || *keeps_flag_when_inline;
            let (l2_preamble, l2_code) = render(source, inline, false);
            let (legacy_preamble, legacy_code) = render(source, inline, true);
            assert_eq!(
                legacy_preamble, l2_preamble,
                "{name} preamble (inline: {inline})"
            );
            assert_eq!(legacy_code, l2_code, "{name} (inline: {inline})\n{l2_code}");
            assert_eq!(
                l2_preamble, EXPECTED_PREAMBLE,
                "{name} full preamble (inline: {inline})"
            );
            assert_eq!(
                l2_code,
                expected_code(inline, keeps_flag),
                "{name} full renderer and TEXT flag (inline: {inline})"
            );
        }
    }
}
