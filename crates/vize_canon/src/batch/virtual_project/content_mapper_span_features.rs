//! TypeScript Content Mapper protocol-v1 feature bits.

use super::ContentMapperSpanKind;

/// TypeScript Content Mapper protocol-v1 feature bits.
///
/// This is deliberately separate from `crate::source_map::MappingFlags`:
/// that compact Vize-internal type has different bit positions, includes
/// diagnostics, and represents only seven capabilities. Protocol diagnostics
/// do not have a feature bit, and exact edits additionally require verbatim
/// geometry in TypeScript.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
enum ContentMapperSpanFeature {
    Hover = 1 << 0,
    SignatureHelp = 1 << 1,
    Completion = 1 << 2,
    Definition = 1 << 3,
    TypeDefinition = 1 << 4,
    Implementation = 1 << 5,
    SourceDefinition = 1 << 6,
    References = 1 << 7,
    DocumentHighlights = 1 << 8,
    Rename = 1 << 9,
    CallHierarchy = 1 << 10,
    CodeActions = 1 << 11,
    Formatting = 1 << 12,
    InlayHints = 1 << 13,
    SemanticTokens = 1 << 14,
    FoldingRanges = 1 << 15,
    SelectionRanges = 1 << 16,
    LinkedEditing = 1 << 17,
    AutoInsert = 1 << 18,
    DocumentSymbols = 1 << 19,
    CodeLens = 1 << 20,
}

/// Every protocol-v1 feature bit a mapped span can advertise.
pub(super) const CONTENT_MAPPER_SPAN_FEATURES_ALL: usize = ContentMapperSpanFeature::Hover as usize
    | ContentMapperSpanFeature::SignatureHelp as usize
    | ContentMapperSpanFeature::Completion as usize
    | ContentMapperSpanFeature::Definition as usize
    | ContentMapperSpanFeature::TypeDefinition as usize
    | ContentMapperSpanFeature::Implementation as usize
    | ContentMapperSpanFeature::SourceDefinition as usize
    | ContentMapperSpanFeature::References as usize
    | ContentMapperSpanFeature::DocumentHighlights as usize
    | ContentMapperSpanFeature::Rename as usize
    | ContentMapperSpanFeature::CallHierarchy as usize
    | ContentMapperSpanFeature::CodeActions as usize
    | ContentMapperSpanFeature::Formatting as usize
    | ContentMapperSpanFeature::InlayHints as usize
    | ContentMapperSpanFeature::SemanticTokens as usize
    | ContentMapperSpanFeature::FoldingRanges as usize
    | ContentMapperSpanFeature::SelectionRanges as usize
    | ContentMapperSpanFeature::LinkedEditing as usize
    | ContentMapperSpanFeature::AutoInsert as usize
    | ContentMapperSpanFeature::DocumentSymbols as usize
    | ContentMapperSpanFeature::CodeLens as usize;

/// Read-only features that can safely project through an ordinary synthesized
/// atom. Symbol navigation and every edit-producing feature require either
/// verbatim text or an explicitly recognized whole-symbol projection below.
pub(super) const CONTENT_MAPPER_SPAN_FEATURES_ATOM: usize =
    ContentMapperSpanFeature::Hover as usize | ContentMapperSpanFeature::SignatureHelp as usize;

/// Whole-symbol features that remain exact across a camel/kebab casing rewrite.
pub(super) const CONTENT_MAPPER_SPAN_FEATURES_CASED_SYMBOL: usize = ContentMapperSpanFeature::Hover
    as usize
    | ContentMapperSpanFeature::Definition as usize
    | ContentMapperSpanFeature::TypeDefinition as usize
    | ContentMapperSpanFeature::Implementation as usize
    | ContentMapperSpanFeature::SourceDefinition as usize
    | ContentMapperSpanFeature::References as usize
    | ContentMapperSpanFeature::DocumentHighlights as usize;

pub(super) fn content_mapper_span_features(
    generated: &str,
    start: usize,
    kind: ContentMapperSpanKind,
) -> usize {
    match kind {
        ContentMapperSpanKind::Verbatim => CONTENT_MAPPER_SPAN_FEATURES_ALL,
        ContentMapperSpanKind::Atom if is_cased_symbol_projection(generated, start) => {
            CONTENT_MAPPER_SPAN_FEATURES_CASED_SYMBOL
        }
        ContentMapperSpanKind::Atom => CONTENT_MAPPER_SPAN_FEATURES_ATOM,
    }
}

fn is_cased_symbol_projection(generated: &str, start: usize) -> bool {
    let line_start = generated[..start].rfind('\n').map_or(0, |index| index + 1);
    generated[line_start..start].starts_with("  void __vize_kebab_events_nav_")
}
