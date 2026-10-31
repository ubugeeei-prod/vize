//! Interrupt the real custody transitions, retaining the same original owners.

use super::*;

#[test]
fn caught_conditional_unwind_cannot_reuse_the_cursor_or_publish_partial_if_ownership() {
    for point in 1..=5 {
        let arena = Allocator::default();
        let source = "<script setup>let ready=true</script><template><div v-if='ready'>kept</div></template>";
        let mut owner = owner(&arena, source);
        {
            let mut walk = owner.begin().unwrap();
            let selected = walk.selected();
            let parent = walk.root.scope;
            let child = selected.children().next().unwrap();
            FAULT.with(|fault| fault.set(point));
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    walk.child(child.reborrow())
                }))
                .is_err()
            );
            assert_eq!(walk.root.scope, parent);
            assert_eq!(walk.root.facts.scopes.len(), 2);
            assert!(walk.root.facts.template_declarations.is_empty());
            assert_eq!(walk.cursor, 0);
            assert!(walk.root.facts.template_walk.interruption().is_some());
            if point == 1 {
                assert!(walk.root.facts.native_conditionals().is_empty());
            } else {
                let [record] = walk.root.facts.native_conditionals() else {
                    panic!("parked original");
                };
                assert_eq!(record.state(), NativeFileConditionalState::Pending);
                assert_eq!(record.scope(), parent);
                assert!(record.original.is_none());
                assert_eq!(record.input().operand().raw_value(), "ready");
                assert!(core::ptr::eq(
                    record.input().operand().syntax().source().authored_root(),
                    source
                ));
                assert_eq!(record.input().expression().is_some(), point >= 3);
                assert_eq!(record.binding().is_some(), point >= 3);
            }
            assert_eq!(walk.child(child).unwrap_err().kind, Kind::Interrupted);
            assert_eq!(walk.complete().unwrap_err().kind, Kind::Interrupted);
        }
        let output = owner.finish();
        assert!(output.view().is_err());
        assert!(output.retained_setup().is_some());
        assert!(core::ptr::eq(
            output.selected().component().block().root_source(),
            source
        ));
        if point == 3 || point == 4 {
            assert!(output.file().is_none());
            let rejected = output.rejected_file().unwrap();
            assert!(rejected.template_interruption().is_some());
            let [record] = rejected.native_conditionals() else {
                panic!("whole pending input");
            };
            assert_eq!(record.state(), NativeFileConditionalState::Pending);
            assert!(record.original.is_none());
            let [Op::If(original)] = rejected.artifact().parts.root.ops.as_slice() else {
                panic!("retained pending If");
            };
            assert!(!record.accepts(original));
            let [branch] = original.branches.as_slice() else {
                panic!("original partial branch");
            };
            let Some(crate::expr::ExprRef::Js(expression)) = branch.condition else {
                panic!("original condition");
            };
            assert!(record.input().matches_expression(expression));
            if point == 3 {
                assert!(branch.region.ops.is_empty());
            } else {
                let [Op::Element(element)] = branch.region.ops.as_slice() else {
                    panic!("retained completed body");
                };
                let [Op::Text(text)] = element.children.ops.as_slice() else {
                    panic!("retained body text");
                };
                assert_eq!(text.content, "kept");
            }
        } else {
            assert!(output.rejected_file().is_none());
            let file = output.file().unwrap();
            assert!(!file.is_complete());
            assert!(file.template_interruption().is_some());
            if point == 5 {
                let [record] = file.native_conditionals() else {
                    panic!("closed allocation with pending receipt");
                };
                let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
                    panic!("real closed If");
                };
                assert!(!record.accepts(original));
                assert_eq!(record.state(), NativeFileConditionalState::Pending);
            } else {
                assert!(file.artifact().root().ops.is_empty());
                assert_eq!(file.artifact().node_count(), 0);
            }
        }
    }
}

#[test]
fn interrupted_preparation_failure_stays_owned_without_a_fabricated_input_or_node() {
    let arena = Allocator::default();
    let source = "<script setup>let ready=true</script><template><div v-if/></template>";
    let mut owner = owner(&arena, source);
    {
        let mut walk = owner.begin().unwrap();
        let child = walk.selected().children().next().unwrap();
        FAULT.with(|fault| fault.set(6));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| walk.child(child))).is_err()
        );
        assert!(walk.root.facts.native_conditionals().is_empty());
        let [failure] = walk.root.facts.native_conditional_failures() else {
            panic!("whole preparation refusal");
        };
        assert_eq!(
            failure.kind(),
            vize_l1::markup::NativeAttributeOperandError::IncompleteValue
        );
        assert_eq!(walk.complete().unwrap_err().kind, Kind::Interrupted);
    }
    let output = owner.finish();
    assert!(output.view().is_err());
    assert!(output.retained_setup().is_some());
    let file = output.file().unwrap();
    assert!(!file.is_complete());
    assert!(file.template_interruption().is_some());
    assert_eq!(file.artifact().node_count(), 0);
    assert!(file.native_conditionals().is_empty());
    let [failure] = file.native_conditional_failures() else {
        panic!("retained failure");
    };
    assert_eq!(
        failure.kind(),
        vize_l1::markup::NativeAttributeOperandError::IncompleteValue
    );
}
