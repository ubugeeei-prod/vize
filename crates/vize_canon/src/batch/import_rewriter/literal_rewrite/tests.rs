use oxc_span::SourceType;
use vize_carton::ToCompactString;

use super::super::{ImportRewriter, collect::collect_specifier_occurrences};

fn assert_encoded_identity(source: &str, path: &str, expected_literal: &str) {
    let result = ImportRewriter::new().rewrite_with(source, SourceType::ts(), |_, _| {
        Some(path.to_compact_string())
    });
    let expected = source
        .replace("'target'", expected_literal)
        .replace("\"target\"", expected_literal);
    assert_eq!(result.code.as_str(), expected);
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &result.code, SourceType::ts()).parse();
    assert!(!parsed.panicked);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let specifiers = collect_specifier_occurrences(&result.code, SourceType::ts());
    assert_eq!(specifiers.len(), 1);
    assert_eq!(specifiers.first().unwrap().0.as_str(), path);

    let start = source.find("target").unwrap() as u32;
    let end = start + 6;
    let generated_start = expected.find(expected_literal).unwrap() as u32 + 1;
    let generated_end = generated_start + expected_literal.len() as u32 - 2;
    for offset in start..end {
        assert_eq!(
            result.source_map.get_virtual_offset(offset),
            generated_start
        );
    }
    for offset in generated_start..generated_end {
        assert_eq!(result.source_map.get_original_offset(offset), start);
    }
    for (authored, generated) in [
        (start - 1, generated_start - 1),
        (end, generated_end),
        (end + 1, generated_end + 1),
        (source.len() as u32, expected.len() as u32),
    ] {
        assert_eq!(result.source_map.get_original_offset(generated), authored);
        assert_eq!(result.source_map.get_virtual_offset(authored), generated);
        assert_eq!(
            source
                .get(..authored as usize)
                .unwrap()
                .encode_utf16()
                .count(),
            source
                .get(..result.source_map.get_original_offset(generated) as usize)
                .unwrap()
                .encode_utf16()
                .count()
        );
    }
}

#[test]
fn both_original_quotes_preserve_the_exact_decoded_project_path() {
    for quote in ["'", "\""] {
        for newline in ["\n", "\r\n"] {
            let source = format!(
                "// 日本語🍣{newline}import Widget from {quote}target{quote};{newline}void Widget;"
            );
            assert_encoded_identity(
                &source,
                "/mirror/src/日本語🍣 quoted'\"directory/lib",
                "\"/mirror/src/日本語🍣 quoted'\\\"directory/lib\"",
            );
        }
    }
}

#[test]
fn backslashes_and_each_control_character_use_the_existing_ts_encoder() {
    let source = "import Widget from 'target';\r\nvoid Widget;";
    for (path, literal) in [
        ("a\\b", "\"a\\\\b\""),
        ("a\nb\rc\td", "\"a\\nb\\rc\\td\""),
        ("a\u{08}b\u{0c}c", "\"a\\bb\\fc\""),
        ("a\0b\u{1f}c", "\"a\\u0000b\\u001fc\""),
        ("a\u{2028}b\u{2029}c", "\"a\\u2028b\\u2029c\""),
    ] {
        assert_encoded_identity(source, path, literal);
    }
}

#[test]
fn safe_paths_preserve_the_original_quote_and_other_quote_content() {
    assert_encoded_identity(
        "import W from 'target';",
        "/safe/日本語🍣",
        "'/safe/日本語🍣'",
    );
    assert_encoded_identity("import W from \"target\";", "safe'path", "\"safe'path\"");
    assert_encoded_identity("import W from 'target';", "safe\"path", "'safe\"path'");
}

#[test]
fn source_kinds_share_the_same_encoder_and_mapping_boundary() {
    for source in [
        "export * from 'target';",
        "export { value } from 'target';",
        "const load = () => import('target');",
        "type Value = import('target').Value;",
        "import Value = require('target');",
    ] {
        assert_encoded_identity(source, "a'b\"c", "\"a'b\\\"c\"");
    }
}

#[test]
fn repeated_safe_and_encoded_literals_keep_later_coordinates_exact() {
    let source = "import A from 'safe';\r\nimport B from 'target';\r\nconst 日本語 = [A, B];";
    let result = ImportRewriter::new().rewrite_with(source, SourceType::ts(), |path, _| {
        Some(
            if path == "safe" {
                "/long/safe/path"
            } else {
                "x'\"y"
            }
            .to_compact_string(),
        )
    });
    let expected =
        "import A from '/long/safe/path';\r\nimport B from \"x'\\\"y\";\r\nconst 日本語 = [A, B];";
    assert_eq!(result.code.as_str(), expected);
    for (original, generated) in [
        (source.find("target").unwrap(), expected.find("x'").unwrap()),
        (
            source.find("const").unwrap(),
            expected.find("const").unwrap(),
        ),
    ] {
        assert_eq!(
            result.source_map.get_original_offset(generated as u32),
            original as u32
        );
        assert_eq!(
            result.source_map.get_virtual_offset(original as u32),
            generated as u32
        );
    }
}

#[test]
fn literal_encoding_does_not_add_a_syntax_parse_or_retain_a_second_entry() {
    let source = "import W from 'target';";
    let mut cache = super::super::ModuleSpecifierFactsCache::default();
    let calls = std::cell::Cell::new(0);
    for _ in 0..2 {
        let result = ImportRewriter::new().rewrite_with_cached(
            source,
            SourceType::ts(),
            |_, _| {
                calls.set(calls.get() + 1);
                Some("quoted'\"path".to_compact_string())
            },
            &mut cache,
        );
        assert_eq!(result.code.as_str(), "import W from \"quoted'\\\"path\";");
    }
    let stats = cache.stats();
    assert_eq!(
        (
            stats.parses,
            stats.hits,
            stats.retained_entries,
            calls.get()
        ),
        (1, 1, 1, 2)
    );
    eprintln!(
        "literal_encoding_syntax {stats:?} resolver_calls={}",
        calls.get()
    );
}

#[test]
fn unproven_original_delimiters_cannot_apply_a_replacement() {
    for source in ["target", "'target\"", "target'", "'target"] {
        let mut output = source.to_compact_string();
        let start = source.find("target").unwrap() as u32;
        assert_eq!(
            super::replace(&mut output, source, start, start + 6, "bad'path"),
            None
        );
        assert_eq!(output.as_str(), source);
    }
}
