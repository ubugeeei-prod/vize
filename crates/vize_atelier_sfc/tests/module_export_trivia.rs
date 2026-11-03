//! Module-shape splice boundaries belong to the returned bytes (#11764).

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::module_shape::{analyze_module_shape, finalize_module_output_with_map};

#[test]
fn export_keyword_offsets_accept_all_parsed_trivia() {
    for prefix in ["", "const label = '雪😀';\r\n"] {
        for gap in [
            " ",
            "\r\n",
            "/* note */",
            "/**//**/",
            "/* default decoy */ \r\n// export decoy\r\n",
            "\u{feff}",
            "\u{2003}/* 😀 */\u{feff}",
            "// note\u{2028}",
            "// note\u{2029}",
        ] {
            let source = format!("{prefix}export{gap}default/* tail */ {{ label: '雪' }};");
            let keyword_end = prefix.len() + "export".len() + gap.len() + "default".len();
            let expected = format!("{prefix}const _sfc_main =/* tail */ {{ label: '雪' }};");
            let map = serde_json::json!({
                "version": 3, "sources": ["Trivia.vue"], "names": [], "mappings": "AAAA"
            });
            for preserve_typescript in [false, true] {
                let (code, shape, returned_map) = finalize_module_output_with_map(
                    source.as_str().into(),
                    preserve_typescript,
                    Some(map.clone()),
                );
                assert_eq!(code.as_str(), source);
                assert_eq!(returned_map, Some(map.clone()));
                let shape = shape.expect("valid authored module");
                assert_eq!(shape.default_export_start, Some(prefix.len() as u32));
                assert_eq!(
                    shape.default_export_keyword_end,
                    Some(keyword_end as u32),
                    "{source}"
                );
                assert_eq!(shape.default_export_end, Some(source.len() as u32));
                let start = shape.default_export_start.unwrap() as usize;
                let end = shape.default_export_keyword_end.unwrap() as usize;
                let rewritten = format!("{}const _sfc_main ={}", &code[..start], &code[end..]);
                assert_eq!(rewritten, expected);
                let allocator = Allocator::default();
                let parsed = Parser::new(&allocator, &rewritten, SourceType::mjs()).parse();
                assert!(
                    parsed.diagnostics.is_empty(),
                    "{rewritten}: {:?}",
                    parsed.diagnostics
                );
            }
        }
    }
}

#[test]
fn comment_decoys_and_invalid_modules_do_not_gain_export_boundaries() {
    for source in [
        "const text = 'export/* note */default {}'; // export default {}\r\n",
        "/* export default {} */ const text = '雪😀';",
    ] {
        let shape = analyze_module_shape(source).expect("valid module without export");
        assert!(!shape.has_default_export);
        assert_eq!(shape.default_export_keyword_end, None);
    }
    for source in [
        "export/* unterminated",
        "export/* note */default const = 1;",
    ] {
        assert_eq!(analyze_module_shape(source), None);
    }
}

#[test]
fn large_comment_gaps_keep_exact_utf8_byte_offsets() {
    let gap = "/* 雪😀 default */\r\n".repeat(4096);
    let source = format!("export{gap}default {{}};");
    let shape = analyze_module_shape(&source).expect("large valid comment gap");
    assert_eq!(shape.default_export_start, Some(0));
    assert_eq!(
        shape.default_export_keyword_end,
        Some(("export".len() + gap.len() + "default".len()) as u32)
    );
    assert_eq!(shape.default_export_end, Some(source.len() as u32));
}
