use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{SmallVec, String, cstr};
use vize_maestro::VirtualCodeGenerator;

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
        for mapping in document.source_map.mappings() {
            mappings.push(cstr!(
                "{}|block={}|{}:{}>{}:{}|{:?}|{:?}",
                document.uri,
                document.source_map.block_offset,
                mapping.source.start,
                mapping.source.end,
                mapping.generated.start,
                mapping.generated.end,
                mapping.features,
                mapping.data
            ));
        }
        for (anchor_index, anchor) in fixture.anchors.iter().enumerate() {
            for (offset, _) in source.match_indices(anchor.as_str()) {
                let Ok(offset) = u32::try_from(offset) else {
                    continue;
                };
                let Ok(anchor_len) = u32::try_from(anchor.len()) else {
                    continue;
                };
                let Some(anchor_end) = offset.checked_add(anchor_len) else {
                    continue;
                };
                let Some(local_offset) = offset.checked_sub(document.source_map.block_offset)
                else {
                    continue;
                };
                for mapping in document.source_map.find_by_source(local_offset) {
                    let Some(mapping_source_start) = mapping
                        .source
                        .start
                        .checked_add(document.source_map.block_offset)
                    else {
                        continue;
                    };
                    let Some(mapping_source_end) = mapping
                        .source
                        .end
                        .checked_add(document.source_map.block_offset)
                    else {
                        continue;
                    };
                    if mapping_source_start > offset || mapping_source_end < anchor_end {
                        continue;
                    }
                    authored_anchor_hits[anchor_index] = true;
                    authored_hits.push(cstr!(
                        "{anchor}@{offset}|{}|{}:{}>{}:{}|{:?}",
                        document.uri,
                        mapping.source.start + document.source_map.block_offset,
                        mapping.source.end + document.source_map.block_offset,
                        mapping.generated.start,
                        mapping.generated.end,
                        mapping.features
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
