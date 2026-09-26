//! Computed DOM names consume checked operands without changing static events.

use super::{lowered_source, options};
use crate::s3::{VaporS3BridgeStatus, admit, lower_source_for_vapor, retained::Retained};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn computed_event_names_match_retained_generation() {
    for source in [
        r#"<button @[eventName]="save">go</button>"#,
        r#"<button v-on:[names[selected]].once.capture.passive="save">go</button>"#,
        r#"<button @[enabled?first:second].enter.stop="save">go</button>"#,
        r#"<button @[eventName.toLowerCase()].right="save">go</button>"#,
        r#"<button @[name].middle="save">go</button>"#,
        r#"<button @[name]="enabled ? save() : cancel()">go</button>"#,
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
            assert!(native.error_messages.is_empty(), "{source}");
            assert_eq!(
                native.code, retained.code,
                "{source}: prefix={prefix_identifiers}"
            );
        }
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta formats generated-code snapshots"
)]
fn computed_event_name_is_owned_by_the_checked_graph() {
    let allocator = Allocator::new();
    let mut s3 = lowered_source(&allocator, r#"<button @[original]="save">go</button>"#);
    let name = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| operand.role == vize_s3::operand::OperandRole::Name)
        .unwrap();
    name.value.text = "replacement";
    let code = super::generated(admit(s3, &Retained::new(&allocator)), &allocator);
    insta::assert_snapshot!("computed_event_payload", code);
}

#[test]
fn computed_component_modifiers_and_once_events_remain_explicitly_unsupported() {
    for source in [
        r#"<MyComp @[name].stop="save" />"#,
        r#"<component :is="component" @[name].stop="save" />"#,
        r#"<button v-once @[name]="save">go</button>"#,
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
