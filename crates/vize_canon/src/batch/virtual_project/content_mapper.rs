//! TypeScript content-mapper projection for Vue single-file components.

use std::ops::Range;
use std::path::Path;

use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_sfc::{SfcError, SfcParseOptions, parse_sfc};
use vize_l0::{String as CompactString, ToCompactString, config::VueVersion};

use crate::batch::Diagnostic;
use crate::batch::error::CorsaResult;
use crate::virtual_ts::{ProjectionFeatures, ProjectionMeta, VirtualTsOptions, VizeMapping};

mod alias;
use alias::{is_alias_projection, is_synthetic_content_mapper_identifier};

mod protocol;
use protocol::protocol_semantic_links;
pub use protocol::{
    CONTENT_MAPPER_GENERATED_DIAGNOSTIC_CODE, CONTENT_MAPPER_SFC_PARSE_ERROR_CODE,
    CONTENT_MAPPER_VIRTUAL_EXTENSION, ContentMapperDiagnostic, ContentMapperDiagnosticDirective,
    ContentMapperDiagnosticDirectives, ContentMapperSemanticLink, ContentMapperSpan,
    ContentMapperTransform, ContentMapperUnusedExpectDiagnostic,
};

mod directives;
use directives::{
    sfc_diagnostic_directives, with_component_prop_echoes, with_generic_echoes,
    with_owned_input_echoes,
};

mod component_name;
use component_name::content_mapper_component_name;

use super::art_usage::apply_musea_art_globals;
use super::build::{descriptor_uses_jsx_script, prepend_vue_jsx_reference};
use super::diagnostics::invalid_sfc_fallback_virtual_ts;
use super::javascript_sfc::{descriptor_is_unchecked_javascript, prepare_script_blocks};
use super::vue_codegen::{GeneratedVueFile, VueCodegenOptions, generate_vue_virtual_ts};

mod masked_spans;
mod quoted_spans;
mod span_features;
mod span_normalize;

use span_features::content_mapper_span_features;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
enum ContentMapperSpanKind {
    Verbatim = 0,
    Atom = 1,
    Alias = 2,
}

mod options;
pub use options::ContentMapperTransformOptions;

/// Generate self-contained TypeScript and protocol-v1 mappings for one Vue SFC.
///
/// Authored parse failures are returned as mapper diagnostics with a safe
/// fallback module. They are not transport errors, so TypeScript can keep the
/// mapper process alive while the user edits an invalid document.
pub fn generate_vue_content_mapper_transform(
    path: &Path,
    content: &str,
) -> CorsaResult<ContentMapperTransform> {
    generate_vue_content_mapper_transform_with_options(
        path,
        content,
        ContentMapperTransformOptions::default(),
    )
}

/// Generate a content-mapper transform with Vize-specific mapper settings.
pub fn generate_vue_content_mapper_transform_with_options(
    path: &Path,
    content: &str,
    transform_options: ContentMapperTransformOptions,
) -> CorsaResult<ContentMapperTransform> {
    generate_vue_content_mapper_transform_with_project_options(
        path,
        content,
        transform_options,
        None,
    )
}

/// Generate a transform using configuration admitted when the host opened the project.
/// The borrowed router root performs no configuration reads during transformation.
pub fn generate_vue_content_mapper_transform_with_project_options(
    path: &Path,
    content: &str,
    transform_options: ContentMapperTransformOptions,
    typed_router_root: Option<&Path>,
) -> CorsaResult<ContentMapperTransform> {
    vize_l0::profile!(
        "canon.content_mapper.transform",
        transform_with_diagnostic_ownership(
            path,
            content,
            transform_options,
            typed_router_root,
            Some(Default::default()),
        )
    )
}

fn transform_with_diagnostic_ownership(
    path: &Path,
    content: &str,
    transform_options: ContentMapperTransformOptions,
    typed_router_root: Option<&Path>,
    diagnostic_ownership: Option<
        crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector,
    >,
) -> CorsaResult<ContentMapperTransform> {
    let authored_content = content;
    let authored = crate::authored_sfc::AuthoredSfc::new(content);
    let content = authored.analyzed();
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
                extension: CONTENT_MAPPER_VIRTUAL_EXTENSION,
                mappings: Vec::new(),
                semantic_links: Vec::new(),
                diagnostic_directives: None,
                diagnostics: vec![sfc_parse_diagnostic(content, &error)],
            });
        }
    };

    let descriptor = prepare_script_blocks(descriptor);
    let suppress_typescript_diagnostics =
        !transform_options.check_javascript && descriptor_is_unchecked_javascript(&descriptor);
    let use_tsx = descriptor_uses_jsx_script(&descriptor);
    let component_name = content_mapper_component_name(path);
    let mut virtual_ts_options = VirtualTsOptions::default();
    apply_musea_art_globals(path, &mut virtual_ts_options);
    let GeneratedVueFile {
        mut code,
        mut mappings,
        mapping_meta,
        mut generic_copies,
        mut semantic_links,
        mut prop_default_key_links,
        mut diagnostics,
        ..
    } = generate_vue_virtual_ts(
        path,
        content,
        &descriptor,
        &virtual_ts_options,
        VueCodegenOptions {
            diagnostic_ownership: diagnostic_ownership.as_ref(),
            check_options: transform_options.virtual_ts_check_options(),
            typed_router: typed_router_root.map(super::typed_router::Config::files_only),
            preserve_unused_diagnostics: transform_options.preserve_unused_diagnostics,
            options_api: transform_options.options_api(),
            preserve_authored_component: true,
            preserve_script_on_template_error: true,
            component_name: Some(component_name.as_str()),
            external_component_attrs_module: transform_options
                .external_component_attrs_module
                .then_some(if transform_options.external_component_type_helpers {
                    crate::virtual_ts::ComponentHelperModule::Types
                } else {
                    crate::virtual_ts::ComponentHelperModule::Attributes
                }),
            preserve_event_navigation: true,
            legacy_vue2: false,
            dialect: VueVersion::default(),
            template_syntax: TemplateSyntaxMode::default(),
            experimental_in_tag_comments: false,
            experimental_patterned_template: false,
            experimental_strict_slot_children: false,
            hoist_shared_preamble: false,
            inline_auto_import_stubs: true,
            omit_vite_client_reference: true,
            runtime_prop_resolve_cache: None,
        },
    )?;

    publicize_generated_diagnostic_helpers(&mut code, &mappings);

    let unprefixed_len = code.len();
    if use_tsx {
        let previous_length = code.len();
        prepend_vue_jsx_reference(
            &mut code,
            &mut mappings,
            &mut semantic_links,
            &mut prop_default_key_links,
        );
        if let Some(collector) = &diagnostic_ownership {
            collector
                .borrow_mut()
                .note_generated_insertion(0, code.len() - previous_length);
        }
    }
    if suppress_typescript_diagnostics {
        let previous_length = code.len();
        prepend_typescript_nocheck(
            &mut code,
            &mut mappings,
            &mut semantic_links,
            &mut prop_default_key_links,
        );
        if let Some(collector) = &diagnostic_ownership {
            collector
                .borrow_mut()
                .note_generated_insertion(0, code.len() - previous_length);
        }
    }
    if let Some(copies) = &mut generic_copies {
        copies.shift_generated(code.len() - unprefixed_len);
    }

    authored.remap_mappings(&mut mappings);
    for diagnostic in &mut diagnostics {
        (diagnostic.line, diagnostic.column) =
            authored.to_authored_line_column(diagnostic.line, diagnostic.column);
    }
    let content = authored_content;
    let spans = protocol_spans(content, &code, &mappings, &mapping_meta);
    let (mut diagnostic_echoes, callback_primaries) = diagnostic_ownership
        .map(|collector| {
            let mut ownership = collector.into_inner();
            let primaries = ownership.take_callback_primaries();
            (ownership.into_echoes(), primaries)
        })
        .unwrap_or_default();
    for echo in &mut diagnostic_echoes {
        echo.authored = authored.to_authored(echo.authored.clone());
    }
    Ok(ContentMapperTransform {
        extension: CONTENT_MAPPER_VIRTUAL_EXTENSION,
        // Directive comments are read from HTML templates and SFC blocks only.
        diagnostic_directives: with_owned_input_echoes(
            (!authored.is_derived())
                .then(|| {
                    let directives = with_generic_echoes(
                        sfc_diagnostic_directives(content, &descriptor, &spans, &diagnostic_echoes),
                        generic_copies.as_ref(),
                    );
                    with_component_prop_echoes(directives, &code, &mappings, &callback_primaries)
                })
                .flatten(),
            &diagnostic_echoes,
        ),
        mappings: spans,
        semantic_links: protocol_semantic_links(&semantic_links),
        diagnostics: diagnostics
            .iter()
            .map(|diagnostic| generated_diagnostic(content, diagnostic))
            .chain(authored.preprocessing_errors().map(|error| {
                let (start, length) =
                    checked_source_span(content, error.start as usize, error.end as usize);
                ContentMapperDiagnostic {
                    message_text: error.message.into(),
                    start,
                    length,
                    code: CONTENT_MAPPER_SFC_PARSE_ERROR_CODE,
                }
            }))
            .collect(),
        text: code,
    })
}

/// Give generated component-input helpers public display names before the
/// upstream host parses the virtual module. The replacements are byte-identical
/// so every projection offset stays stable.
///
/// Script `/// <reference types>` values are copied into the generated prefix
/// without mappings. Treat those complete lines as authored ranges as well, so
/// a package that happens to use an internal helper name is never rewritten.
fn publicize_generated_diagnostic_helpers(code: &mut CompactString, mappings: &[VizeMapping]) {
    let mut ranges = mappings
        .iter()
        .map(|mapping| mapping.gen_range.clone())
        .collect::<Vec<_>>();
    ranges.extend(reference_type_directive_ranges(code));
    ranges.sort_unstable_by_key(|range| range.start);
    let mut authored_ranges: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        if let Some(last) = authored_ranges.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
        } else {
            authored_ranges.push(range);
        }
    }

    for (internal, display) in [
        ("__VizeComponentCheckProps", "ComponentPropsForTemplate"),
        ("__VizePublicComponentAttrs", "VueTemplatePublicAttrsType"),
    ] {
        debug_assert_eq!(internal.len(), display.len());
        let occurrences = code
            .match_indices(internal)
            .map(|(start, _)| start)
            .collect::<Vec<_>>();
        for start in occurrences {
            let end = start + internal.len();
            let candidate = start..end;
            let next = authored_ranges.partition_point(|range| range.end <= candidate.start);
            let authored = authored_ranges
                .get(next)
                .is_some_and(|range| candidate.start < range.end && range.start < candidate.end);
            if !authored {
                code.replace_range(candidate, display);
            }
        }
    }
}

fn reference_type_directive_ranges(code: &str) -> Vec<Range<usize>> {
    let mut start = 0;
    code.split_inclusive('\n')
        .filter_map(|line| {
            let range = start..start + line.len();
            start = range.end;
            line.trim_start()
                .starts_with("/// <reference types=")
                .then_some(range)
        })
        .collect()
}

const TYPESCRIPT_NOCHECK_DIRECTIVE: &str = "// @ts-nocheck\n";

fn prepend_typescript_nocheck(
    code: &mut vize_l0::String,
    mappings: &mut [VizeMapping],
    semantic_links: &mut [crate::virtual_ts::VizeSemanticLink],
    prop_default_key_links: &mut [crate::virtual_ts::VizeSemanticLink],
) {
    let offset = TYPESCRIPT_NOCHECK_DIRECTIVE.len();
    let mut prefixed = vize_l0::String::with_capacity(offset + code.len());
    prefixed.push_str(TYPESCRIPT_NOCHECK_DIRECTIVE);
    prefixed.push_str(code.as_str());
    *code = prefixed;
    for mapping in mappings {
        mapping.gen_range.start += offset;
        mapping.gen_range.end += offset;
        for sub_span in &mut mapping.sub_spans {
            sub_span.gen_range.start += offset;
            sub_span.gen_range.end += offset;
        }
    }
    for link in semantic_links.iter_mut().chain(prop_default_key_links) {
        link.shift_generated_ranges(offset);
    }
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
        code: CONTENT_MAPPER_SFC_PARSE_ERROR_CODE,
    }
}

fn generated_diagnostic(source: &str, diagnostic: &Diagnostic) -> ContentMapperDiagnostic {
    let index = vize_l0::line_index::LineIndex::new(source);
    let start = index
        .line_col_to_offset(diagnostic.line, diagnostic.column)
        .unwrap_or(0);
    let (start, length) = checked_source_span(source, start, start);
    ContentMapperDiagnostic {
        message_text: diagnostic.message.clone(),
        start,
        length,
        code: CONTENT_MAPPER_GENERATED_DIAGNOSTIC_CODE,
    }
}

fn checked_source_span(source: &str, start: usize, end: usize) -> (usize, usize) {
    let start = start.min(source.len());
    let start = source.floor_char_boundary(start);
    let end = end.min(source.len()).max(start);
    let end = source.ceil_char_boundary(end);
    let default_length = source
        .get(start..)
        .and_then(|rest| rest.chars().next())
        .map_or(0, char::len_utf8);
    (start, end.saturating_sub(start).max(default_length))
}

#[derive(Clone, Debug)]
struct SpanCandidate {
    generated: Range<usize>,
    original: Range<usize>,
    kind: ContentMapperSpanKind,
}

fn protocol_spans(
    source: &str,
    generated: &str,
    mappings: &[VizeMapping],
    metadata: &[ProjectionMeta],
) -> Vec<ContentMapperSpan> {
    // Protocol diagnostics have no feature bit. Preserve all diagnostic
    // geometry while withholding editor operations from inference-only rows.
    let mut diagnostic_ranges = mappings
        .iter()
        .zip(metadata)
        .filter_map(|(row, meta)| {
            (meta.features == ProjectionFeatures::DIAGNOSTICS).then_some(row.gen_range.clone())
        })
        .collect::<Vec<_>>();
    diagnostic_ranges.sort_by_key(|range| range.start);
    let mut diagnostic_regions: Vec<Range<usize>> = Vec::new();
    for range in diagnostic_ranges {
        if let Some(previous) = diagnostic_regions.last_mut()
            && range.start <= previous.end
        {
            previous.end = previous.end.max(range.end);
        } else {
            diagnostic_regions.push(range);
        }
    }
    let mut candidates = quoted_spans::candidates(source, generated, mappings);

    masked_spans::expand(&mut candidates, source, generated);
    let mut accepted = span_normalize::normalize(candidates);
    accepted.sort_by_key(|candidate| candidate.generated.start);
    let mut spans = Vec::with_capacity(accepted.len());
    for (index, candidate) in accepted.iter().enumerate() {
        let after =
            diagnostic_regions.partition_point(|range| range.start <= candidate.generated.start);
        let diagnostic_only = after
            .checked_sub(1)
            .and_then(|index| diagnostic_regions.get(index))
            .is_some_and(|range| candidate.generated.end <= range.end);
        let features = if diagnostic_only {
            0
        } else {
            content_mapper_span_features(generated, candidate.generated.start, candidate.kind)
        };
        let previous_end = index
            .checked_sub(1)
            .and_then(|previous| accepted.get(previous))
            .map_or(0, |previous| previous.generated.end);
        let next_start = accepted
            .get(index + 1)
            .map_or(generated.len(), |next| next.generated.start);
        let delimiters = synthetic_delimiters(generated, &candidate.generated)
            .filter(|(open, close)| previous_end <= *open && *close < next_start);
        let target = &candidate.original;
        let delimiter = |at: usize| {
            ContentMapperSpan([
                at,
                1,
                target.start,
                target.len(),
                ContentMapperSpanKind::Atom as usize,
                0,
            ])
        };
        spans.extend(delimiters.map(|(open, _)| delimiter(open)));
        spans.push(ContentMapperSpan([
            candidate.generated.start,
            candidate.generated.len(),
            candidate.original.start,
            candidate.original.len(),
            candidate.kind as usize,
            features,
        ]));
        spans.extend(delimiters.map(|(_, close)| delimiter(close)));
    }
    spans
}

/// The generated quotes around a mapped name, when no authored span owns them.
///
/// TypeScript reports a string-literal node (a quoted property key, a CSS
/// module name) from its opening quote, and maps a position in an unmapped gap
/// to the end of the preceding span. Diagnostic-only atoms on both quotes keep
/// such a diagnostic on the authored name instead of the previous attribute.
fn synthetic_delimiters(generated: &str, range: &Range<usize>) -> Option<(usize, usize)> {
    let open = range.start.checked_sub(1)?;
    let quote = *generated.as_bytes().get(open)?;
    (matches!(quote, b'"' | b'\'') && generated.as_bytes().get(range.end) == Some(&quote))
        .then_some((open, range.end))
}

fn candidate(
    source: &str,
    generated: &str,
    generated_range: Range<usize>,
    original_range: Range<usize>,
) -> Option<SpanCandidate> {
    if generated_range.is_empty() || original_range.is_empty() {
        return None;
    }
    let generated_text = generated.get(generated_range.clone())?;
    let original_text = source.get(original_range.clone())?;
    if !is_synthetic_content_mapper_identifier(generated_text)
        && let Some(relative_start) = generated_text.find(original_text)
    {
        let start = generated_range.start + relative_start;
        return Some(SpanCandidate {
            generated: start..start + original_text.len(),
            original: original_range,
            kind: ContentMapperSpanKind::Verbatim,
        });
    }

    Some(SpanCandidate {
        generated: generated_range,
        original: original_range,
        kind: if is_alias_projection(generated_text, original_text) {
            ContentMapperSpanKind::Alias
        } else {
            ContentMapperSpanKind::Atom
        },
    })
}

fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#[cfg(test)]
mod tests;

#[cfg(test)]
#[expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
mod unused_props_tests;

#[cfg(test)]
mod unknown_options_tests;

#[cfg(test)]
mod directive_tests;

#[cfg(test)]
mod typed_router_profile_tests;
