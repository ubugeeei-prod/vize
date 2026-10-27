//! Exact public ParserReturn parity with the pre-optimization recovery algorithm.
#![expect(
    clippy::disallowed_macros,
    reason = "complete Debug packets are test oracles"
)]
#![expect(
    clippy::disallowed_types,
    reason = "complete Debug packets use owned strings"
)]

use super::{
    Allocator, Parser, ParserReturn, SourceType, String, incomplete_call_before_declaration,
    incomplete_call_operator, missing_member_operator, parse_program_for_analysis,
};

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
