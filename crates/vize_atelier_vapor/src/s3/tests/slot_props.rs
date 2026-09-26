//! Slot selectors and computed prop names retain distinct expression identities.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn computed_slot_props_match_retained_generation_and_order() {
    for source in [
        r#"<slot :[name]="value" />"#,
        r#"<slot :name="selected" :[name]="value" />"#,
        r#"<slot title="fixed" :[name]="value" />"#,
        r#"<slot :[name]="value" title="fixed" />"#,
        r#"<slot data-id="fixed" :[names[index]]="value" :extra="extra" />"#,
        r#"<slot :['data-'+suffix]="value" :[name.toLowerCase()]="other" />"#,
        r#"<slot class="base" :class="classes" style="color:red" :style="styles" :[name]="value" />"#,
        r#"<Child v-slot="{ item }"><slot :[item.name]="item.value" /></Child>"#,
        r#"<slot :[name]="value" disabled />"#,
        r#"<slot data-id="fixed" disabled />"#,
        r#"<slot class="base" :class="classes" style="color:red" :style="styles" />"#,
        r#"<slot class :class="classes" />"#,
    ] {
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
fn malformed_computed_slot_sources_preserve_parser_diagnostics() {
    for source in [
        r#"<slot :[name]="value"></span>"#,
        r#"<slot :name="selected" :[name]="value""#,
        r#"<slot :[names[index]]="value"><b></slot>"#,
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
            !retained.1.is_empty(),
            "{source}: malformed source diagnostics"
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
fn unproved_computed_slot_prop_surfaces_remain_legacy() {
    for source in [
        r#"<slot :[name].camel="value" />"#,
        r#"<slot :[name].prop="value" />"#,
        r#"<slot :[name]="value" @saved="save" />"#,
        r#"<slot :[name]="value" @[event]="save" />"#,
        r#"<slot :[name]="value" v-bind="attrs" />"#,
        r#"<slot :[name]="value" v-model="model" />"#,
        r#"<slot :[name]="value" ref="slot" />"#,
        r#"<slot name="first" :name="second" :[name]="value" />"#,
        r#"<slot :[name]="value" :[name]="other" />"#,
        r#"<slot :[class]="value" :[class]="other" />"#,
        r#"<slot :[style]="value" :[style]="other" />"#,
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
#[expect(
    clippy::disallowed_macros,
    reason = "insta formats generated-code snapshots"
)]
fn checked_slot_prop_payload_owns_computed_keys_and_selector() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(
        &allocator,
        r#"<slot :name="selected" title="fixed" :[name]="value" />"#,
    );
    for operand in &mut s3.program.operands {
        match (operand.role, operand.value.text) {
            (vize_s3::operand::OperandRole::Name, "selected") => operand.value.text = "otherSlot",
            (vize_s3::operand::OperandRole::Name, "name") => operand.value.text = "otherName",
            (vize_s3::operand::OperandRole::Value, "value") => operand.value.text = "otherValue",
            _ => {}
        }
    }
    let code = super::generated(
        crate::s3::admit(s3, &crate::s3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("slot_prop_payload", code);
}
