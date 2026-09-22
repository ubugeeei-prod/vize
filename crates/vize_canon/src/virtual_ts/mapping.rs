//! The one projection mapping model (P4-5a, charter #14).
//!
//! [`ProjectionMapping`] is the span-link container every virtual-language
//! projection hands to its consumers: Corsa/tsgo diagnostics, the
//! content-mapper protocol and Maestro's editor features. It absorbed Canon's
//! retired `source_map` module and Maestro's retired virtual-code source map
//! (both deleted in P4-5a); its rows are the [`VizeMapping`]
//! span links the generators emit, with [`ProjectionMeta`] carrying the
//! per-row feature flags and construct kind those two models used to own.
//!
//! Both the language server and the batch type checker must place a
//! diagnostic on the same authored bytes, so the sub-span-aware offset
//! arithmetic lives here and is consumed by both paths.

mod model;
mod rows;

pub use model::ProjectionMapping;
pub use rows::{
    ProjectionFeatures, ProjectionMeta, ProjectionRow, ProjectionSpanKind, VizeMapping, VizeSubSpan,
};

/// Returns the narrowest mapping whose generated range contains `offset`.
pub fn mapping_for_generated_offset(
    mappings: &[VizeMapping],
    offset: usize,
) -> Option<&VizeMapping> {
    mappings
        .iter()
        .filter(|mapping| mapping.gen_range.contains(&offset))
        .min_by_key(|mapping| {
            mapping
                .gen_range
                .end
                .saturating_sub(mapping.gen_range.start)
        })
}

/// Returns the mapping that should resolve a diagnostic end offset.
///
/// Reuses the start mapping while the end stays inside it so one diagnostic
/// cannot straddle two unrelated source ranges.
pub fn mapping_for_end_offset<'a>(
    mappings: &'a [VizeMapping],
    start_mapping: &'a VizeMapping,
    end_offset: usize,
) -> Option<&'a VizeMapping> {
    if end_offset > start_mapping.gen_range.start && end_offset <= start_mapping.gen_range.end {
        Some(start_mapping)
    } else {
        mapping_for_generated_offset(mappings, end_offset)
    }
}

/// Maps one generated offset into authored source through `mapping`.
///
/// An exact-expression sub-span wins over the enclosing mapping, and both
/// paths clamp to the authored range so synthetic generated text (helper
/// names, punctuation) can never push a diagnostic past the authored bytes.
pub fn map_generated_offset_to_source(mapping: &VizeMapping, generated_offset: usize) -> usize {
    if let Some(span) = mapping.sub_spans.iter().find(|span| {
        generated_offset >= span.gen_range.start && generated_offset <= span.gen_range.end
    }) {
        let generated_relative = generated_offset.saturating_sub(span.gen_range.start);
        let source_len = span.src_range.end.saturating_sub(span.src_range.start);
        return span
            .src_range
            .start
            .saturating_add(generated_relative.min(source_len));
    }

    let generated_relative = generated_offset.saturating_sub(mapping.gen_range.start);
    let source_len = mapping
        .src_range
        .end
        .saturating_sub(mapping.src_range.start);
    mapping
        .src_range
        .start
        .saturating_add(generated_relative.min(source_len))
}

/// Maps a generated diagnostic range into authored source offsets.
///
/// Returns `None` when no mapping covers the start offset; the end offset is
/// resolved through [`mapping_for_end_offset`] and never collapses below one
/// authored byte.
pub fn map_generated_range_to_source(
    mappings: &[VizeMapping],
    start_offset: usize,
    end_offset: usize,
) -> Option<(usize, usize)> {
    let start_mapping = mapping_for_generated_offset(mappings, start_offset)?;
    let src_start = map_generated_offset_to_source(start_mapping, start_offset);
    let src_end = mapping_for_end_offset(mappings, start_mapping, end_offset)
        .map(|mapping| map_generated_offset_to_source(mapping, end_offset))
        .unwrap_or_else(|| {
            let generated_len = end_offset.saturating_sub(start_offset);
            src_start
                .saturating_add(generated_len)
                .min(start_mapping.src_range.end)
        })
        .max(src_start.saturating_add(1));
    Some((src_start, src_end))
}

#[cfg(test)]
mod tests;
