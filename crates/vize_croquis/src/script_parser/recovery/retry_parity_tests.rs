//! Exact public ParserReturn parity with the pre-optimization recovery algorithm.
#![expect(
    clippy::disallowed_macros,
    reason = "complete Debug packets are test oracles"
)]
#![expect(
    clippy::disallowed_types,
    reason = "complete Debug packets use owned strings"
)]

use std::ops::Range;

use super::{
    Allocator, Parser, ParserReturn, SourceType, String, incomplete_call_operator,
    missing_member_operator, parse_program_for_analysis, starts_declaration,
};

// Scalar helper frozen from 21dbb3573d4 before vectorizing the producer.
fn incomplete_call_before_declaration(source: &str) -> Option<Range<usize>> {
    source.char_indices().find_map(|(line_break, character)| {
        if !matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
            return None;
        }
        let end = source.get(..line_break)?.trim_end().len();
        let before = source.get(..end)?.strip_suffix('(')?.trim_end();
        if before.is_empty()
            || !before.chars().next_back().is_some_and(|character| {
                character.is_alphanumeric() || matches!(character, '_' | '$' | ')' | ']')
            })
        {
            return None;
        }
        let next = source.get(line_break..)?.trim_start();
        starts_declaration(next).then_some(end - 1..end)
    })
}

// Frozen algorithm from 22323f9552c. Keep its first identical retry and budget:
// this reference is deliberately independent of the optimized dispatch logic.
fn previous<'a>(allocator: &'a Allocator, source: &'a str, kind: SourceType) -> ParserReturn<'a> {
    let original = Parser::new(allocator, source, kind).parse();
    if !original.panicked && original.diagnostics.is_empty() {
        return original;
    }
    let mut analysis = String::from(source);
    if !original.panicked {
        let Some(range) = incomplete_call_operator(&original, source)
            .or_else(|| incomplete_call_before_declaration(source))
        else {
            return original;
        };
        analysis.replace_range(range, " ");
    }
    for _ in 0..4 {
        let view = allocator.alloc_str(&analysis);
        let mut parsed = Parser::new(allocator, view, kind).parse();
        if !parsed.panicked {
            parsed.program.source_text = source;
            parsed.diagnostics = original.diagnostics;
            return parsed;
        }
        let Some(range) = missing_member_operator(&parsed, view)
            .or_else(|| incomplete_call_operator(&parsed, view))
            .or_else(|| incomplete_call_before_declaration(view))
        else {
            break;
        };
        analysis.replace_range(range.clone(), if range.len() == 2 { "  " } else { " " });
    }
    original
}

#[derive(Debug, PartialEq)]
struct Packet {
    program: std::string::String,
    module_record: std::string::String,
    diagnostics: std::string::String,
    irregular_whitespaces: std::string::String,
    tokens: std::string::String,
    panicked: bool,
    flow: bool,
}

fn packet(parsed: &ParserReturn<'_>) -> Packet {
    Packet {
        program: format!("{:?}", parsed.program),
        module_record: format!("{:?}", parsed.module_record),
        diagnostics: format!("{:?}", parsed.diagnostics),
        irregular_whitespaces: format!("{:?}", parsed.irregular_whitespaces),
        tokens: format!("{:?}", parsed.tokens),
        panicked: parsed.panicked,
        flow: parsed.is_flow_language,
    }
}

fn same_packet(source: &str, kind: SourceType) -> bool {
    let old_allocator = Allocator::default();
    let new_allocator = Allocator::default();
    let current = parse_program_for_analysis(&new_allocator, source, kind);
    assert_eq!(
        packet(&current),
        packet(&previous(&old_allocator, source, kind)),
        "all public parser observations must match for {source:?}"
    );
    current.panicked
}

#[test]
fn recovery_reuses_original_errors_without_changing_any_public_parser_field() {
    for source in [
        "const value = 1;",
        "const value = missing.;",
        "const value = missing?.;",
        "const value = missing.",
        "const value = ;",
        "const value = get(\nconst done = 1;",
        "const value = 'literal.'; /* missing. */\nconst done = 1;",
        "const value = 1.;",
        "import { ref } from 'vue'; export { ref };\nconst value = missing.;",
        "const emoji = '𠮷🙂';\r\nconst value = missing?.;\r\nconst done = 1;",
        "const space = '\u{2028}';\u{2028}const value = missing.;",
        "const view = <div>{missing.}</div>;",
    ] {
        for kind in [SourceType::ts(), SourceType::tsx(), SourceType::mjs()] {
            same_packet(source, kind);
        }
    }
}

#[test]
fn retry_budget_and_nested_independent_holes_remain_exact() {
    for holes in 0..=6 {
        let mut body = std::string::String::new();
        for index in 0..holes {
            body.push_str(&format!("const item_{index} = missing_{index}.;\n"));
        }
        body.push_str("const done = 1;\n");
        let source = format!("import {{ ref }} from 'vue';\n{body}");
        assert_eq!(same_packet(&source, SourceType::ts()), holes > 3);
        let nested =
            format!("import {{ ref }} from 'vue';\nfunction outer() {{\n{body}\nreturn done;\n}}");
        assert_eq!(same_packet(&nested, SourceType::ts()), holes > 3);
    }
}

#[test]
fn vector_line_breaks_keep_scalar_byte_offsets_for_every_unicode_scalar() {
    let mut source = String::default();
    for character in (0..=0x10ffff).filter_map(char::from_u32) {
        source.push(character);
    }
    let expected = source
        .char_indices()
        .filter_map(|(offset, character)| {
            matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}').then_some(offset)
        })
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 4);
    assert_eq!(
        super::line_break_offsets(&source).collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        super::line_break_offsets("a\r\nb\u{2028}c\u{2029}d\re\n").collect::<Vec<_>>(),
        [1, 2, 4, 8, 12, 14]
    );
    assert_eq!(
        super::line_break_offsets("\u{2000}\u{2027}\u{202a}\u{20ac}\u{2603}\u{27ff}").count(),
        0
    );
}

#[test]
fn vector_line_breaks_keep_authored_repair_ranges_and_complete_packets() {
    for separator in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
        for padding in ["", " \t", "\u{2000}\u{3000}"] {
            let source = format!(
                "const 雪 = '日本☃😀';{separator}consume({padding}{separator}const done = 1;{separator}"
            );
            let previous = incomplete_call_before_declaration(&source);
            let candidate = super::incomplete_call_before_declaration(&source);
            assert_eq!(candidate, previous);
            assert!(candidate.is_some());
            if let Some(range) = candidate {
                assert_eq!(&source[range], "(");
            }
            for kind in [SourceType::ts(), SourceType::tsx(), SourceType::mjs()] {
                same_packet(&source, kind);
            }
        }
    }
}

#[test]
fn vector_line_breaks_keep_huge_irreparable_authored_programs() {
    for minimum_bytes in [131_072, 1_048_576, 10_485_760] {
        let mut source = String::from("const broken = ;\r\n");
        let mut declarations = 0;
        while source.len() < minimum_bytes {
            use std::fmt::Write;
            assert!(writeln!(source, "const 雪_{declarations}: string = \"日本☃😀\";\r").is_ok());
            declarations += 1;
        }
        assert!(source.len() >= minimum_bytes);
        assert!(declarations > 0);
        assert_eq!(super::incomplete_call_before_declaration(&source), None);
        assert_eq!(incomplete_call_before_declaration(&source), None);
        assert!(same_packet(&source, SourceType::ts()));
    }
}

#[test]
fn hybrid_line_breaks_keep_offsets_across_the_single_scalar_tail() {
    for source in [
        "\r\nhead\u{2028}\u{2009}middle\r\ntail\u{2029}",
        "\u{2014}early\r\nthen ascii\n",
        "ascii\r\nthen late\u{2014}\r\n",
        "\u{2028}\u{2029}\r\n\u{2028}\u{2029}",
        "\u{2009}\u{2014}\u{2027}\r\n\u{2028}\u{2029}",
    ] {
        let expected = source
            .char_indices()
            .filter_map(|(offset, character)| {
                matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}').then_some(offset)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            super::line_break_offsets(source).collect::<Vec<_>>(),
            expected
        );
        let source = format!("const prefix = {source:?};\r\nconsume(\r\nconst done = 1;");
        assert_eq!(
            super::incomplete_call_before_declaration(&source),
            incomplete_call_before_declaration(&source)
        );
        for kind in [SourceType::ts(), SourceType::tsx(), SourceType::mjs()] {
            same_packet(&source, kind);
        }
    }
}
