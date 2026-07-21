//! Shared translation of generated virtual-TS offsets back to SFC source.
//!
//! Both the language server and the batch type checker must place a
//! diagnostic on the same authored bytes, so the sub-span-aware offset
//! arithmetic lives here and is consumed by both paths.

use super::types::VizeMapping;

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
mod tests {
    use super::super::types::{VizeMapping, VizeSubSpan};
    use super::*;

    #[test]
    fn prefers_exact_expression_sub_spans() {
        let mapping = VizeMapping {
            gen_range: 10..80,
            src_range: 100..140,
            sub_spans: vec![VizeSubSpan {
                gen_range: 20..43,
                src_range: 107..130,
            }],
        };
        assert_eq!(map_generated_offset_to_source(&mapping, 20), 107);
        assert_eq!(map_generated_offset_to_source(&mapping, 43), 130);
    }

    #[test]
    fn clamps_generated_overflow_to_the_authored_range() {
        let mapping = VizeMapping {
            gen_range: 0..100,
            src_range: 10..20,
            sub_spans: Vec::new(),
        };
        assert_eq!(map_generated_offset_to_source(&mapping, 95), 20);
    }

    #[test]
    fn selects_the_narrowest_mapping_for_an_offset() {
        let mappings = [
            VizeMapping {
                gen_range: 0..100,
                src_range: 0..100,
                sub_spans: Vec::new(),
            },
            VizeMapping {
                gen_range: 40..60,
                src_range: 200..220,
                sub_spans: Vec::new(),
            },
        ];
        let mapping = mapping_for_generated_offset(&mappings, 45).expect("mapping");
        assert_eq!(mapping.src_range, 200..220);
    }

    #[test]
    fn maps_ranges_and_keeps_at_least_one_authored_byte() {
        let mappings = [VizeMapping {
            gen_range: 10..80,
            src_range: 100..140,
            sub_spans: vec![VizeSubSpan {
                gen_range: 20..43,
                src_range: 107..130,
            }],
        }];
        assert_eq!(
            map_generated_range_to_source(&mappings, 20, 43),
            Some((107, 130))
        );
        assert_eq!(
            map_generated_range_to_source(&mappings, 20, 20),
            Some((107, 108))
        );
        assert_eq!(map_generated_range_to_source(&mappings, 5, 20), None);
    }
}
