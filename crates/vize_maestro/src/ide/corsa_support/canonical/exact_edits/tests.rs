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
    let source = "<script setup lang=\"ts\">\r\nconst emoji = '😀'; const value = 1;\r\n</script>";
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
