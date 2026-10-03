//! Real custody interruptions in the original receiver, never production flags.

extern crate std;
use super::*;
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    embed::{
        EmbedSource,
        syntax::{ProgramOptions, parse_program_once},
    },
};

std::thread_local! {
    static FAULT: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}
fn inject(point: u8) {
    FAULT.with(|fault| {
        if fault.get() == point {
            fault.set(0);
            panic!("injected interruption after original For custody");
        }
    });
}
pub(super) fn after_park() {
    inject(1);
}
pub(super) fn after_scope() {
    inject(2);
}

#[test]
fn caught_original_for_park_and_scope_unwind_preserve_whole_owner_parent_and_prefix() {
    use crate::lang::js::NativeTemplateOwner;
    for point in [1, 2] {
        let arena = vize_l0::Allocator::default();
        let source = "<script setup>const items=2;</script><template>prefix<div @click='item' v-for='item in items'>body</div></template>";
        let descriptor = Vue.observe_descriptor(
            &arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let admitted = descriptor.admitted().unwrap();
        let setup = admitted.setup().unwrap();
        let syntax = parse_program_once(
            &arena,
            EmbedSource::authored(source, setup.block().span()).unwrap(),
            ProgramOptions::module(setup.lang()),
        );
        let selected = NativeTemplateComponent::parse_in(&arena, admitted)
            .unwrap()
            .unwrap();
        let mut owner =
            NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("actual owner"));
        owner
            .setup_program(syntax.admitted_program().unwrap())
            .unwrap();
        {
            let mut walk = owner.begin().unwrap();
            let selected = walk.selected();
            let prefix_receipt = selected
                .prepare_condensed_root_text(selected.children().next().unwrap())
                .unwrap();
            walk.child(selected.children().next().unwrap()).unwrap();
            let parent = walk.root.scope;
            FAULT.with(|fault| fault.set(point));
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    walk.child(selected.children().nth(1).unwrap())
                }))
                .is_err()
            );
            assert_eq!(walk.root.scope, parent);
            assert_eq!(walk.root.facts.pending_handlers.len(), 1);
            assert_eq!(walk.root.facts.pending_for_heads.len(), 1);
            if point == 1 {
                assert_eq!(
                    walk.root.facts.pending_for_heads[0]
                        .input
                        .as_ref()
                        .unwrap()
                        .operand()
                        .raw_value(),
                    "item in items"
                );
                assert!(walk.root.facts.for_heads.iter().next().is_none());
            } else {
                let (_, record) = walk.root.facts.for_heads.iter().next().unwrap();
                assert_eq!(record.enclosing, parent);
                assert_ne!(record.scope, parent);
                assert!(record.original.is_none());
                assert_eq!(
                    record.resolution.input().operand().raw_value(),
                    "item in items"
                );
                assert_eq!(
                    walk.root.facts.scopes[record.scope.index() as usize].parent,
                    Some(parent)
                );
                assert_eq!(walk.root.facts.template_declarations.len(), 1);
            }
            assert_eq!(walk.cursor, 1);
            assert_eq!(
                walk.root_text(&prefix_receipt).unwrap_err().kind,
                NativeTemplateIssueKind::Interrupted
            );
            assert_eq!(walk.cursor, 1);
            assert_eq!(
                walk.child(selected.children().nth(1).unwrap())
                    .unwrap_err()
                    .kind,
                NativeTemplateIssueKind::Interrupted
            );
            assert_eq!(
                walk.complete().unwrap_err().kind,
                NativeTemplateIssueKind::Interrupted
            );
        }
        let output = owner.finish();
        assert!(output.view().is_err());
        if point == 1 {
            let file = output.file().unwrap();
            assert!(output.rejected_file().is_none());
            assert!(!file.is_complete());
            assert!(file.template_interruption().is_some());
            assert_eq!(file.artifact().node_count(), 1);
            assert_eq!(
                file.unattached_for_heads()
                    .next()
                    .unwrap()
                    .operand()
                    .raw_value(),
                "item in items"
            );
            assert_eq!(
                file.unattached_handlers()
                    .next()
                    .unwrap()
                    .operand()
                    .raw_value(),
                "item"
            );
            let crate::op::Op::Text(prefix) = &file.artifact().root().ops[0] else {
                panic!("prefix");
            };
            assert_eq!(prefix.content, "prefix");
        } else {
            assert!(output.file().is_none());
            let rejected = output.rejected_file().unwrap();
            assert!(rejected.template_interruption().is_some());
            assert_eq!(rejected.source(), source);
            let inputs: alloc::vec::Vec<_> = rejected.original_for_inputs().collect();
            assert_eq!(inputs.len(), 1);
            assert_eq!(inputs[0].operand().raw_value(), "item in items");
            assert_eq!(rejected.facts.pending_handlers.len(), 1);
            assert_eq!(
                rejected.facts.pending_handlers[0]
                    .input
                    .as_ref()
                    .unwrap()
                    .operand()
                    .raw_value(),
                "item"
            );
            let ops = &rejected.artifact().parts.root.ops;
            assert_eq!(ops.len(), 2);
            let crate::op::Op::Text(prefix) = &ops[0] else {
                panic!("prefix");
            };
            assert_eq!(prefix.content, "prefix");
            let crate::op::Op::OriginalFor(original) = &ops[1] else {
                panic!("partial For");
            };
            assert_eq!(original.id().node().index(), 1);
            assert_eq!(
                rejected.artifact().error,
                crate::artifact::ArtifactError::UnfinishedOwner {
                    node: original.id().node(),
                }
            );
            assert!(original.region.ops.is_empty());
            let record = rejected.facts.for_heads.get(original.id().node()).unwrap();
            assert!(record.original.is_none());
            assert!(core::ptr::eq(record.resolution.input(), inputs[0]));
            assert_eq!(rejected.facts.template_declarations.len(), 1);
            assert_eq!(
                rejected.scopes()[record.scope.index() as usize].parent,
                Some(record.enclosing)
            );
        }
    }
}
