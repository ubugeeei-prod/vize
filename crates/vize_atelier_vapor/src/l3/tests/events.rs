//! Computed DOM names consume checked operands without changing static events.

use super::{lowered_source, options};
use crate::l3::{VaporL3BridgeStatus, admit, lower_source_for_vapor, retained::Retained};
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
                VaporL3BridgeStatus::Accepted(_)
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
        .find(|operand| operand.role == vize_l3::operand::OperandRole::Name)
        .unwrap();
    name.value.text = "replacement";
    let code = super::generated(admit(s3, &Retained::new(&allocator)), &allocator);
    insta::assert_snapshot!("computed_event_payload", code);
}

#[test]
fn computed_component_modifiers_are_admitted() {
    for source in [
        r#"<MyComp @[name].stop="save" />"#,
        r#"<component :is="component" @[name].stop="save" />"#,
    ] {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporL3BridgeStatus::Accepted(_)
            ),
            "{source}"
        );
    }
}

#[test]
fn computed_once_element_events_are_admitted() {
    let sources = [
        r#"<button v-once @[name]="save">go</button>"#,
        r#"<main v-once><button @[name].once.capture="save">go</button></main>"#,
    ];
    for source in sources {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
    }

    let allocator = Allocator::new();
    let root_source = sources[0];
    let nested_source = sources[1];
    for prefix_identifiers in [false, true] {
        let compile = |source, davinci_retained_lane| {
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
        let native = compile(root_source, false);
        let retained = compile(root_source, true);
        assert!(native.error_messages.is_empty(), "{root_source}");
        assert_eq!(
            native.code, retained.code,
            "{root_source}: prefix={prefix_identifiers}"
        );
        assert_eq!(native.templates, retained.templates, "{root_source}");

        // Both backends emit the same nested program after their internal
        // root/child IDs are normalized to the retained numbering policy.
        let native = compile(nested_source, false);
        let retained = compile(nested_source, true);
        assert!(native.error_messages.is_empty(), "{nested_source}");
        let normalized_native = native.code.replace("n0", "root").replace("n1", "child");
        let normalized_retained = retained.code.replace("n1", "root").replace("n0", "child");
        assert_eq!(
            normalized_native, normalized_retained,
            "{nested_source}: prefix={prefix_identifiers}"
        );
        assert_eq!(native.templates, retained.templates, "{nested_source}");
    }
}
