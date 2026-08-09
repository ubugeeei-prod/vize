//! TypeScript content-mapper projection for Vue single-file components.

use std::ops::Range;
use std::path::Path;

use serde::Serialize;
use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_sfc::{SfcError, SfcParseOptions, parse_sfc};
use vize_carton::{String as CompactString, ToCompactString, config::VueVersion};

use crate::batch::Diagnostic;
use crate::batch::error::CorsaResult;
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions, VizeMapping};

use super::build::{
    descriptor_uses_jsx_script, prepend_vue_jsx_reference, virtual_ts_options_for_descriptor,
};
use super::diagnostics::invalid_sfc_fallback_virtual_ts;
use super::vue_codegen::{GeneratedVueFile, VueCodegenOptions, generate_vue_virtual_ts};

const SCRIPT_KIND_TS: u8 = 3;
const SCRIPT_KIND_TSX: u8 = 4;
const MAPPING_KIND_VERBATIM: usize = 0;
const MAPPING_KIND_ATOM: usize = 1;

// microsoft/typescript-go content-mapper protocol v1 assigns one feature bit
// to every operation from Hover (bit 0) through CodeLens (bit 20). A mapper
// that supports every operation must send SpanMapFeature.All, not the number of
// supported mapping kinds.
const PROTOCOL_V1_SPAN_MAP_FEATURE_CODE_LENS: usize = 1 << 20;
const PROTOCOL_V1_SPAN_MAP_FEATURE_ALL: usize = (PROTOCOL_V1_SPAN_MAP_FEATURE_CODE_LENS << 1) - 1;

/// A TypeScript content-mapper transform result.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapperTransform {
    pub text: CompactString,
    pub script_kind: u8,
    pub mappings: Vec<ContentMapperSpan>,
    pub diagnostics: Vec<ContentMapperDiagnostic>,
}

/// A protocol v1 span tuple:
/// `[generatedStart, generatedLength, originalStart, originalLength, kind, featureMask]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ContentMapperSpan(pub [usize; 6]);

/// A diagnostic expressed in the mapper's negotiated UTF-8 coordinates.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapperDiagnostic {
    pub message_text: CompactString,
    pub start: usize,
    pub length: usize,
}

/// Generate self-contained TypeScript and protocol-v1 mappings for one Vue SFC.
///
/// Authored parse failures are returned as mapper diagnostics with a safe
/// fallback module. They are not transport errors, so TypeScript can keep the
/// mapper process alive while the user edits an invalid document.
pub fn generate_vue_content_mapper_transform(
    path: &Path,
    content: &str,
) -> CorsaResult<ContentMapperTransform> {
    let descriptor = match parse_sfc(
        content,
        SfcParseOptions {
            filename: path.to_string_lossy().to_compact_string(),
            ..Default::default()
        },
    ) {
        Ok(descriptor) => descriptor,
        Err(error) => {
            return Ok(ContentMapperTransform {
                text: invalid_sfc_fallback_virtual_ts(),
                script_kind: SCRIPT_KIND_TS,
                mappings: Vec::new(),
                diagnostics: vec![sfc_parse_diagnostic(content, &error)],
            });
        }
    };

    let options = virtual_ts_options_for_descriptor(&VirtualTsOptions::default(), &descriptor);
    let use_tsx = descriptor_uses_jsx_script(&descriptor);
    let GeneratedVueFile {
        mut code,
        mut mappings,
        diagnostics,
    } = generate_vue_virtual_ts(
        path,
        content,
        &descriptor,
        &options,
        VueCodegenOptions {
            check_options: VirtualTsCheckOptions::default(),
            preserve_unused_diagnostics: false,
            options_api: false,
            legacy_vue2: false,
            dialect: VueVersion::default(),
            template_syntax: TemplateSyntaxMode::default(),
            experimental_in_tag_comments: false,
            hoist_shared_preamble: false,
            omit_vite_client_reference: true,
        },
    )?;

    if use_tsx {
        prepend_vue_jsx_reference(&mut code, &mut mappings);
    }

    Ok(ContentMapperTransform {
        mappings: protocol_spans(content, &code, &mappings),
        diagnostics: diagnostics
            .iter()
            .map(|diagnostic| generated_diagnostic(content, diagnostic))
            .collect(),
        text: code,
        script_kind: if use_tsx {
            SCRIPT_KIND_TSX
        } else {
            SCRIPT_KIND_TS
        },
    })
}

fn sfc_parse_diagnostic(source: &str, error: &SfcError) -> ContentMapperDiagnostic {
    let (start, length) = error
        .loc
        .as_ref()
        .map(|loc| checked_source_span(source, loc.start, loc.end))
        .unwrap_or_else(|| checked_source_span(source, 0, 0));
    ContentMapperDiagnostic {
        message_text: error.message.clone(),
        start,
        length,
    }
}

fn generated_diagnostic(source: &str, diagnostic: &Diagnostic) -> ContentMapperDiagnostic {
    let index = vize_carton::line_index::LineIndex::new(source);
    let start = index
        .line_col_to_offset(diagnostic.line, diagnostic.column)
        .unwrap_or(0);
    let (start, length) = checked_source_span(source, start, start);
    ContentMapperDiagnostic {
        message_text: diagnostic.message.clone(),
        start,
        length,
    }
}

fn checked_source_span(source: &str, start: usize, end: usize) -> (usize, usize) {
    let start = start.min(source.len());
    let start = source.floor_char_boundary(start);
    let end = end.min(source.len()).max(start);
    let end = source.ceil_char_boundary(end);
    let default_length = source[start..].chars().next().map_or(0, char::len_utf8);
    (start, end.saturating_sub(start).max(default_length))
}

#[derive(Clone, Debug)]
struct SpanCandidate {
    generated: Range<usize>,
    original: Range<usize>,
    kind: usize,
}

fn protocol_spans(
    source: &str,
    generated: &str,
    mappings: &[VizeMapping],
) -> Vec<ContentMapperSpan> {
    let mut candidates = mappings
        .iter()
        .filter_map(|mapping| {
            if mapping.sub_spans.is_empty() {
                Some(vec![candidate(
                    source,
                    generated,
                    mapping.gen_range.clone(),
                    mapping.src_range.clone(),
                )?])
            } else {
                Some(
                    mapping
                        .sub_spans
                        .iter()
                        .filter_map(|span| {
                            candidate(
                                source,
                                generated,
                                span.gen_range.clone(),
                                span.src_range.clone(),
                            )
                        })
                        .collect(),
                )
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    // Narrow authored spans win over enclosing synthetic projections.
    candidates.sort_by_key(|candidate| {
        (
            candidate.generated.len(),
            candidate.original.len(),
            candidate.generated.start,
        )
    });

    let mut accepted = Vec::<SpanCandidate>::new();
    for candidate in candidates {
        let generated_overlap = accepted
            .iter()
            .any(|span| ranges_overlap(&candidate.generated, &span.generated));
        let invalid_original_overlap = accepted.iter().any(|span| {
            candidate.original != span.original
                && ranges_overlap(&candidate.original, &span.original)
        });
        if !generated_overlap && !invalid_original_overlap {
            accepted.push(candidate);
        }
    }
    accepted.sort_by_key(|candidate| candidate.generated.start);
    accepted
        .into_iter()
        .map(|candidate| {
            ContentMapperSpan([
                candidate.generated.start,
                candidate.generated.len(),
                candidate.original.start,
                candidate.original.len(),
                candidate.kind,
                PROTOCOL_V1_SPAN_MAP_FEATURE_ALL,
            ])
        })
        .collect()
}

fn candidate(
    source: &str,
    generated: &str,
    generated_range: Range<usize>,
    original_range: Range<usize>,
) -> Option<SpanCandidate> {
    if generated_range.is_empty()
        || original_range.is_empty()
        || generated_range.end > generated.len()
        || original_range.end > source.len()
        || !generated.is_char_boundary(generated_range.start)
        || !generated.is_char_boundary(generated_range.end)
        || !source.is_char_boundary(original_range.start)
        || !source.is_char_boundary(original_range.end)
    {
        return None;
    }

    let generated_text = &generated[generated_range.clone()];
    let original_text = &source[original_range.clone()];
    if let Some(relative_start) = generated_text.find(original_text) {
        let start = generated_range.start + relative_start;
        return Some(SpanCandidate {
            generated: start..start + original_text.len(),
            original: original_range,
            kind: MAPPING_KIND_VERBATIM,
        });
    }

    Some(SpanCandidate {
        generated: generated_range,
        original: original_range,
        kind: MAPPING_KIND_ATOM,
    })
}

fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

#[cfg(test)]
#[path = "content_mapper_tests.rs"]
mod tests;
