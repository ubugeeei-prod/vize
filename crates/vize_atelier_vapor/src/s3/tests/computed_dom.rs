//! Computed DOM keys remain typed and restore ordered static prop sources.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

const SOURCES: &[&str] = &[
    r#"<div title="fixed" :[name]="value"></div>"#,
    r#"<div :[name]="value" title="fixed"></div>"#,
    r#"<div title="fixed" :[title]="value"></div>"#,
    r#"<div class="base" :class="classes" style="color:red" :style="styles" :[keys[index]]="value"></div>"#,
    r#"<div :[name.toLowerCase()]="value" v-bind="attrs" :['data-'+suffix]="extra"></div>"#,
    r#"<input value="fixed" :[name]="value">"#,
    r#"<div style="color:red" :[style]="value" :style="styles"></div>"#,
    r#"<ul><li v-for="item in items" :key="item.id" :data-id="item.id" title="fixed" :[item.name]="item.value">{{ item.label }}</li></ul>"#,
];

#[test]
fn computed_dom_props_match_retained_ordered_sources() {
    for source in SOURCES {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        for prefix_identifiers in [false, true] {
            let compile = |davinci_retained_lane| {
                compile_vapor(
                    &allocator,
                    source,
                    VaporCompilerOptions {
                        prefix_identifiers,
                        davinci_retained_lane,
                        ..Default::default()
                    },
                )
            };
            let native = compile(false);
            let retained = compile(true);
            assert!(
                native.error_messages.is_empty(),
                "{source}: {:?}",
                native.error_messages
            );
            assert_eq!(
                native.code, retained.code,
                "{source}: prefix={prefix_identifiers}"
            );
            assert_eq!(native.templates, retained.templates, "{source}");
        }
    }
}

#[test]
fn unproved_computed_dom_combinations_remain_legacy() {
    for source in [
        r#"<div :[name].camel="value"></div>"#,
        r#"<div :[name].prop="value"></div>"#,
        r#"<div :[name]="value" @click="save"></div>"#,
        r#"<div :[name]="value" v-show="visible"></div>"#,
        r#"<input :[name]="value" v-model="model">"#,
        r#"<div :[name]="value" v-on="handlers"></div>"#,
        r#"<div v-once :[name]="value"></div>"#,
        r#"<div :[name]="value" :[name]="second"></div>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Legacy(_)),
            "{source}: {status:?}"
        );
    }
}

#[test]
fn malformed_computed_dom_sources_preserve_parser_diagnostics() {
    for source in [
        r#"<div :[name]="value"/>"#,
        r#"<div :[name]="value">"#,
        r#"<div :[name]="value"></span>"#,
    ] {
        let allocator = Allocator::new();
        let compile = |davinci_retained_lane| {
            crate::compile::compile_vapor_with_diagnostics(
                &allocator,
                source,
                VaporCompilerOptions {
                    davinci_retained_lane,
                    ..Default::default()
                },
            )
        };
        let native = compile(false);
        let retained = compile(true);
        assert!(
            !native.1.is_empty(),
            "{source}: malformed HTML must retain its diagnostics"
        );
        assert_eq!(
            native
                .1
                .iter()
                .map(|error| (&error.code, &error.message, &error.loc))
                .collect::<std::vec::Vec<_>>(),
            retained
                .1
                .iter()
                .map(|error| (&error.code, &error.message, &error.loc))
                .collect::<std::vec::Vec<_>>(),
            "{source}: parser diagnostics"
        );
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta formats generated-code snapshots"
)]
fn checked_computed_dom_payload_owns_keys_and_values() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(&allocator, r#"<div title="fixed" :[name]="value"></div>"#);
    for operand in &mut s3.program.operands {
        match (operand.role, operand.value.text) {
            (vize_s3::operand::OperandRole::Name, "name") => operand.value.text = "otherName",
            (vize_s3::operand::OperandRole::Value, "value") => operand.value.text = "otherValue",
            (vize_s3::operand::OperandRole::Attribute, "fixed") => operand.value.text = "restored",
            _ => {}
        }
    }
    let code = super::generated(
        crate::s3::admit(s3, &crate::s3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("computed_dom_payload", code);
}
