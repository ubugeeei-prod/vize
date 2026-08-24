use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{SmallVec, String, cstr};
use vize_maestro::VirtualCodeGenerator;

use super::matrix::Fixture;
use super::normalize::{sha256, stable_lines};
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
        for anchor in &fixture.anchors {
            for (offset, _) in source.match_indices(anchor.as_str()) {
                let Some(local_offset) =
                    (offset as u32).checked_sub(document.source_map.block_offset)
                else {
                    continue;
                };
                for mapping in document.source_map.find_by_source(local_offset) {
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
    let text = stable_lines(text);
    let mappings = stable_lines(mappings);
    let authored_hits = stable_lines(authored_hits);
    LaneRecord {
        status: "ok".into(),
        text_bytes,
        text_sha256: sha256(&text),
        mapping_count: mappings.lines().count(),
        mappings_sha256: sha256(&mappings),
        semantic_link_count: 0,
        semantic_links_sha256: sha256(""),
        diagnostic_count: 0,
        diagnostics_sha256: sha256(""),
        authored_hit_count: authored_hits.lines().count(),
        authored_hits_sha256: sha256(&authored_hits),
    }
}
