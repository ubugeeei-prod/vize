use super::*;
use vize_croquis_cf::{ComplexityDimensionScores, FileId};

#[test]
fn complexity_markdown_explains_dimensions_and_hotspots() {
    let input = ComplexityInput {
        component_count: 1,
        template_if_count: 2,
        prop_drilling_edge_count: 3,
        provide_inject_fanout_count: 3,
        reactive_cycle_count: 1,
        ..ComplexityInput::default()
    };
    let report = ComplexityReport::from_input(input);
    let hotspot = ComplexityHotspot {
        file_id: FileId::new(2),
        file_name: "Feature|Panel.vue".into(),
        component_name: Some("FeaturePanel".into()),
        input,
        dimensions: report.dimensions,
        total_score: report.total_score,
        dominant_dimension: report.dominant_dimension(),
    };

    let markdown = render_complexity_markdown(&report, &[hotspot]);

    assert!(markdown.contains("## Cross-file Complexity"));
    assert!(markdown.contains("| Total score | 26 |"));
    assert!(markdown.contains("| reactive-graph | 10 |"));
    assert!(markdown.contains("Feature\\|Panel.vue"));
    assert!(markdown.contains(
        "reactive-graph (10) via v-if=2, prop edges=3, provide fanout=3, reactive cycles=1"
    ));
}

#[test]
fn complexity_markdown_handles_empty_hotspots() {
    let report = ComplexityReport::from_input(ComplexityInput::default());

    let markdown = render_complexity_markdown(&report, &[]);

    assert!(markdown.contains("| Band | low |"));
    assert!(markdown.contains("No component hotspots were identified."));
}

#[test]
fn complexity_markdown_uses_report_dimension_scores() {
    let report = ComplexityReport {
        input: ComplexityInput::default(),
        dimensions: ComplexityDimensionScores {
            slot_usage: 7,
            provide_inject: 5,
            ..ComplexityDimensionScores::default()
        },
        cyclomatic_score: 0,
        cognitive_score: 0,
        total_score: 12,
        band: ComplexityBand::Low,
    };

    let markdown = render_complexity_markdown(&report, &[]);

    assert!(markdown.contains("| slot-usage | 7 |"));
    assert!(markdown.contains("| provide-inject | 5 |"));
}
