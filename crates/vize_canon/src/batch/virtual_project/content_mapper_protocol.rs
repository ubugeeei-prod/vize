//! Protocol DTOs for the TypeScript content mapper.

use serde::Serialize;
use vize_carton::String as CompactString;

/// A TypeScript content-mapper transform result.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapperTransform {
    pub text: CompactString,
    pub mappings: Vec<ContentMapperSpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_links: Vec<ContentMapperSemanticLink>,
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

pub(super) fn protocol_semantic_links(
    links: &[crate::virtual_ts::VizeSemanticLink],
) -> Vec<ContentMapperSemanticLink> {
    links
        .iter()
        .map(|link| ContentMapperSemanticLink {
            source_start: link.source_range.start,
            source_length: link.source_range.len(),
            target_start: link.target_range.start,
            target_length: link.target_range.len(),
            kind: match link.kind {
                crate::virtual_ts::VizeSemanticLinkKind::VueSetupTemplateRefUnwrap => {
                    "vueSetupTemplateRefUnwrap"
                }
            },
        })
        .collect()
}
