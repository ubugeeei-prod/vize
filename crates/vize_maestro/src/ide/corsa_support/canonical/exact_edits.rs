//! Writable ranges require a complete, length-preserving authored projection.

use tower_lsp::lsp_types::{Position, Range};

use super::CanonicalVirtualDocument;
use crate::ide::{
    IdeContext, diagnostics::VirtualTsResult, offset_to_position, position_to_offset,
};

pub(crate) fn map_canonical_exact_edit_range(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    range: Range,
) -> Option<Range> {
    map_exact_range(&ctx.content, &document.virtual_result, range)
}

pub(super) fn map_exact_range(
    source: &str,
    document: &VirtualTsResult,
    range: Range,
) -> Option<Range> {
    let start = position_to_offset(&document.code, range.start.line, range.start.character)?;
    let end = position_to_offset(&document.code, range.end.line, range.end.character)?;
    if start > end {
        return None;
    }
    let map = &document.import_source_map;
    let pre_start = map.get_original_offset(u32::try_from(start).ok()?) as usize;
    let pre_end = map.get_original_offset(u32::try_from(end).ok()?) as usize;
    if map.get_virtual_offset(pre_start as u32) as usize != start
        || map.get_virtual_offset(pre_end as u32) as usize != end
    {
        return None;
    }
    let spans = document.source_mappings.iter().flat_map(|mapping| {
        std::iter::once((&mapping.gen_range, &mapping.src_range)).chain(
            mapping
                .sub_spans
                .iter()
                .map(|span| (&span.gen_range, &span.src_range)),
        )
    });
    let mut authored = None;
    for (generated, original) in spans {
        if generated.len() != original.len()
            || generated.is_empty()
            || pre_start < generated.start
            || pre_end > generated.end
            || pre_start > pre_end
        {
            continue;
        }
        let source_start = original.start + pre_start - generated.start;
        let source_end = original.start + pre_end - generated.start;
        if source.get(source_start..source_end)? != document.code.get(start..end)? {
            continue;
        }
        let candidate = (source_start, source_end);
        if authored.is_some_and(|previous| previous != candidate) {
            return None;
        }
        authored = Some(candidate);
    }
    let (start, end) = authored?;
    let (line, character) = offset_to_position(source, start);
    let start = Position::new(line, character);
    let (line, character) = offset_to_position(source, end);
    Some(Range::new(start, Position::new(line, character)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ide::corsa_support::canonical_dependency_tests::host_document;
    use tower_lsp::lsp_types::Url;
    use vize_canon::virtual_ts::VizeMapping;

    fn range(text: &str, start: usize, end: usize) -> Range {
        let (line, character) = offset_to_position(text, start);
        let start = Position::new(line, character);
        let (line, character) = offset_to_position(text, end);
        Range::new(start, Position::new(line, character))
    }

    #[test]
    fn exact_edits_preserve_unicode_crlf_and_insertions_and_reject_generated_spans() {
        let source =
            "<script setup lang=\"ts\">\r\nconst emoji = '😀'; const value = 1;\r\n</script>";
        let uri = Url::parse("file:///workspace/App.vue").unwrap();
        let document = host_document(&uri, source);
        let result = &document.virtual_result;
        let source_start = source.find("value").unwrap();
        let generated_start = result.code.find("value = 1").unwrap();
        for length in [0, 5] {
            assert_eq!(
                map_exact_range(
                    source,
                    result,
                    range(&result.code, generated_start, generated_start + length)
                ),
                Some(range(source, source_start, source_start + length)),
            );
        }
        assert_eq!(
            map_exact_range(source, result, range(&result.code, 0, 0)),
            None
        );
        assert_eq!(
            map_exact_range(source, result, range(&result.code, 0, generated_start + 5)),
            None
        );
    }

    #[test]
    fn approximate_and_ambiguous_mappings_are_not_writable() {
        let uri = Url::parse("file:///workspace/App.vue").unwrap();
        let mut document = host_document(&uri, "<script setup>const x = 1;</script>");
        let result = &mut document.virtual_result;
        result.code = "value".into();
        result.import_source_map = Default::default();
        result.source_mappings = vec![VizeMapping {
            gen_range: 0..5,
            src_range: 0..4,
            sub_spans: vec![],
        }];
        assert_eq!(map_exact_range("valu", result, range("value", 0, 4)), None);
        result.source_mappings[0].src_range = 0..5;
        assert_eq!(map_exact_range("other", result, range("value", 0, 5)), None);
        result.source_mappings.push(VizeMapping {
            gen_range: 0..5,
            src_range: 5..10,
            sub_spans: vec![],
        });
        assert_eq!(
            map_exact_range("valuevalue", result, range("value", 0, 5)),
            None
        );
    }
}
