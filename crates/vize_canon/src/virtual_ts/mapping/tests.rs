use super::{
    ProjectionFeatures, ProjectionMapping, ProjectionMeta, ProjectionSpanKind, VizeMapping,
    VizeSubSpan, map_generated_offset_to_source, map_generated_range_to_source,
    mapping_for_generated_offset,
};
use vize_carton::cstr;

#[path = "tests/caret.rs"]
mod caret;

#[test]
fn prefers_exact_expression_sub_spans() {
    let mapping = VizeMapping {
        gen_range: 10..80,
        src_range: 100..140,
        sub_spans: vec![VizeSubSpan {
            gen_range: 20..43,
            src_range: 107..130,
        }],
    };
    assert_eq!(map_generated_offset_to_source(&mapping, 20), 107);
    assert_eq!(map_generated_offset_to_source(&mapping, 43), 130);
}

#[test]
fn clamps_generated_overflow_to_the_authored_range() {
    let mapping = VizeMapping::new(0..100, 10..20);
    assert_eq!(map_generated_offset_to_source(&mapping, 95), 20);
}

#[test]
fn selects_the_narrowest_mapping_for_an_offset() {
    let mappings = [
        VizeMapping::new(0..100, 0..100),
        VizeMapping::new(40..60, 200..220),
    ];
    let mapping = mapping_for_generated_offset(&mappings, 45).expect("mapping");
    assert_eq!(mapping.src_range, 200..220);
}

#[test]
fn maps_ranges_and_keeps_at_least_one_authored_byte() {
    let mappings = [VizeMapping {
        gen_range: 10..80,
        src_range: 100..140,
        sub_spans: vec![VizeSubSpan {
            gen_range: 20..43,
            src_range: 107..130,
        }],
    }];
    assert_eq!(
        map_generated_range_to_source(&mappings, 20, 43),
        Some((107, 130))
    );
    assert_eq!(
        map_generated_range_to_source(&mappings, 20, 20),
        Some((107, 108))
    );
    assert_eq!(map_generated_range_to_source(&mappings, 5, 20), None);
}

#[test]
fn diagnostic_lookups_share_the_free_arithmetic_and_add_the_base() {
    let spans = vec![
        VizeMapping::new(0..100, 0..100),
        VizeMapping {
            gen_range: 40..60,
            src_range: 200..220,
            sub_spans: vec![VizeSubSpan {
                gen_range: 45..50,
                src_range: 205..210,
            }],
        },
    ];
    let mut mapping = ProjectionMapping::from_spans(spans.clone());
    assert_eq!(
        mapping.span_at_generated(47).map(|span| &span.src_range),
        Some(&(200..220))
    );
    assert_eq!(
        mapping.diagnostic_range_to_authored(45, 50),
        map_generated_range_to_source(&spans, 45, 50)
    );
    assert_eq!(
        mapping.diagnostic_range_to_authored(45, 50),
        Some((205, 210))
    );
    mapping.set_authored_base(1000);
    assert_eq!(
        mapping.diagnostic_range_to_authored(45, 50),
        Some((1205, 1210))
    );
    assert_eq!(mapping.diagnostic_range_to_authored(150, 151), None);
}

#[test]
fn metadata_defaults_to_every_feature_and_an_unknown_kind() {
    let mut mapping = ProjectionMapping::from_spans(vec![VizeMapping::new(0..3, 0..3)]);
    assert_eq!(mapping.meta(0), ProjectionMeta::default());
    assert_eq!(mapping.meta(0).features, ProjectionFeatures::ALL);
    assert_eq!(mapping.meta(0).kind, ProjectionSpanKind::Unknown);

    let expression = ProjectionMeta::of_kind(ProjectionSpanKind::TemplateExpression);
    mapping.push_with(VizeMapping::new(5..8, 5..8), expression);
    mapping.push(VizeMapping::new(9..12, 9..12));
    let rows: Vec<_> = mapping.rows().map(|row| row.meta).collect();
    assert_eq!(
        rows,
        [
            ProjectionMeta::default(),
            expression,
            ProjectionMeta::default()
        ]
    );
}

#[test]
fn sorting_by_authored_start_is_stable_and_moves_metadata_with_rows() {
    let mut mapping = ProjectionMapping::new();
    let script = ProjectionMeta::of_kind(ProjectionSpanKind::Script);
    let expression = ProjectionMeta::of_kind(ProjectionSpanKind::TemplateExpression);
    mapping.push_with(VizeMapping::new(0..4, 30..34), expression);
    mapping.push_with(VizeMapping::new(10..14, 10..14), script);
    mapping.push(VizeMapping::new(20..24, 10..12));
    mapping.sort_by_authored();

    let rows: Vec<_> = mapping
        .rows()
        .map(|row| (row.span.gen_range.clone(), row.meta.kind))
        .collect();
    assert_eq!(
        rows,
        [
            (10..14, ProjectionSpanKind::Script),
            (20..24, ProjectionSpanKind::Unknown),
            (0..4, ProjectionSpanKind::TemplateExpression),
        ]
    );
}

#[test]
fn feature_debug_names_every_present_flag_in_declaration_order() {
    assert_eq!(cstr!("{:?}", ProjectionFeatures::NONE), "NONE");
    assert_eq!(
        cstr!(
            "{:?}",
            ProjectionFeatures::RENAME.union(ProjectionFeatures::HOVER)
        ),
        "hover | rename"
    );
    assert_eq!(
        cstr!("{:?}", ProjectionFeatures::ALL),
        "hover | completion | signature_help | definition | references | rename | diagnostics | semantic_tokens"
    );
    assert!(ProjectionFeatures::ALL.contains(ProjectionFeatures::SEMANTIC_TOKENS));
    assert!(!ProjectionFeatures::HOVER.contains(ProjectionFeatures::ALL));
    assert_eq!(ProjectionFeatures::default(), ProjectionFeatures::ALL);
}

#[test]
fn into_parts_returns_rows_and_links_in_producer_order() {
    let link = crate::virtual_ts::VizeSemanticLink {
        source_range: 1..2,
        target_range: 3..4,
        kind: crate::virtual_ts::VizeSemanticLinkKind::VuePlainScriptExport,
    };
    let spans = vec![VizeMapping::new(9..10, 0..1), VizeMapping::new(0..1, 5..6)];
    let mapping = ProjectionMapping::from_parts(spans.clone(), vec![link.clone()]);
    assert_eq!(mapping.semantic_links(), [link.clone()]);
    assert_eq!(mapping.len(), 2);
    assert!(!mapping.is_empty());
    assert_eq!(mapping.into_parts(), (spans, vec![link]));
    assert!(ProjectionMapping::new().is_empty());
}
