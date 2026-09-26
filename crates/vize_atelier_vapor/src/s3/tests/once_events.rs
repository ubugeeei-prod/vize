//! Frozen DOM values do not freeze the context read by event handlers.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta formats exact code snapshots with std string names"
)]
fn frozen_subtree_events_match_retained_generation() {
    for (case, source) in [
        r#"<button v-once :title="label" @click="save">{{ label }}</button>"#,
        r#"<main v-once><div :title="label"><button @click.stop.prevent="record(label)">{{ label }}</button></div></main>"#,
        r#"<main v-once><button @focus.capture="save" @click.once="save" @keydown.enter="record(label)">{{ label }}</button></main>"#,
    ].into_iter().enumerate() {
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
            if case == 0 {
                assert_eq!(native.code, retained.code, "{source}: prefix={prefix_identifiers}");
            }
            insta::assert_snapshot!(format!("once_events_{case}_{prefix_identifiers}_code"), native.code);
            assert_eq!(native.templates, retained.templates, "{source}");
        }
    }
}

#[test]
fn unproved_once_surfaces_remain_legacy() {
    for source in [
        r#"<button v-once @[event]="save">frozen</button>"#,
        r#"<main v-once><MyComp /></main>"#,
        r#"<main v-once><span v-if="visible">content</span></main>"#,
        r#"<main v-once><span v-for="item in items">{{ item }}</span></main>"#,
        r#"<button v-once v-show="visible">content</button>"#,
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
fn frozen_event_payload_owns_the_handler() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(
        &allocator,
        r#"<button v-once @click="save">frozen</button>"#,
    );
    let handler = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| {
            operand.role == vize_s3::operand::OperandRole::Value && operand.value.text == "save"
        })
        .unwrap();
    handler.value.text = "record";
    let code = super::generated(
        crate::s3::admit(s3, &crate::s3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("once_event_payload", code);
}
