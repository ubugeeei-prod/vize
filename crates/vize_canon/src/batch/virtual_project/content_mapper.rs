//! TypeScript content-mapper projection for Vue single-file components.

use std::ops::Range;
use std::path::Path;

use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_sfc::{SfcError, SfcParseOptions, parse_sfc};
use vize_l0::{ToCompactString, config::VueVersion};

use crate::batch::Diagnostic;
use crate::batch::error::CorsaResult;
use crate::virtual_ts::{
    ProjectionFeatures, ProjectionMeta, VirtualTsCheckOptions, VirtualTsOptions, VizeMapping,
};

#[path = "content_mapper_alias.rs"]
mod alias;
use alias::{is_alias_projection, is_synthetic_content_mapper_identifier};

#[path = "content_mapper_protocol.rs"]
mod protocol;
use protocol::protocol_semantic_links;
pub use protocol::{
    CONTENT_MAPPER_GENERATED_DIAGNOSTIC_CODE, CONTENT_MAPPER_SFC_PARSE_ERROR_CODE,
    CONTENT_MAPPER_VIRTUAL_EXTENSION, ContentMapperDiagnostic, ContentMapperDiagnosticDirective,
    ContentMapperDiagnosticDirectives, ContentMapperSemanticLink, ContentMapperSpan,
    ContentMapperTransform, ContentMapperUnusedExpectDiagnostic,
};

#[path = "content_mapper_directives.rs"]
mod directives;
use directives::sfc_diagnostic_directives;

#[path = "content_mapper_component_name.rs"]
mod component_name;
use component_name::content_mapper_component_name;

use super::build::{descriptor_uses_jsx_script, prepend_vue_jsx_reference};
use super::diagnostics::invalid_sfc_fallback_virtual_ts;
use super::javascript_sfc::prepare_script_blocks;
use super::vue_codegen::{GeneratedVueFile, VueCodegenOptions, generate_vue_virtual_ts};

#[path = "content_mapper_masked_spans.rs"]
mod masked_spans;
#[path = "content_mapper_quoted_spans.rs"]
mod quoted_spans;
#[path = "content_mapper_span_features.rs"]
mod span_features;
#[path = "content_mapper_span_normalize.rs"]
mod span_normalize;

use span_features::content_mapper_span_features;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
enum ContentMapperSpanKind {
    Verbatim = 0,
    Atom = 1,
    Alias = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ContentMapperTransformOptions {
    options_api: bool,
    preserve_unused_diagnostics: bool,
    pub(super) check_unknown_props: bool,
    pub(super) check_unknown_fallthrough_props: bool,
    pub(super) strict_css_modules: bool,
}

impl Default for ContentMapperTransformOptions {
    fn default() -> Self {
        Self {
            options_api: true,
            preserve_unused_diagnostics: false,
            check_unknown_props: true,
            check_unknown_fallthrough_props: false,
            strict_css_modules: false,
        }
    }
}

impl ContentMapperTransformOptions {
    /// Enable or disable Vue Options API instance bindings.
    #[must_use]
    pub const fn with_options_api(mut self, enabled: bool) -> Self {
        self.options_api = enabled;
        self
    }

    /// Whether Vue Options API instance bindings are enabled.
    #[must_use]
    pub const fn options_api(self) -> bool {
        self.options_api
    }

    /// Preserve diagnostics for user-authored unused locals.
    #[must_use]
    pub const fn with_preserve_unused_diagnostics(mut self, enabled: bool) -> Self {
        self.preserve_unused_diagnostics = enabled;
        self
    }
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
    let use_tsx = descriptor_uses_jsx_script(&descriptor);
    let component_name = content_mapper_component_name(path);
    let GeneratedVueFile {
        mut code,
        mut mappings,
        mapping_meta,
        mut semantic_links,
        mut prop_default_key_links,
        mut diagnostics,
        ..
    } = generate_vue_virtual_ts(
        path,
        content,
        &descriptor,
        &VirtualTsOptions::default(),
        VueCodegenOptions {
            check_options: VirtualTsCheckOptions {
                check_unknown_props: transform_options.check_unknown_props,
                check_unknown_fallthrough_props: transform_options.check_unknown_fallthrough_props,
                strict_css_modules: transform_options.strict_css_modules,
                ..Default::default()
            },
            typed_router_root: None,
            preserve_unused_diagnostics: transform_options.preserve_unused_diagnostics,
            options_api: transform_options.options_api(),
            preserve_authored_component: true,
            preserve_script_on_template_error: true,
            component_name: Some(component_name.as_str()),
            preserve_event_navigation: true,
            legacy_vue2: false,
            dialect: VueVersion::default(),
            template_syntax: TemplateSyntaxMode::default(),
            experimental_in_tag_comments: false,
            experimental_patterned_template: false,
            experimental_strict_slot_children: false,
            hoist_shared_preamble: false,
            omit_vite_client_reference: true,
            runtime_prop_resolve_cache: None,
        },
    )?;

    if use_tsx {
        prepend_vue_jsx_reference(
            &mut code,
            &mut mappings,
            &mut semantic_links,
            &mut prop_default_key_links,
        );
    }

    authored.remap_mappings(&mut mappings);
    for diagnostic in &mut diagnostics {
        (diagnostic.line, diagnostic.column) =
            authored.to_authored_line_column(diagnostic.line, diagnostic.column);
    }
    let content = authored_content;
    let spans = protocol_spans(content, &code, &mappings, &mapping_meta);
    Ok(ContentMapperTransform {
        extension: CONTENT_MAPPER_VIRTUAL_EXTENSION,
        // Directive comments are read from HTML templates and SFC blocks only.
        diagnostic_directives: (!authored.is_derived())
            .then(|| sfc_diagnostic_directives(content, &descriptor, &spans))
            .flatten(),
        mappings: spans,
        semantic_links: protocol_semantic_links(&semantic_links),
        diagnostics: diagnostics
            .iter()
            .map(|diagnostic| generated_diagnostic(content, diagnostic))
            .collect(),
        text: code,
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
        let delimiter = |at: usize| {
            ContentMapperSpan([
                at,
                1,
                candidate.original.start,
                candidate.original.len(),
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
#[path = "content_mapper_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "content_mapper_directive_tests.rs"]
mod directive_tests;
