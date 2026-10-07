//! Ordered default diagnostics and Vue's mode-sensitive processed key shape.
//! Official controls retain complete inputs and modules, not compiler snippets.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "integration tests compare complete authored artifacts"
)]

mod support;

use sha2::{Digest, Sha256};
use toml::Value;
use vize_l0::Span;
use vize_l0::diag::{Diagnostic, Stage};
use vize_l1_to_l2::{DomEmitMode, DomEmitOptions, EmitError, emit_dom_with_options, exemptions};

const CONTROLS: &[&str] = &[
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/local_member_three.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/global_identifier.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/local_identifier.toml"
    ),
    include_str!("../../../tests/_fixtures/differential/compiler/n8n-if-keys/numeric_three.toml"),
    include_str!("../../../tests/_fixtures/differential/compiler/n8n-if-keys/static_three.toml"),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/attribute_bind_kind.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/global_processed_to_raw.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/literal_arithmetic.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/object_property_only.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/member_without_references.toml"
    ),
    include_str!("../../../tests/_fixtures/differential/compiler/n8n-if-keys/local_shorthand.toml"),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/binding_identifier_only.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/expression_local_reference.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/normal_wrapper_member.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/unrelated_orphan_else.toml"
    ),
];

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().expect("capture string")
}

fn controls() -> Vec<Value> {
    CONTROLS
        .iter()
        .map(|source| toml::from_str(source).expect("official compiler capture TOML"))
        .collect()
}

#[test]
fn native_branch_keys_match_the_stock_default_and_prefix_verdicts() {
    for control in controls() {
        assert_eq!(text(&control, "version"), "3.5.26");
        assert_eq!(
            text(&control, "sha256"),
            "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec"
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(text(&control, "source").as_bytes())),
            text(&control, "source_sha256")
        );
        let modes = control["modes"].as_array().expect("two captured modes");
        assert_eq!(modes.len(), 2);
        support::with_transformed(text(&control, "template"), |lowered, _, facts, _| {
            let default_codes = modes[0]["error_codes"].as_array().expect("error vector");
            if default_codes
                .iter()
                .all(|code| code.as_integer() == Some(29))
            {
                let expected = modes[0]["error_spans"]
                    .as_array()
                    .expect("ordered key diagnostic spans")
                    .iter()
                    .map(|span| {
                        let span = span.as_array().expect("span pair");
                        Diagnostic::legacy_error(
                            &exemptions::LOWERING,
                            Stage::Semantic,
                            Span::new(
                                u32::try_from(span[0].as_integer().expect("start")).expect("u32"),
                                u32::try_from(span[1].as_integer().expect("end")).expect("u32"),
                            ),
                            "v-if/v-else-if branches must use unique keys.",
                        )
                    })
                    .collect::<Vec<_>>();
                assert_eq!(lowered.diagnostics, expected, "{}", text(&control, "name"));
            }
            for mode in modes {
                let prefixed = mode["prefixIdentifiers"].as_bool().expect("prefix mode");
                let options = DomEmitOptions {
                    mode: if prefixed {
                        DomEmitMode::Module
                    } else {
                        DomEmitMode::Function
                    },
                    prefix_identifiers: prefixed,
                    hoist_static: false,
                    ..DomEmitOptions::DEFAULT
                };
                let emitted = emit_dom_with_options(lowered, facts, &options);
                let stock_accepts = mode["error_codes"].as_array().expect("errors").is_empty();
                assert_eq!(
                    emitted.is_ok(),
                    stock_accepts,
                    "{} prefixed={prefixed}: {emitted:?}",
                    text(&control, "name")
                );
                if !stock_accepts {
                    assert!(matches!(emitted, Err(EmitError::Diagnostics)));
                }
            }
        });
    }
}

#[test]
fn conditional_slot_key_fallback_preserves_strict_native_refusal() {
    // Stock Vue does not run its VIfSameKey check on #slot templates. The native
    // strict fallback is still an open compatibility boundary, not admission.
    let cases = [
        (
            r#"<Foo><template #one v-if="a" :key="1">1</template><template #one v-else :key="1">2</template></Foo>"#,
            false,
        ),
        (
            r#"<Foo><template #one v-if="a" :key="foo">1</template><template #one v-else :key="foo">2</template></Foo>"#,
            true,
        ),
        (
            r#"<Foo v-for="chunk in chunks"><template #one v-if="a" :key="chunk.key">1</template><template #one v-else :key="chunk.key">2</template></Foo>"#,
            true,
        ),
        (
            r#"<Foo><template #one v-if="a" :key="foo">1</template><template v-else :key="foo"><template #two>2</template></template></Foo>"#,
            false,
        ),
    ];
    for (source, accepts_prefix) in cases {
        support::with_transformed(source, |lowered, _, facts, _| {
            for prefix_identifiers in [false, true] {
                let options = DomEmitOptions {
                    prefix_identifiers,
                    ..DomEmitOptions::DEFAULT
                };
                let emitted = emit_dom_with_options(lowered, facts, &options);
                assert_eq!(
                    emitted.is_ok(),
                    prefix_identifiers && accepts_prefix,
                    "{source}: {emitted:?}"
                );
            }
        });
    }
}
