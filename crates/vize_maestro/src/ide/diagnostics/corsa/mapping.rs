//! Translation of Corsa diagnostic ranges back into SFC source coordinates.

use vize_canon::ImportSourceMap;
use vize_s0::line_index::LineBreaks;

pub(super) type LspRangeParts = (u32, u32, u32, u32);

#[allow(clippy::too_many_arguments)]
pub(super) fn map_diagnostic_with_source_mappings(
    virtual_ts: &str,
    source: &str,
    mappings: &[vize_canon::virtual_ts::VizeMapping],
    import_source_map: &ImportSourceMap,
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
) -> Option<LspRangeParts> {
    // Diagnostics come back from Corsa in coordinates of the *rewritten*
    // virtual TS (the one we sent). The byte-range mappings, however, were
    // produced before the `.vue` → `.vue.ts` rewrite. Translate first.
    let start_offset_post =
        LineBreaks::Lsp.position_to_offset(virtual_ts, start_line, start_character)?;
    let end_offset_post = LineBreaks::Lsp
        .position_to_offset(virtual_ts, end_line, end_character)
        .unwrap_or(start_offset_post.saturating_add(1));
    let start_offset = import_source_map.get_original_offset(start_offset_post as u32) as usize;
    let end_offset = import_source_map.get_original_offset(end_offset_post as u32) as usize;
    let (src_start, src_end) = vize_canon::virtual_ts::mapping::map_generated_range_to_source(
        mappings,
        start_offset,
        end_offset,
    )?;

    let (start_line, start_char) = source_offset_to_position(source, src_start);
    let (end_line, end_char) = source_offset_to_position(source, src_end.min(source.len()));
    Some((start_line, end_line, start_char, end_char))
}

pub(super) fn line_character_to_byte_offset(
    text: &str,
    line: u32,
    character: u32,
) -> Option<usize> {
    LineBreaks::Lsp.position_to_offset(text, line, character)
}

pub(super) fn source_offset_to_position(source: &str, offset: usize) -> (u32, u32) {
    LineBreaks::Lsp.offset_to_position(source, offset)
}

#[cfg(test)]
mod tests {
    use vize_canon::virtual_ts::mapping::{
        map_generated_offset_to_source, mapping_for_end_offset, mapping_for_generated_offset,
    };
    use vize_canon::virtual_ts::{VizeMapping, VizeSubSpan};

    #[test]
    fn diagnostic_mapping_prefers_exact_expression_sub_span() {
        let mappings = [VizeMapping {
            gen_range: 10..80,
            src_range: 100..140,
            sub_spans: vec![VizeSubSpan {
                gen_range: 20..43,
                src_range: 107..130,
            }],
        }];
        let mapping = mapping_for_generated_offset(&mappings, 20).expect("mapping present");

        assert_eq!(map_generated_offset_to_source(mapping, 20), 107);
        assert_eq!(map_generated_offset_to_source(mapping, 43), 130);
    }

    #[test]
    fn diagnostic_mapping_prefers_the_narrowest_generated_range() {
        let mappings = [
            VizeMapping {
                gen_range: 0..100,
                src_range: 0..100,
                sub_spans: Vec::new(),
            },
            VizeMapping {
                gen_range: 20..40,
                src_range: 200..220,
                sub_spans: Vec::new(),
            },
        ];

        assert_eq!(
            mapping_for_generated_offset(&mappings, 30)
                .expect("mapping present")
                .src_range,
            200..220
        );
        assert_eq!(
            mapping_for_generated_offset(&mappings, 40)
                .expect("mapping present")
                .src_range,
            0..100
        );
        assert_eq!(
            mapping_for_end_offset(&mappings, &mappings[1], 40)
                .expect("end mapping present")
                .src_range,
            200..220
        );
    }

    #[test]
    fn diagnostic_mapping_preserves_exclusive_range_end() {
        let mapping = VizeMapping {
            gen_range: 20..40,
            src_range: 200..220,
            sub_spans: Vec::new(),
        };

        assert_eq!(map_generated_offset_to_source(&mapping, 20), 200);
        assert_eq!(map_generated_offset_to_source(&mapping, 39), 219);
        assert_eq!(map_generated_offset_to_source(&mapping, 40), 220);
    }
}
