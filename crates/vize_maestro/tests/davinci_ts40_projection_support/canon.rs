use vize_canon::batch::{
    ContentMapperTransformOptions, ImportRewriter, VueDocumentVirtualTsOptions,
    generate_vue_content_mapper_transform_with_options,
    generate_vue_document_virtual_ts_with_options,
};
use vize_canon::virtual_ts::VirtualTsOptions;
use vize_carton::{SmallVec, String, cstr};

use super::matrix::Fixture;
use super::normalize::{fixture_path, sha256, stable_lines};
use super::record::LaneRecord;

pub(super) fn capture_canon(fixture: &Fixture, source: &str, mapper: &LaneRecord) -> LaneRecord {
    let result = generate_vue_document_virtual_ts_with_options(
        fixture_path(fixture),
        source,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        false,
        VueDocumentVirtualTsOptions {
            options_api: fixture.options_api,
            legacy_vue2: fixture.legacy_vue2,
            preserve_event_navigation: true,
            dialect: Default::default(),
        },
    );
    match result {
        Ok(document) => {
            let mappings = stable_lines(
                document
                    .mappings
                    .iter()
                    .map(|mapping| {
                        let sub_spans = stable_lines(
                            mapping
                                .sub_spans
                                .iter()
                                .map(|span| {
                                    cstr!(
                                        "{}:{}>{}:{}",
                                        span.gen_range.start,
                                        span.gen_range.end,
                                        span.src_range.start,
                                        span.src_range.end
                                    )
                                })
                                .collect(),
                        );
                        cstr!(
                            "{}:{}>{}:{}[{sub_spans}]",
                            mapping.gen_range.start,
                            mapping.gen_range.end,
                            mapping.src_range.start,
                            mapping.src_range.end
                        )
                    })
                    .collect(),
            );
            let links = stable_lines(
                document
                    .semantic_links
                    .iter()
                    .map(|link| {
                        cstr!(
                            "{:?}:{}:{}>{}:{}",
                            link.kind,
                            link.source_range.start,
                            link.source_range.end,
                            link.target_range.start,
                            link.target_range.end
                        )
                    })
                    .collect(),
            );
            LaneRecord {
                status: "ok".into(),
                text_bytes: document.pre_rewrite_code.len(),
                text_sha256: sha256(&document.pre_rewrite_code),
                mapping_count: document.mappings.len(),
                mappings_sha256: sha256(&mappings),
                semantic_link_count: document.semantic_links.len(),
                semantic_links_sha256: sha256(&links),
                diagnostic_count: mapper.diagnostic_count,
                diagnostics_sha256: mapper.diagnostics_sha256.clone(),
                authored_hit_count: 0,
                authored_hits_sha256: sha256(""),
            }
        }
        Err(error) => LaneRecord::error(error),
    }
}

pub(super) fn capture_content_mapper(fixture: &Fixture, source: &str) -> LaneRecord {
    let options = ContentMapperTransformOptions::default().with_options_api(fixture.options_api);
    match generate_vue_content_mapper_transform_with_options(fixture_path(fixture), source, options)
    {
        Ok(transform) => {
            let mappings = stable_lines(
                transform
                    .mappings
                    .iter()
                    .map(|mapping| cstr!("{:?}", mapping.0))
                    .collect(),
            );
            let links = stable_lines(
                transform
                    .semantic_links
                    .iter()
                    .map(|link| {
                        cstr!(
                            "{}:{}>{}:{}:{}",
                            link.source_start,
                            link.source_length,
                            link.target_start,
                            link.target_length,
                            link.kind
                        )
                    })
                    .collect(),
            );
            let diagnostics = stable_lines(
                transform
                    .diagnostics
                    .iter()
                    .map(|diagnostic| {
                        cstr!(
                            "{}:{}:{}:{}",
                            diagnostic.start,
                            diagnostic.length,
                            diagnostic.code,
                            diagnostic.message_text
                        )
                    })
                    .collect(),
            );
            let authored_hits = content_mapper_anchor_hits(fixture, source, &transform.mappings);
            LaneRecord {
                status: "ok".into(),
                text_bytes: transform.text.len(),
                text_sha256: sha256(&transform.text),
                mapping_count: transform.mappings.len(),
                mappings_sha256: sha256(&mappings),
                semantic_link_count: transform.semantic_links.len(),
                semantic_links_sha256: sha256(&links),
                diagnostic_count: transform.diagnostics.len(),
                diagnostics_sha256: sha256(&diagnostics),
                authored_hit_count: authored_hits.lines().count(),
                authored_hits_sha256: sha256(&authored_hits),
            }
        }
        Err(error) => LaneRecord::error(error),
    }
}

fn content_mapper_anchor_hits(
    fixture: &Fixture,
    source: &str,
    mappings: &[vize_canon::ContentMapperSpan],
) -> String {
    let mut hits = SmallVec::<[String; 8]>::new();
    for anchor in &fixture.anchors {
        for (offset, _) in source.match_indices(anchor.as_str()) {
            for mapping in mappings {
                let [
                    generated,
                    generated_len,
                    original,
                    original_len,
                    kind,
                    features,
                ] = mapping.0;
                if offset >= original && offset < original + original_len {
                    hits.push(cstr!(
                        "{anchor}@{offset}|{generated}:{generated_len}>{original}:{original_len}|{kind}:{features}"
                    ));
                }
            }
        }
    }
    stable_lines(hits)
}
