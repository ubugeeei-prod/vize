use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_maestro::VirtualCodeGenerator;
use vize_maestro::virtual_code::{ProjectionFeatures, ProjectionRow, ProjectionSpanKind};
use vize_s0::{SmallVec, String, append, cstr};

use super::matrix::Fixture;
use super::normalize::{ordered_lines, sha256};
use super::record::LaneRecord;

pub(super) fn capture_maestro(fixture: &Fixture, source: &str) -> LaneRecord {
    let descriptor = match parse_sfc(
        source,
        SfcParseOptions {
            filename: fixture.file.as_str().into(),
            ..Default::default()
        },
    ) {
        Ok(descriptor) => descriptor,
        Err(error) => return LaneRecord::error(error.message),
    };
    let mut generator = VirtualCodeGenerator::new();
    let documents = generator.generate(&descriptor, fixture.file.as_str());
    let mut text = SmallVec::<[String; 8]>::new();
    let mut text_bytes = 0;
    let mut mappings = SmallVec::<[String; 8]>::new();
    let mut authored_hits = SmallVec::<[String; 8]>::new();
    let mut authored_anchor_hits = SmallVec::<[bool; 8]>::from_elem(false, fixture.anchors.len());
    for document in documents.all() {
        text_bytes += document.content.len();
        text.push(cstr!(
            "{}|{:?}|{}|{}",
            document.uri,
            document.language,
            document.content.len(),
            sha256(&document.content)
        ));
        let block_offset = document.source_map.authored_base();
        for row in document.source_map.rows() {
            let mapping = row.span;
            mappings.push(cstr!(
                "{}|block={block_offset}|{}:{}>{}:{}|{}|{}",
                document.uri,
                mapping.src_range.start,
                mapping.src_range.end,
                mapping.gen_range.start,
                mapping.gen_range.end,
                features_record(row.meta.features),
                data_record(row, source, block_offset)
            ));
        }
        for (anchor_index, anchor) in fixture.anchors.iter().enumerate() {
            for (offset, _) in source.match_indices(anchor.as_str()) {
                let Some(anchor_end) = offset.checked_add(anchor.len()) else {
                    continue;
                };
                let Some(local_offset) = offset.checked_sub(block_offset) else {
                    continue;
                };
                for row in document.source_map.rows_containing_authored(local_offset) {
                    let mapping = row.span;
                    let Some(mapping_source_start) =
                        mapping.src_range.start.checked_add(block_offset)
                    else {
                        continue;
                    };
                    let Some(mapping_source_end) = mapping.src_range.end.checked_add(block_offset)
                    else {
                        continue;
                    };
                    if mapping_source_start > offset || mapping_source_end < anchor_end {
                        continue;
                    }
                    authored_anchor_hits[anchor_index] = true;
                    authored_hits.push(cstr!(
                        "{anchor}@{offset}|{}|{mapping_source_start}:{mapping_source_end}>{}:{}|{}",
                        document.uri,
                        mapping.gen_range.start,
                        mapping.gen_range.end,
                        features_record(row.meta.features)
                    ));
                }
            }
        }
    }
    let text = ordered_lines(text);
    let mappings = ordered_lines(mappings);
    let authored_hits = ordered_lines(authored_hits);
    let authored_hit_anchors = fixture
        .anchors
        .iter()
        .zip(authored_anchor_hits)
        .filter(|(_, hit)| *hit)
        .map(|(anchor, _)| anchor.clone())
        .collect();
    LaneRecord {
        status: if fixture.legacy_vue2 {
            "ok:legacy-feature-projection".into()
        } else {
            "ok".into()
        },
        text_bytes,
        text_sha256: sha256(&text),
        pre_rewrite_text_bytes: 0,
        pre_rewrite_text_sha256: sha256(""),
        import_rewrite_count: 0,
        import_source_map_sha256: sha256(""),
        import_source_map_probe_count: 0,
        import_source_map_probes_sha256: sha256(""),
        mapping_count: mappings.lines().count(),
        mappings_sha256: sha256(&mappings),
        semantic_link_count: 0,
        semantic_links_sha256: sha256(""),
        diagnostic_count: 0,
        diagnostics_sha256: sha256(""),
        authored_hit_count: authored_hits.lines().count(),
        authored_hits_sha256: sha256(&authored_hits),
        authored_hit_anchors,
    }
}

/// The frozen record spelling of a row's feature flags. It predates the
/// unified model (it is the `Debug` form of Maestro's retired per-row feature
/// struct), so TS-40 digests stay byte-identical across P4-5a.
fn features_record(features: ProjectionFeatures) -> String {
    let mut record = String::from("MappingFeatures { ");
    for (index, (flag, name)) in ProjectionFeatures::NAMED.into_iter().enumerate() {
        if index > 0 {
            record.push_str(", ");
        }
        append!(record, "{name}: {}", features.contains(flag));
    }
    record.push_str(" }");
    record
}

/// The frozen record spelling of a row's construct: a template expression row
/// records its authored text, every other row records nothing.
fn data_record(row: ProjectionRow<'_>, source: &str, block_offset: usize) -> String {
    match row.meta.kind {
        ProjectionSpanKind::TemplateExpression => {
            let range = &row.span.src_range;
            let text = &source[block_offset + range.start..block_offset + range.end];
            cstr!("Some(Expression {{ text: {text:?} }})")
        }
        _ => "None".into(),
    }
}
