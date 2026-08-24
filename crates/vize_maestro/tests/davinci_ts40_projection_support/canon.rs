use vize_canon::batch::{
    ContentMapperTransformOptions, ImportRewriter, ImportSourceMap, VueDocumentVirtualTsOptions,
    generate_vue_content_mapper_transform_with_options,
    generate_vue_document_virtual_ts_with_options,
};
use vize_canon::virtual_ts::VirtualTsOptions;
use vize_carton::{SmallVec, String, append, cstr};

use super::matrix::Fixture;
use super::normalize::{fixture_path, ordered_lines, sha256};
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
            let mappings = ordered_lines(
                document
                    .mappings
                    .iter()
                    .map(|mapping| {
                        let sub_spans = ordered_lines(
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
            let links = ordered_lines(
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
            let import_map = import_source_map_facts(
                &document.import_source_map,
                document.pre_rewrite_code.len(),
                document.code.len(),
            );
            let (diagnostic_count, diagnostics_sha256) = if fixture.legacy_vue2 {
                (0, sha256(""))
            } else {
                (mapper.diagnostic_count, mapper.diagnostics_sha256.clone())
            };
            LaneRecord {
                status: if fixture.legacy_vue2 {
                    "ok:legacy-feature-projection".into()
                } else {
                    "ok".into()
                },
                text_bytes: document.code.len(),
                text_sha256: sha256(&document.code),
                pre_rewrite_text_bytes: document.pre_rewrite_code.len(),
                pre_rewrite_text_sha256: sha256(&document.pre_rewrite_code),
                import_rewrite_count: import_map.rewrite_count,
                import_source_map_sha256: import_map.map_sha256,
                import_source_map_probe_count: import_map.probe_count,
                import_source_map_probes_sha256: import_map.probes_sha256,
                mapping_count: document.mappings.len(),
                mappings_sha256: sha256(&mappings),
                semantic_link_count: document.semantic_links.len(),
                semantic_links_sha256: sha256(&links),
                diagnostic_count,
                diagnostics_sha256,
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
            let mappings = ordered_lines(
                transform
                    .mappings
                    .iter()
                    .map(|mapping| cstr!("{:?}", mapping.0))
                    .collect(),
            );
            let links = ordered_lines(
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
            let diagnostics = ordered_lines(
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
                status: if fixture.legacy_vue2 {
                    "ok:vue3-fixed-production".into()
                } else {
                    "ok".into()
                },
                text_bytes: transform.text.len(),
                text_sha256: sha256(&transform.text),
                pre_rewrite_text_bytes: 0,
                pre_rewrite_text_sha256: sha256(""),
                import_rewrite_count: 0,
                import_source_map_sha256: sha256(""),
                import_source_map_probe_count: 0,
                import_source_map_probes_sha256: sha256(""),
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
    ordered_lines(hits)
}

struct ImportSourceMapFacts {
    rewrite_count: usize,
    map_sha256: String,
    probe_count: usize,
    probes_sha256: String,
}

fn import_source_map_facts(
    map: &ImportSourceMap,
    pre_rewrite_bytes: usize,
    rewritten_bytes: usize,
) -> ImportSourceMapFacts {
    let debug = cstr!("{map:?}");
    let rewrite_count = debug.match_indices("OffsetAdjustment").count();
    let mut probes = String::with_capacity((pre_rewrite_bytes + rewritten_bytes) * 12);
    for original in 0..=pre_rewrite_bytes as u32 {
        append!(probes, "o{original}>{};", map.get_virtual_offset(original));
    }
    for virtual_offset in 0..=rewritten_bytes as u32 {
        append!(
            probes,
            "v{virtual_offset}>{};",
            map.get_original_offset(virtual_offset)
        );
    }
    ImportSourceMapFacts {
        rewrite_count,
        map_sha256: sha256(&debug),
        probe_count: pre_rewrite_bytes + rewritten_bytes + 2,
        probes_sha256: sha256(&probes),
    }
}
