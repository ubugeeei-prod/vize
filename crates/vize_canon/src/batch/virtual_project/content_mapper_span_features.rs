//! TypeScript Content Mapper protocol-v1 feature bits.

use super::ContentMapperSpanKind;

/// TypeScript Content Mapper protocol-v1 feature bits.
///
/// These positions must mirror `internal/spanmap.Feature` in the pinned
/// upstream revision exactly: `SpanMap.Validate` rejects any segment carrying a
/// bit outside `FeatureAll`, so a stale high bit fails every transform with
/// TS100040 rather than degrading gracefully. Upstream
/// microsoft/typescript-go@cb5a1862e dropped the separate `SourceDefinition`
/// bit (source definition now rides on `Definition`) and shifted every bit
/// above it down by one.
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
    References = 1 << 6,
    DocumentHighlights = 1 << 7,
    Rename = 1 << 8,
    CallHierarchy = 1 << 9,
    CodeActions = 1 << 10,
    Formatting = 1 << 11,
    InlayHints = 1 << 12,
    SemanticTokens = 1 << 13,
    FoldingRanges = 1 << 14,
    SelectionRanges = 1 << 15,
    LinkedEditing = 1 << 16,
    AutoInsert = 1 << 17,
    DocumentSymbols = 1 << 18,
    CodeLens = 1 << 19,
}

/// Every protocol-v1 feature bit a mapped span can advertise.
pub(super) const CONTENT_MAPPER_SPAN_FEATURES_ALL: usize = ContentMapperSpanFeature::Hover as usize
    | ContentMapperSpanFeature::SignatureHelp as usize
    | ContentMapperSpanFeature::Completion as usize
    | ContentMapperSpanFeature::Definition as usize
    | ContentMapperSpanFeature::TypeDefinition as usize
    | ContentMapperSpanFeature::Implementation as usize
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

pub(super) const CONTENT_MAPPER_SPAN_FEATURES_COMPLETION: usize =
    ContentMapperSpanFeature::Completion as usize;

/// Whole-symbol features that remain exact across a spelling rewrite.
pub(super) const CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL: usize = ContentMapperSpanFeature::Hover
    as usize
    | ContentMapperSpanFeature::Definition as usize
    | ContentMapperSpanFeature::TypeDefinition as usize
    | ContentMapperSpanFeature::Implementation as usize
    | ContentMapperSpanFeature::References as usize
    | ContentMapperSpanFeature::DocumentHighlights as usize;

/// Navigation-only features for generated type-level references that can appear
/// as target locations but should not answer source-side hover/completion.
pub(super) const CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET: usize =
    ContentMapperSpanFeature::Definition as usize
        | ContentMapperSpanFeature::TypeDefinition as usize
        | ContentMapperSpanFeature::Implementation as usize
        | ContentMapperSpanFeature::References as usize
        | ContentMapperSpanFeature::DocumentHighlights as usize;

#[cfg(test)]
pub(super) const CONTENT_MAPPER_SPAN_FEATURE_BITS: [(&str, usize); 20] = [
    ("Hover", ContentMapperSpanFeature::Hover as usize),
    (
        "SignatureHelp",
        ContentMapperSpanFeature::SignatureHelp as usize,
    ),
    ("Completion", ContentMapperSpanFeature::Completion as usize),
    ("Definition", ContentMapperSpanFeature::Definition as usize),
    (
        "TypeDefinition",
        ContentMapperSpanFeature::TypeDefinition as usize,
    ),
    (
        "Implementation",
        ContentMapperSpanFeature::Implementation as usize,
    ),
    ("References", ContentMapperSpanFeature::References as usize),
    (
        "DocumentHighlights",
        ContentMapperSpanFeature::DocumentHighlights as usize,
    ),
    ("Rename", ContentMapperSpanFeature::Rename as usize),
    (
        "CallHierarchy",
        ContentMapperSpanFeature::CallHierarchy as usize,
    ),
    (
        "CodeActions",
        ContentMapperSpanFeature::CodeActions as usize,
    ),
    ("Formatting", ContentMapperSpanFeature::Formatting as usize),
    ("InlayHints", ContentMapperSpanFeature::InlayHints as usize),
    (
        "SemanticTokens",
        ContentMapperSpanFeature::SemanticTokens as usize,
    ),
    (
        "FoldingRanges",
        ContentMapperSpanFeature::FoldingRanges as usize,
    ),
    (
        "SelectionRanges",
        ContentMapperSpanFeature::SelectionRanges as usize,
    ),
    (
        "LinkedEditing",
        ContentMapperSpanFeature::LinkedEditing as usize,
    ),
    ("AutoInsert", ContentMapperSpanFeature::AutoInsert as usize),
    (
        "DocumentSymbols",
        ContentMapperSpanFeature::DocumentSymbols as usize,
    ),
    ("CodeLens", ContentMapperSpanFeature::CodeLens as usize),
];

pub(super) fn content_mapper_span_features(
    generated: &str,
    start: usize,
    kind: ContentMapperSpanKind,
) -> usize {
    if is_diagnostic_handler_anchor(generated, start, kind) {
        return 0;
    }
    if is_component_prop_check_anchor(generated, start, kind) {
        return CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET;
    }
    if is_component_prop_alias_key(generated, start) {
        return CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET;
    }
    let prefix = projection_prefix(generated, start);
    match (kind, prefix) {
        (_, Some(prefix))
            if prefix
                .trim_start()
                .starts_with("void __vize_model_events_completion_") =>
        {
            CONTENT_MAPPER_SPAN_FEATURES_COMPLETION
        }
        (_, Some(prefix)) if is_whole_symbol_projection(prefix) => {
            CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL
        }
        (ContentMapperSpanKind::Verbatim, _) => CONTENT_MAPPER_SPAN_FEATURES_ALL,
        (ContentMapperSpanKind::Atom, _) => CONTENT_MAPPER_SPAN_FEATURES_ATOM,
        (ContentMapperSpanKind::Alias, _) => CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL,
    }
}

fn is_diagnostic_handler_anchor(
    generated: &str,
    start: usize,
    kind: ContentMapperSpanKind,
) -> bool {
    if kind != ContentMapperSpanKind::Atom
        || !generated
            .get(start..)
            .is_some_and(|suffix| suffix.starts_with("__vize_handler_"))
    {
        return false;
    }
    let line_start = generated[..start].rfind('\n').map_or(0, |index| index + 1);
    generated[line_start..start].trim_start() == "const "
}

fn is_component_prop_alias_key(generated: &str, start: usize) -> bool {
    let line_start = generated[..start].rfind('\n').map_or(0, |index| index + 1);
    let prefix = generated[line_start..start].trim_start();
    prefix.contains("__VizePropValue<") && prefix.ends_with(", '")
}

fn is_component_prop_check_anchor(
    generated: &str,
    start: usize,
    kind: ContentMapperSpanKind,
) -> bool {
    kind == ContentMapperSpanKind::Atom
        && generated
            .get(start..)
            .is_some_and(|suffix| suffix.starts_with("__vize_prop_check_"))
}

fn projection_prefix(generated: &str, start: usize) -> Option<&str> {
    const MARKER: &[u8] = b"__vize_";
    let probe_start = start.saturating_sub(64);
    generated.as_bytes()[probe_start..start]
        .windows(MARKER.len())
        .any(|window| window == MARKER)
        .then(|| {
            let line_start = generated[..start].rfind('\n').map_or(0, |index| index + 1);
            &generated[line_start..start]
        })
}

fn is_whole_symbol_projection(prefix: &str) -> bool {
    let prefix = prefix.trim_start();
    prefix.starts_with("void __vize_kebab_events_nav_")
        || prefix.starts_with("void __vize_model_events_nav_")
        || prefix.starts_with("/* __vize_model_event */ ")
}
