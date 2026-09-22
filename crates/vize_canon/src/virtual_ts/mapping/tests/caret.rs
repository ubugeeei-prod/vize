//! Caret lookups: byte-to-byte, first containing row, clamped to the last byte.

use super::{ProjectionFeatures, ProjectionMapping, ProjectionMeta, VizeMapping};

fn two_rows() -> ProjectionMapping {
    let mut mapping = ProjectionMapping::new();
    mapping.push(VizeMapping::new(100..110, 10..20));
    mapping.push(VizeMapping::new(200..210, 30..40));
    mapping.sort_by_authored();
    mapping
}

#[test]
fn authored_to_generated_uses_half_open_rows() {
    let mapping = two_rows();
    assert_eq!(mapping.to_generated(10), Some(100));
    assert_eq!(mapping.to_generated(15), Some(105));
    assert_eq!(mapping.to_generated(19), Some(109));
    assert_eq!(mapping.to_generated(35), Some(205));
    assert_eq!(mapping.to_generated(9), None);
    assert_eq!(mapping.to_generated(20), None);
    assert_eq!(mapping.to_generated(25), None);
}

#[test]
fn generated_to_authored_uses_half_open_rows_and_adds_the_base() {
    let mut mapping = two_rows();
    assert_eq!(mapping.to_authored(100), Some(10));
    assert_eq!(mapping.to_authored(105), Some(15));
    assert_eq!(mapping.to_authored(109), Some(19));
    assert_eq!(mapping.to_authored(205), Some(35));
    assert_eq!(mapping.to_authored(99), None);
    assert_eq!(mapping.to_authored(110), None);
    assert_eq!(mapping.to_authored(150), None);

    mapping.set_authored_base(50);
    assert_eq!(mapping.authored_base(), 50);
    assert_eq!(mapping.to_authored(105), Some(65));
    assert_eq!(
        mapping.generated_range_to_authored(100..110),
        Some(60..70),
        "the range end stays exclusive"
    );
    assert_eq!(mapping.generated_range_to_authored(105..150), None);
    // Authored lookups take row-relative offsets; the base is never removed.
    assert_eq!(mapping.to_generated(15), Some(105));
}

#[test]
fn unequal_rows_clamp_to_the_last_byte_of_the_target_range() {
    let mut mapping = ProjectionMapping::new();
    mapping.push(VizeMapping::new(0..3, 10..20));
    mapping.push(VizeMapping::new(50..60, 30..33));
    assert_eq!(mapping.to_generated(19), Some(2));
    assert_eq!(mapping.to_authored(59), Some(32));
    assert_eq!(mapping.to_generated(10), Some(0));
    assert_eq!(mapping.to_authored(50), Some(30));
}

#[test]
fn overlapping_authored_rows_resolve_to_the_first_row_in_row_order() {
    let mut mapping = ProjectionMapping::new();
    mapping.push(VizeMapping::new(0..20, 0..20));
    mapping.push(VizeMapping::new(100..105, 5..10));
    mapping.push(VizeMapping::new(300..301, 30..31));
    for authored in 5..10 {
        assert_eq!(mapping.to_generated(authored), Some(authored));
    }
    assert_eq!(mapping.to_generated(30), Some(300));
    assert_eq!(
        mapping
            .rows_containing_authored(7)
            .map(|row| row.span.gen_range.start)
            .collect::<Vec<_>>(),
        [0, 100]
    );
}

#[test]
fn feature_lookups_skip_rows_that_do_not_serve_the_feature() {
    let hover_only = ProjectionMeta {
        features: ProjectionFeatures::HOVER,
        ..ProjectionMeta::default()
    };
    let mut mapping = ProjectionMapping::new();
    mapping.push_with(VizeMapping::new(0..10, 0..10), hover_only);
    mapping.push(VizeMapping::new(100..110, 0..10));
    assert_eq!(mapping.to_generated(4), Some(4));
    assert_eq!(
        mapping.to_generated_for(4, ProjectionFeatures::HOVER),
        Some(4)
    );
    assert_eq!(
        mapping.to_generated_for(4, ProjectionFeatures::RENAME),
        Some(104)
    );
    assert_eq!(mapping.to_authored_for(4, ProjectionFeatures::RENAME), None);
    assert_eq!(
        mapping.to_authored_for(104, ProjectionFeatures::RENAME),
        Some(4)
    );
}

#[test]
fn default_metadata_feature_lookups_match_plain_lookups() {
    let mapping = two_rows();
    for authored in 0..45 {
        assert_eq!(
            mapping.to_generated_for(authored, ProjectionFeatures::DEFINITION),
            mapping.to_generated(authored)
        );
    }
    for generated in 90..215 {
        assert_eq!(
            mapping.to_authored_for(generated, ProjectionFeatures::COMPLETION),
            mapping.to_authored(generated)
        );
    }
}

#[test]
fn an_empty_mapping_resolves_nothing() {
    let mapping = ProjectionMapping::new();
    assert_eq!(mapping.to_generated(0), None);
    assert_eq!(mapping.to_authored(0), None);
    assert_eq!(mapping.span_at_generated(0), None);
    assert_eq!(mapping.diagnostic_range_to_authored(0, 1), None);
    assert_eq!(mapping.len(), 0);
}
