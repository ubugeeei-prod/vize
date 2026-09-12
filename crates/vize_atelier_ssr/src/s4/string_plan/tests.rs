use super::{SsrStringPlanErrorKind, SsrStringSegmentKind, lower_s2_to_string_plan};
use vize_s0::Allocator;
use vize_s1::parse;
use vize_s2_to_s3::{PartitionKind, lower};

fn with_plan(source: &str, check: impl FnOnce(super::SsrStringPlanLowering<'_>)) {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let s3 = lower(&allocator, &s2.root);
    check(lower_s2_to_string_plan(&allocator, &s2.root, &s3.partition));
}

#[test]
fn static_and_dynamic_segments_read_partition_facts() {
    with_plan(
        r#"<main class="shell"><span>Hi</span>{{ name }}</main>"#,
        |lowered| {
            assert!(lowered.errors.is_empty(), "{:?}", lowered.errors);

            let kinds: std::vec::Vec<_> = lowered
                .plan
                .segments
                .iter()
                .map(|segment| segment.kind)
                .collect();
            assert!(kinds.contains(&SsrStringSegmentKind::OpenElement));
            assert!(kinds.contains(&SsrStringSegmentKind::StaticAttribute));
            assert!(kinds.contains(&SsrStringSegmentKind::Text));
            assert!(kinds.contains(&SsrStringSegmentKind::DynamicText));
            assert_eq!(lowered.plan.partition.dynamic_segments, 1);
            assert!(lowered.plan.partition.static_segments >= 4);
            assert!(
                lowered
                    .plan
                    .segments
                    .iter()
                    .any(|segment| segment.partition == PartitionKind::Dynamic)
            );
        },
    );
}

#[test]
fn stale_partition_facts_are_rejected() {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, "<p>{{ msg }}</p>");
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let s3 = lower(&allocator, &s2.root);
    let mut truncated = vize_s2_to_s3::PartitionFacts::new(&allocator);
    truncated.push(s3.partition.ops[0]);

    let lowered = lower_s2_to_string_plan(&allocator, &s2.root, &truncated);

    assert!(
        lowered
            .errors
            .iter()
            .any(|error| error.kind == SsrStringPlanErrorKind::MissingPartitionFact)
    );
}
