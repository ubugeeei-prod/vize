use super::{
    SsrStringPayloadKind, SsrStringPlanErrorKind, SsrStringSegment, SsrStringSegmentKind,
    lower_s2_to_string_plan,
};
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
            assert!(has_payload(
                &lowered.plan.segments,
                SsrStringSegmentKind::OpenElement,
                SsrStringPayloadKind::TagName,
                "main",
            ));
            assert!(has_payload(
                &lowered.plan.segments,
                SsrStringSegmentKind::StaticAttribute,
                SsrStringPayloadKind::AttributeName,
                "class",
            ));
            assert!(has_payload(
                &lowered.plan.segments,
                SsrStringSegmentKind::Text,
                SsrStringPayloadKind::Text,
                "Hi",
            ));
            assert!(has_payload(
                &lowered.plan.segments,
                SsrStringSegmentKind::DynamicText,
                SsrStringPayloadKind::Expression,
                "name",
            ));
        },
    );
}

#[test]
fn binding_segments_carry_typed_payloads() {
    with_plan(
        r#"<slot name="item" :title="label" v-html="raw" v-text="text" />"#,
        |lowered| {
            assert!(lowered.errors.is_empty(), "{:?}", lowered.errors);
            assert!(has_payload(
                &lowered.plan.segments,
                SsrStringSegmentKind::SlotOutlet,
                SsrStringPayloadKind::SlotName,
                "item",
            ));
            assert!(has_payload(
                &lowered.plan.segments,
                SsrStringSegmentKind::DynamicAttribute,
                SsrStringPayloadKind::Expression,
                "label",
            ));
            assert!(has_payload(
                &lowered.plan.segments,
                SsrStringSegmentKind::RawHtml,
                SsrStringPayloadKind::Expression,
                "raw",
            ));
            assert!(has_payload(
                &lowered.plan.segments,
                SsrStringSegmentKind::DynamicText,
                SsrStringPayloadKind::Expression,
                "text",
            ));
        },
    );
}

#[test]
fn custom_directive_values_carry_expression_payloads() {
    with_plan(r#"<div v-example="value" />"#, |lowered| {
        assert!(lowered.errors.is_empty(), "{:?}", lowered.errors);
        assert!(has_payload(
            &lowered.plan.segments,
            SsrStringSegmentKind::Directive,
            SsrStringPayloadKind::Expression,
            "value",
        ));
    });
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

fn has_payload(
    segments: &[SsrStringSegment<'_>],
    segment_kind: SsrStringSegmentKind,
    payload_kind: SsrStringPayloadKind,
    source: &str,
) -> bool {
    segments.iter().any(|segment| {
        segment.kind == segment_kind
            && segment
                .payload
                .is_some_and(|payload| payload.kind == payload_kind && payload.source == source)
    })
}
