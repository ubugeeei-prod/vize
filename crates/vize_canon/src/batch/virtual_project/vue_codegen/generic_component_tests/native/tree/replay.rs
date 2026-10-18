//! The before control removes only duplicated synthetic inference arguments.

use super::super::packets;
use crate::batch::source_map::CompositeSourceMap;
use crate::batch::virtual_project::VirtualFile;
use crate::virtual_ts::{ProjectionFeatures, VizeMapping};
use std::ops::Range;
use vize_carton::{String, cstr};

fn materialized_range(map: &CompositeSourceMap, range: &Range<usize>) -> Option<Range<usize>> {
    let start = map
        .import_map
        .get_virtual_offset(range.start.try_into().ok()?);
    let end = map
        .import_map
        .get_virtual_offset(range.end.try_into().ok()?);
    if (end.checked_sub(start)? as usize) != range.len()
        || !(start..end).enumerate().all(|(delta, offset)| {
            map.import_map.get_original_offset(offset) as usize == range.start + delta
        })
    {
        return None;
    }
    Some(start as usize..end as usize)
}

fn same_owner(left: &VizeMapping, right: &VizeMapping) -> bool {
    left.src_range == right.src_range
        && left.sub_spans.len() == right.sub_spans.len()
        && left
            .sub_spans
            .iter()
            .zip(&right.sub_spans)
            .all(|(left_sub, right_sub)| {
                left.gen_range.start <= left_sub.gen_range.start
                    && left_sub.gen_range.start <= left_sub.gen_range.end
                    && left_sub.gen_range.end <= left.gen_range.end
                    && right.gen_range.start <= right_sub.gen_range.start
                    && right_sub.gen_range.start <= right_sub.gen_range.end
                    && right_sub.gen_range.end <= right.gen_range.end
                    && left_sub.src_range == right_sub.src_range
                    && left_sub.gen_range.start - left.gen_range.start
                        == right_sub.gen_range.start - right.gen_range.start
                    && left_sub.gen_range.end - left.gen_range.start
                        == right_sub.gen_range.end - right.gen_range.start
            })
}

/// A diagnostic-only inference copy may be omitted only while a byte-identical
/// ALL-owned copy of the same authored spans remains outside the erasure.
/// Inspect every overlapping row so a narrower diagnostic row cannot conceal
/// an authored/navigation owner. Import rewrites must keep the range contiguous.
fn inference_erasure_witnesses(
    code: &str,
    map: &CompositeSourceMap,
    erasure: Range<usize>,
) -> Option<Vec<(usize, usize)>> {
    let projection = map.sfc_map.as_ref()?.projection();
    let start = map
        .import_map
        .get_original_offset(erasure.start.try_into().ok()?) as usize;
    let end = map
        .import_map
        .get_original_offset(erasure.end.try_into().ok()?) as usize;
    if end.checked_sub(start)? != erasure.len()
        || materialized_range(map, &(start..end))? != erasure
    {
        return None;
    }
    let mut witnesses = Vec::new();
    for (index, row) in projection.rows().enumerate() {
        if row.span.gen_range.end <= start || end <= row.span.gen_range.start {
            continue;
        }
        if row.meta.features != ProjectionFeatures::DIAGNOSTICS
            || row.span.gen_range.start < start
            || end < row.span.gen_range.end
        {
            return None;
        }
        let generated = materialized_range(map, &row.span.gen_range)?;
        let counterpart = projection
            .rows()
            .enumerate()
            .find_map(|(other_index, other)| {
                if other.meta.features != ProjectionFeatures::ALL
                    || !same_owner(row.span, other.span)
                {
                    return None;
                }
                let retained = materialized_range(map, &other.span.gen_range)?;
                ((retained.end <= erasure.start || erasure.end <= retained.start)
                    && code.get(generated.clone())? == code.get(retained)?)
                .then_some(other_index)
            })?;
        witnesses.push((index, counterpart));
    }
    Some(witnesses)
}

pub(super) fn without_event_spreads(
    file: &VirtualFile,
    expected: bool,
    navigation: bool,
    name: &str,
) -> String {
    let mut bytes = file.content.as_bytes().to_vec();
    let mut offset = 0;
    let mut inference = None;
    let mut erased = Vec::new();
    let mut kinds = [false; 2];
    for line in file.content.split_inclusive('\n') {
        for (index, marker) in ["const __vize_emit_props_", "const __vize_events_resolved_"]
            .iter()
            .enumerate()
        {
            if line.contains(marker) && line.trim_end().ends_with('{') {
                assert!(inference.is_none(), "ambiguous inference argument: {line}");
                inference = Some(index);
            }
        }
        if let Some(index) = inference {
            let trimmed = line.trim();
            if trimmed.starts_with("...") && trimmed.ends_with(".bind,") {
                let start = offset + line.find("...").unwrap();
                let end = offset + line.trim_end().len();
                let witnesses = inference_erasure_witnesses(
                    file.content.as_str(),
                    &file.source_map,
                    start..end,
                )
                .expect("an inference replay must retain every primary authored owner");
                bytes[start..end].fill(b' ');
                erased.push((start, end, String::from(trimmed), witnesses));
                kinds[index] = true;
            }
            if trimmed == "});" {
                inference = None;
            }
        }
        offset += line.len();
    }
    let before = String::from(std::string::String::from_utf8(bytes).unwrap());
    packets::retain(
        &cstr!("tree-{name}-replay"),
        &serde_json::json!({
            "wholeProduction": file.content,"wholePriorArgumentOmission": before,
            "wholeUnchangedMaps":cstr!("{:#?}",file.source_map),"syntheticInferenceEqualWidthErasures":erased,
        }),
    );
    assert!(inference.is_none(), "unterminated actual inference object");
    assert_eq!(
        kinds,
        [expected, expected && navigation],
        "both existing event inference emitters must consume the same authored spread"
    );
    assert_eq!(before.len(), file.content.len());
    if !expected {
        assert_eq!(before, file.content);
    }
    before
}

#[test]
fn inference_replay_refuses_primary_owners_and_unproved_diagnostic_copies() {
    use crate::batch::source_map::SfcSourceMap;
    use crate::batch::{ImportSourceMap, OffsetAdjustment};
    use crate::virtual_ts::{ProjectionMapping, ProjectionMeta};

    let code = "xxitem.bind item.bind";
    let build = |diagnostic_features, primary_features, primary_source| {
        let mut projection = ProjectionMapping::new();
        projection.push_with(
            VizeMapping::new(0..9, 40..49),
            ProjectionMeta {
                features: diagnostic_features,
                ..Default::default()
            },
        );
        projection.push_with(
            VizeMapping::new(10..19, primary_source),
            ProjectionMeta {
                features: primary_features,
                ..Default::default()
            },
        );
        CompositeSourceMap::new_vue(
            SfcSourceMap::from_projection(projection, Vec::new()),
            ImportSourceMap::new(vec![OffsetAdjustment {
                original_offset: 0,
                adjustment: 2,
            }]),
        )
    };
    let valid = build(
        ProjectionFeatures::DIAGNOSTICS,
        ProjectionFeatures::ALL,
        40..49,
    );
    assert_eq!(
        inference_erasure_witnesses(code, &valid, 2..11),
        Some(vec![(0, 1)])
    );
    assert_eq!(inference_erasure_witnesses(code, &valid, 12..21), None);
    assert_eq!(
        inference_erasure_witnesses("xxitem.bind different", &valid, 2..11),
        None
    );
    let mut overlapping = valid.sfc_map.as_ref().unwrap().projection().clone();
    overlapping.push(VizeMapping::new(1..8, 41..48));
    let overlapping = CompositeSourceMap::new_vue(
        SfcSourceMap::from_projection(overlapping, Vec::new()),
        valid.import_map.clone(),
    );
    assert_eq!(inference_erasure_witnesses(code, &overlapping, 2..11), None);
    for invalid in [
        build(ProjectionFeatures::ALL, ProjectionFeatures::ALL, 40..49),
        build(
            ProjectionFeatures::DIAGNOSTICS,
            ProjectionFeatures::DIAGNOSTICS,
            40..49,
        ),
        build(
            ProjectionFeatures::DIAGNOSTICS,
            ProjectionFeatures::ALL,
            50..59,
        ),
        build(ProjectionFeatures::HOVER, ProjectionFeatures::ALL, 40..49),
    ] {
        assert_eq!(inference_erasure_witnesses(code, &invalid, 2..11), None);
    }
}
