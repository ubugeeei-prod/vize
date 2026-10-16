use super::{
    ContentMapperDefinition, DiagnosticDirectivePolicy, MappedDiagnosticDirective, SpanMap,
    SpanMapFeature, SpanMapFidelity, SpanMapKind, SpanMapSegment, TextRange,
    is_supported_virtual_extension,
};

/// `<script>` content copied verbatim out of a single-file component.
fn script_block() -> SpanMap {
    SpanMap::new([SpanMapSegment::verbatim(0, 12, 8, 20)])
}

/// The duplicate-group layout documented on `original_to_virtual_spans`.
///
/// ```text
/// original:  [ A ][ B ]
/// virtual:   [ A ][ B ]      [ A ][ B ]
/// ```
fn duplicated_blocks() -> SpanMap {
    SpanMap::new([
        SpanMapSegment::verbatim(0, 2, 0, 2),
        SpanMapSegment::verbatim(2, 4, 2, 4),
        SpanMapSegment::verbatim(10, 12, 0, 2),
        SpanMapSegment::verbatim(12, 14, 2, 4),
    ])
}

#[test]
fn new_sorts_segments_by_virtual_start() {
    let map = SpanMap::new([
        SpanMapSegment::verbatim(10, 12, 0, 2),
        SpanMapSegment::verbatim(0, 2, 4, 6),
    ]);

    assert_eq!(
        map.segments()
            .iter()
            .map(|segment| segment.virtual_start)
            .collect::<Vec<_>>(),
        [0, 10]
    );
}

#[test]
fn virtual_positions_inside_a_verbatim_segment_map_exactly() {
    let map = script_block();

    let mapped = map.virtual_to_original_position(3);

    assert_eq!(mapped.position, 11);
    assert!(mapped.fidelity.is_exact());
}

#[test]
fn virtual_positions_in_synthesized_text_report_no_fidelity() {
    let map = SpanMap::new([SpanMapSegment::verbatim(10, 22, 8, 20)]);

    let before = map.virtual_to_original_position(3);
    let after = map.virtual_to_original_position(30);

    assert_eq!(before.fidelity, SpanMapFidelity::None);
    assert_eq!(before.position, 0);
    assert_eq!(after.fidelity, SpanMapFidelity::None);
    assert_eq!(
        after.position, 20,
        "gaps map to the preceding insertion point"
    );
}

#[test]
fn virtual_positions_inside_an_atom_map_to_the_segment_start() {
    let map = SpanMap::new([SpanMapSegment::new(0, 12, 8, 20, SpanMapKind::Atom)]);

    let mapped = map.virtual_to_original_position(7);

    assert_eq!(mapped.position, 8);
    assert_eq!(mapped.fidelity, SpanMapFidelity::Atom);
}

#[test]
fn virtual_spans_inside_one_verbatim_segment_stay_exact() {
    let map = script_block();

    let mapped = map.virtual_to_original_span(TextRange::new(2, 5));

    assert_eq!(mapped.range, TextRange::new(10, 13));
    assert_eq!(mapped.fidelity, SpanMapFidelity::Exact);
}

#[test]
fn virtual_spans_crossing_segments_are_approximate() {
    let map = SpanMap::new([
        SpanMapSegment::verbatim(0, 4, 0, 4),
        SpanMapSegment::verbatim(8, 12, 20, 24),
    ]);

    let mapped = map.virtual_to_original_span(TextRange::new(2, 10));

    assert_eq!(mapped.range, TextRange::new(2, 22));
    assert_eq!(mapped.fidelity, SpanMapFidelity::Approximate);
}

#[test]
fn virtual_spans_inside_an_atom_cover_the_whole_original_range() {
    let map = SpanMap::new([SpanMapSegment::new(0, 12, 8, 20, SpanMapKind::Atom)]);

    let mapped = map.virtual_to_original_span(TextRange::new(3, 5));

    assert_eq!(mapped.range, TextRange::new(8, 20));
    assert_eq!(mapped.fidelity, SpanMapFidelity::Atom);
}

#[test]
fn feature_filtered_queries_drop_segments_that_opt_out() {
    let map =
        SpanMap::new([SpanMapSegment::verbatim(0, 12, 8, 20).with_features(SpanMapFeature::HOVER)]);

    let hover = map.virtual_to_original_position_for_feature(3, SpanMapFeature::HOVER);
    let rename = map.virtual_to_original_position_for_feature(3, SpanMapFeature::RENAME);

    assert_eq!(hover.fidelity, SpanMapFidelity::Exact);
    assert_eq!(rename.fidelity, SpanMapFidelity::None);
    assert_eq!(
        rename.position, hover.position,
        "the position is still reported so callers can fall back"
    );
}

#[test]
fn feature_filtered_spans_require_every_covered_segment_to_participate() {
    let map = SpanMap::new([
        SpanMapSegment::verbatim(0, 4, 0, 4),
        SpanMapSegment::verbatim(4, 8, 4, 8).with_features(SpanMapFeature::HOVER),
    ]);

    let hover =
        map.virtual_to_original_span_for_feature(TextRange::new(2, 6), SpanMapFeature::HOVER);
    let rename =
        map.virtual_to_original_span_for_feature(TextRange::new(2, 6), SpanMapFeature::RENAME);

    assert_eq!(
        hover.fidelity,
        SpanMapFidelity::Approximate,
        "the span crosses a segment boundary, so its endpoints map separately"
    );
    assert_eq!(hover.range, TextRange::new(2, 6));
    assert_eq!(rename.fidelity, SpanMapFidelity::None);
}

#[test]
fn original_positions_project_into_every_copy() {
    let map = duplicated_blocks();

    let projections = map.original_to_virtual_positions(1, SpanMapFeature::HOVER);

    assert_eq!(
        projections
            .iter()
            .map(|projection| projection.position)
            .collect::<Vec<_>>(),
        [1, 11]
    );
    assert!(
        projections
            .iter()
            .all(|projection| projection.fidelity.is_exact())
    );
}

#[test]
fn original_positions_outside_every_segment_project_nowhere() {
    let map = script_block();

    assert!(
        map.original_to_virtual_positions(2, SpanMapFeature::HOVER)
            .is_empty()
    );
}

#[test]
fn original_spans_contained_by_a_segment_stay_exact() {
    let map = script_block();

    let spans = map.original_to_virtual_spans(TextRange::new(10, 13), SpanMapFeature::HOVER);

    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].range, TextRange::new(2, 5));
    assert_eq!(spans[0].fidelity, SpanMapFidelity::Exact);
}

#[test]
fn original_spans_crossing_duplicate_groups_pick_the_smallest_candidates() {
    let map = duplicated_blocks();

    let spans = map.original_to_virtual_spans(TextRange::new(1, 3), SpanMapFeature::HOVER);

    assert_eq!(
        spans
            .iter()
            .map(|span| (span.range.pos, span.range.end))
            .collect::<Vec<_>>(),
        [(1, 3), (11, 13)],
        "the whole [1, 13) span would cover unrelated virtual code"
    );
    assert!(
        spans
            .iter()
            .all(|span| span.fidelity == SpanMapFidelity::Approximate)
    );
}

#[test]
fn an_empty_span_map_describes_fully_synthesized_output() {
    let map = SpanMap::default();

    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(
        map.virtual_to_original_position(4),
        super::MappedPosition {
            position: 0,
            fidelity: SpanMapFidelity::None,
        }
    );
    assert!(
        map.original_to_virtual_positions(0, SpanMapFeature::ALL)
            .is_empty()
    );
}

#[test]
fn segments_round_trip_through_the_wire_shape() {
    let map = SpanMap::new([
        SpanMapSegment::verbatim(0, 4, 8, 12),
        SpanMapSegment::new(4, 6, 12, 18, SpanMapKind::Atom)
            .with_features(SpanMapFeature::HOVER | SpanMapFeature::COMPLETION),
    ]);

    let json = serde_json::to_value(&map).expect("span map serializes");
    let decoded: SpanMap = serde_json::from_value(json.clone()).expect("span map deserializes");

    assert_eq!(decoded, map);
    assert_eq!(json[0]["virtualStart"], 0);
    assert_eq!(json[1]["kind"], 1);
    assert_eq!(json[1]["features"], 5);
}

#[test]
fn omitted_features_default_to_the_full_set() {
    let segment: SpanMapSegment = serde_json::from_value(serde_json::json!({
        "virtualStart": 0,
        "virtualEnd": 4,
        "originalStart": 0,
        "originalEnd": 4,
        "kind": 0,
    }))
    .expect("segment deserializes without features");

    assert_eq!(segment.features, SpanMapFeature::ALL);
    assert!(segment.features.contains(SpanMapFeature::RENAME));
}

#[test]
fn unknown_span_map_kinds_are_rejected() {
    let error = serde_json::from_value::<SpanMapSegment>(serde_json::json!({
        "virtualStart": 0,
        "virtualEnd": 4,
        "originalStart": 0,
        "originalEnd": 4,
        "kind": 9,
    }))
    .expect_err("unknown kinds must not decode silently");

    assert!(error.to_string().contains("unknown span map kind 9"));
}

#[test]
fn diagnostic_directives_round_trip() {
    let directive = MappedDiagnosticDirective {
        original_range: TextRange::new(4, 20),
        virtual_range: TextRange::new(0, 16),
        policy: DiagnosticDirectivePolicy::Expect,
        unused_code: 2578,
    };

    let json = serde_json::to_value(directive).expect("directive serializes");

    assert_eq!(json["policy"], 1);
    assert_eq!(json["unusedCode"], 2578);
    assert_eq!(
        serde_json::from_value::<MappedDiagnosticDirective>(json).expect("directive deserializes"),
        directive
    );
}

#[test]
fn content_mapper_definitions_are_read_out_of_the_raw_config() {
    let raw = serde_json::json!({
        "compilerOptions": { "strict": true },
        "contentMappers": [
            { "package": "vue-mapper", "extensions": [".vue"], "options": { "sfc": true } },
            { "package": "svelte-mapper", "extensions": [".svelte"] },
        ],
    });

    let mappers = ContentMapperDefinition::from_raw_config(&raw);

    assert_eq!(mappers.len(), 2);
    assert_eq!(mappers[0].package, "vue-mapper");
    assert_eq!(mappers[0].options, Some(serde_json::json!({ "sfc": true })));
    assert!(mappers[0].handles_extension(".vue"));
    assert!(!mappers[0].handles_extension(".svelte"));
    assert_eq!(mappers[1].options, None);
}

#[test]
fn configs_without_content_mappers_yield_no_definitions() {
    let raw = serde_json::json!({ "compilerOptions": { "strict": true } });

    assert!(ContentMapperDefinition::from_raw_config(&raw).is_empty());
}

#[test]
fn only_upstream_virtual_extensions_are_supported() {
    assert!(is_supported_virtual_extension(".ts"));
    assert!(is_supported_virtual_extension(".json"));
    assert!(!is_supported_virtual_extension(".vue"));
    assert!(!is_supported_virtual_extension("ts"));
}

/// The `counter.demo` fixture from `sxzz/ts-content-mapper`'s example mapper.
///
/// ```text
/// <template>
///   <button>Count: {count}</button>
/// </template>
///
/// <script>
/// export const count: number = 1
/// export const label = `Count: ${count}`
/// </script>
/// ```
///
/// Its mapper lifts the `<script>` body out with a single
/// `MappedCodeBuilder(content).appendVerbatim(66, 137)`, which emits the one
/// merged segment below — real mappers produce few, large verbatim runs rather
/// than one segment per token.
fn script_block_extraction() -> SpanMap {
    SpanMap::new([SpanMapSegment::verbatim(0, 71, 66, 137)])
}

#[test]
fn a_real_mappers_script_extraction_round_trips_both_ways() {
    let map = script_block_extraction();

    // `count` sits at offset 14 of the virtual text and 80 of the `.demo` file.
    let to_original = map.virtual_to_original_span(TextRange::new(14, 19));
    assert_eq!(to_original.range, TextRange::new(80, 85));
    assert_eq!(to_original.fidelity, SpanMapFidelity::Exact);

    let to_virtual = map.original_to_virtual_spans(TextRange::new(80, 85), SpanMapFeature::HOVER);
    assert_eq!(to_virtual.len(), 1);
    assert_eq!(to_virtual[0].range, TextRange::new(14, 19));
    assert_eq!(to_virtual[0].fidelity, SpanMapFidelity::Exact);
}

#[test]
fn text_outside_the_extracted_block_has_no_virtual_counterpart() {
    let map = script_block_extraction();

    // `{count}` inside `<template>` is never handed to the checker, so a lint
    // or editor request there must not resolve to whatever the checker holds
    // at that raw offset.
    assert!(
        map.original_to_virtual_positions(29, SpanMapFeature::HOVER)
            .is_empty()
    );
    assert!(
        map.original_to_virtual_spans(TextRange::new(29, 34), SpanMapFeature::HOVER)
            .is_empty()
    );
}

#[test]
fn anchored_insertion_points_map_through_zero_length_segments() {
    // `MappedCodeBuilder::appendAnchored` emits an atom whose original span is
    // empty, defaulting to the definition and references features.
    let anchor = SpanMapFeature::DEFINITION | SpanMapFeature::REFERENCES;
    let map = SpanMap::new([
        SpanMapSegment::verbatim(0, 12, 8, 20),
        SpanMapSegment::new(12, 30, 20, 20, SpanMapKind::Atom).with_features(anchor),
    ]);

    let definitions = map.original_to_virtual_positions(20, SpanMapFeature::DEFINITION);

    assert_eq!(
        definitions,
        [
            super::MappedPosition {
                position: 12,
                fidelity: SpanMapFidelity::Exact,
            },
            super::MappedPosition {
                position: 12,
                fidelity: SpanMapFidelity::Atom,
            },
        ],
        "the verbatim segment's exclusive end and the anchor both answer here"
    );
    assert!(
        map.original_to_virtual_positions(20, SpanMapFeature::HOVER)
            .iter()
            .all(|projection| projection.fidelity == SpanMapFidelity::Exact),
        "the anchor opted out of hover, so only the verbatim segment answers"
    );
}

#[test]
fn generated_text_with_no_original_counterpart_reports_no_fidelity() {
    // A mapper's synthesized prologue — `MappedCodeBuilder::append` adds text
    // without recording a segment for it.
    let map = SpanMap::new([SpanMapSegment::verbatim(24, 95, 66, 137)]);

    let mapped = map.virtual_to_original_position(8);

    assert_eq!(mapped.fidelity, SpanMapFidelity::None);
    assert_eq!(mapped.position, 0);
}
