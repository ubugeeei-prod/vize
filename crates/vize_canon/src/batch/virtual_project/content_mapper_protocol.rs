//! Protocol DTOs for the TypeScript content mapper.

use serde::Serialize;
use vize_carton::String as CompactString;

/// The virtual extension every Vize transform output is parsed as.
///
/// TypeScript resolves the virtual syntax per transform response rather than
/// from the package manifest, so this travels on the wire. Vize always emits
/// `.tsx` because the generated module can contain a JSX render expression even
/// when the authored `<script>` block has no JSX of its own.
pub const CONTENT_MAPPER_VIRTUAL_EXTENSION: &str = ".tsx";

/// A TypeScript content-mapper transform result.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapperTransform {
    pub text: CompactString,
    /// The virtual extension TypeScript parses `text` as.
    pub extension: &'static str,
    pub mappings: Vec<ContentMapperSpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_links: Vec<ContentMapperSemanticLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_directives: Option<ContentMapperDiagnosticDirectives>,
    pub diagnostics: Vec<ContentMapperDiagnostic>,
}

/// A protocol v1 span tuple:
/// `[generatedStart, generatedLength, originalStart, originalLength, kind, featureMask]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ContentMapperSpan(pub [usize; 6]);

/// Stable semantic links between generated ranges.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapperSemanticLink {
    pub source_start: usize,
    pub source_length: usize,
    pub target_start: usize,
    pub target_length: usize,
    pub kind: &'static str,
}

/// A diagnostic expressed in the mapper's negotiated UTF-8 coordinates.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapperDiagnostic {
    pub message_text: CompactString,
    pub start: usize,
    pub length: usize,
}

/// Framework directives that suppress TypeScript diagnostics in virtual
/// ranges, shared across the compact directive tuples.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapperDiagnosticDirectives {
    pub unused_expect_directive_diagnostics: Vec<ContentMapperUnusedExpectDiagnostic>,
    pub directives: Vec<ContentMapperDiagnosticDirective>,
}

/// The error TypeScript reports when an expect directive suppressed nothing.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapperUnusedExpectDiagnostic {
    pub code: i32,
    pub message_text: CompactString,
}

/// Policy stored in a mapped diagnostic directive tuple.
pub const DIRECTIVE_POLICY_IGNORE: usize = 0;
/// Policy for directives that must suppress at least one diagnostic.
pub const DIRECTIVE_POLICY_EXPECT: usize = 1;

/// A mapped diagnostic directive tuple:
/// `[originalStart, originalLength, virtualStart, virtualEnd, policy,
/// unusedExpectDirectiveIndex]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ContentMapperDiagnosticDirective(pub [usize; 6]);

pub(super) fn protocol_semantic_links(
    links: &[crate::virtual_ts::VizeSemanticLink],
) -> Vec<ContentMapperSemanticLink> {
    links
        .iter()
        .filter_map(|link| match link.kind {
            crate::virtual_ts::VizeSemanticLinkKind::VueSetupTemplateRefUnwrap => {
                Some(ContentMapperSemanticLink {
                    source_start: link.source_range.start,
                    source_length: link.source_range.len(),
                    target_start: link.target_range.start,
                    target_length: link.target_range.len(),
                    kind: "vueSetupTemplateRefUnwrap",
                })
            }
            // This edge drives Vize's canonical workspace queries. Protocol v1
            // has no corresponding kind, so it must not reach upstream.
            crate::virtual_ts::VizeSemanticLinkKind::VueComponentPropNavigation => None,
        })
        .collect()
}
