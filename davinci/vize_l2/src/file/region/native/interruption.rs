extern crate std;

use super::*;
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
};

std::thread_local! {
    static FAULT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn after_park() {
    FAULT.with(|fault| {
        if fault.replace(false) {
            panic!("injected interruption after original handler custody");
        }
    });
}

#[test]
fn caught_header_unwind_keeps_first_original_handler_and_cannot_revive_completion() {
    let arena = vize_l0::Allocator::default();
    let source = "<template><button @click='/*original*/ return $event'/></template>";
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut builder = FileBuilder::new(&arena, source).unwrap();
    let mut state = NativeRouteState::Scripts;
    {
        let mut walk = NativeTemplateWalk::new(&selected, &mut builder, &mut state, None).unwrap();
        FAULT.with(|fault| fault.set(true));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                walk.child(walk.selected().children().next().unwrap())
            }))
            .is_err()
        );
        assert_eq!(walk.root.facts.pending_handlers.len(), 1);
        let body = walk.root.facts.pending_handlers[0]
            .input
            .as_ref()
            .unwrap()
            .body() as *const _;
        assert_eq!(
            walk.child(walk.selected().children().next().unwrap())
                .unwrap_err()
                .kind,
            NativeTemplateIssueKind::Interrupted
        );
        assert_eq!(walk.root.facts.pending_handlers.len(), 1);
        assert_eq!(
            walk.root.facts.pending_handlers[0]
                .input
                .as_ref()
                .unwrap()
                .body() as *const _,
            body
        );
        assert_eq!(
            walk.complete().unwrap_err().kind,
            NativeTemplateIssueKind::Interrupted
        );
    }
    let file = builder.finish().unwrap();
    assert!(!file.is_complete());
    assert!(file.template_interruption().is_some());
    assert_eq!(file.artifact().node_count(), 0);
    let mut inputs = file.unattached_handlers();
    let input = inputs.next().unwrap();
    assert_eq!(input.operand().raw_value(), "/*original*/ return $event");
    assert_eq!(input.operand().syntax().comments().count(), 1);
    assert!(inputs.next().is_none());
}
