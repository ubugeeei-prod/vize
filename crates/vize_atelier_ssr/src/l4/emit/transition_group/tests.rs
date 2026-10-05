//! Original directive facts and a deliberately contradictory target partition.
#![expect(
    clippy::expect_used,
    reason = "exact original plan custody in negative controls"
)]

use super::admit_directives;
use crate::l4::AdmissionFailure;
use crate::l4::string_plan::{SsrSegmentSource as Source, lower_l2_to_string_plan};
use vize_l0::Allocator;
use vize_l2::op as l2;
use vize_l2_to_l3::PartitionKind;

fn error_text(error: AdmissionFailure) -> &'static str {
    match error {
        AdmissionFailure::Invalid(message) => message,
        AdmissionFailure::Unsupported(_) => "unsupported directive operand",
    }
}

#[test]
fn original_group_directives_require_their_dynamic_fact_even_without_a_value() {
    for (source, authored, has_value) in [
        (
            r#"<TransitionGroup tag="ul" v-example><li /></TransitionGroup>"#,
            "v-example",
            false,
        ),
        (
            r#"<transition-group tag="ul" v-example:[name].active="value"><li /></transition-group>"#,
            "v-example:[name].active=\"value\"",
            true,
        ),
    ] {
        let allocator = Allocator::new();
        let (tree, errors) = vize_l1::parse(&allocator, source);
        assert_eq!(errors.len(), 0);
        let lowered = vize_l1_to_l2::lower(&allocator, &tree, &errors);
        let partitioned = vize_l2_to_l3::lower(&allocator, &lowered.root);
        let planned = lower_l2_to_string_plan(&allocator, &lowered.root, &partitioned.partition);
        assert_eq!(planned.errors.len(), 0);
        let original = planned
            .plan
            .segments
            .iter()
            .find_map(|segment| match segment.source {
                Source::Component(component) => Some((segment.fact, component)),
                _ => None,
            })
            .expect("original group owner");
        let binding = original
            .1
            .bindings
            .iter()
            .find_map(|binding| match binding {
                l2::BindingOp::VueDirective(directive) => Some((binding, directive)),
                _ => None,
            })
            .expect("original directive operand");
        let segment = planned.plan.segments.iter().find(|segment| {
            matches!(segment.source, Source::Binding(owner) if core::ptr::eq(owner, binding.0))
        }).copied().expect("same original directive in the plan");
        assert_eq!(segment.partition, PartitionKind::Dynamic);
        assert_eq!(segment.span, binding.1.span);
        assert_eq!(binding.1.value.is_some(), has_value);
        assert_eq!(
            source.get(segment.span.start as usize..segment.span.end as usize),
            Some(authored)
        );
        assert_eq!(
            admit_directives(core::slice::from_ref(&segment), original.0, original.1.name)
                .map_err(error_text),
            Ok(())
        );
        let mut contradictory = segment;
        contradictory.partition = PartitionKind::Static;
        assert_eq!(
            admit_directives(
                core::slice::from_ref(&contradictory),
                original.0,
                original.1.name
            )
            .map_err(error_text),
            Err("partition fact classifies an expression segment as static")
        );
    }
}
