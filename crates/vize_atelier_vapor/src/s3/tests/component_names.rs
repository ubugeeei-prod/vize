//! Computed component prop and event names retain their own expression AST.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn computed_component_names_match_retained_generation() {
    for source in [
        r#"<Child :[propName]="value" @[eventName]="record" />"#,
        r#"<Child label="static" :[label]="value" @[eventName]="record" />"#,
        r#"<component :is="view" :[is]="value" @[eventName]="record" />"#,
        r#"<Child :[names[selected]]="value" @[events[selected]]="record(value)" />"#,
        r#"<Child :[name.toLowerCase()]="value" @['saved-'+suffix]="(v) => record(v)" />"#,
        r#"<Child v-slot="{ item }"><Other :[item.prop]="item.value" @[item.event]="save" /></Child>"#,
        r#"<Child :[propName]="value" v-model:title="form.title" @[eventName]="record" />"#,
    ] {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporS3BridgeStatus::Accepted(_)
            ),
            "{source}"
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
fn unproved_computed_name_surfaces_remain_legacy() {
    for source in [
        r#"<button :[name]="value">text</button>"#,
        r#"<slot :[name]="value" />"#,
        r#"<Child @[event].stop="save" />"#,
        r#"<Child :[name].camel="value" />"#,
        r#"<Child v-model:[name]="value" />"#,
    ] {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporS3BridgeStatus::Legacy(_)
            ),
            "{source}"
        );
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta formats generated-code snapshots"
)]
fn checked_component_name_payload_owns_generated_keys() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(
        &allocator,
        r#"<Child :[propName]="value" @[eventName]="save" />"#,
    );
    for operand in &mut s3.program.operands {
        if operand.role == vize_s3::operand::OperandRole::Name {
            match operand.value.text {
                "propName" => operand.value.text = "otherProp",
                "eventName" => operand.value.text = "otherEvent",
                _ => {}
            }
        }
    }
    let code = super::generated(
        crate::s3::admit(s3, &crate::s3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("component_name_payload", code);
}
