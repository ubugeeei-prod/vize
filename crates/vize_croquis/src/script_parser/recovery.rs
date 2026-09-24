//! Keep semantic context while a member name is being edited.

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use vize_carton::String;

/// Parse facts for analysis without discarding an entire script at `value.` or
/// `value?.`. Diagnostics and emitted code must still use the authored source.
///
/// Valid programs take the ordinary parser path. On a fatal error, only a
/// member operator at the parser's missing-member diagnostic is blanked
/// in an arena-owned analysis view. Every existing byte offset is preserved;
/// comments, strings, numeric literals and valid optional chains are untouched.
/// A bounded retry also handles independent holes in nested expressions.
pub fn parse_program_for_analysis<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
) -> ParserReturn<'a> {
    let original = Parser::new(allocator, source, source_type).parse();
    if !original.panicked {
        return original;
    }

    let mut analysis = String::from(source);
    for _ in 0..4 {
        let view = allocator.alloc_str(&analysis);
        let mut parsed = Parser::new(allocator, view, source_type).parse();
        if !parsed.panicked {
            parsed.diagnostics = original.diagnostics;
            return parsed;
        }
        let Some(range) = missing_member_operator(&parsed, view) else {
            break;
        };
        analysis.replace_range(range.clone(), if range.len() == 2 { "  " } else { " " });
    }
    original
}

fn missing_member_operator(parsed: &ParserReturn<'_>, source: &str) -> Option<Range<usize>> {
    let error = parsed.diagnostics.last()?;
    let label = error
        .labels
        .iter()
        .find(|label| label.primary())
        .or_else(|| error.labels.first())?;
    let position = label.offset() as usize;
    // OXC points its optional-chain missing-identifier diagnostic at `?.`
    // itself, while the ordinary dot diagnostic points at the next token.
    if label.len() == 2 && source.get(position..position + 2) == Some("?.") {
        return Some(position..position + 2);
    }
    // The unexpected token must delimit the unfinished member name.
    if source
        .as_bytes()
        .get(position)
        .is_some_and(|byte| !matches!(byte, b';' | b',' | b'(' | b')' | b']' | b'}'))
    {
        return None;
    }
    let mut end = position.min(source.len());
    loop {
        end = source.get(..end)?.trim_end().len();
        // OXC retains lexed comments even after a fatal parse error. A comment
        // after the dot is trivia; content inside it is never a repair target.
        if let Some(comment) = parsed
            .program
            .comments
            .iter()
            .find(|comment| comment.span.end as usize == end)
        {
            end = comment.span.start as usize;
        } else {
            break;
        }
    }
    let before = source.get(..end)?.strip_suffix('.')?;
    let start = before.strip_suffix('?').map_or(before.len(), str::len);
    // A spread token or an absent receiver is not a missing member name.
    let receiver = source.get(..start).unwrap_or_default().trim_end();
    if receiver.is_empty() || receiver.ends_with('.') {
        return None;
    }
    Some(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script_parser::parse_script_setup;

    #[test]
    fn incomplete_members_keep_imports_macros_and_authored_binding_offsets() {
        for expression in [
            "slots.",
            "slots?.",
            "slots. /* docs */",
            "slots?. // docs\r\n",
            "(() => slots.)()",
            "[slots., slots?.]",
            "slots.({ total: 1 })",
        ] {
            let source = vize_carton::cstr!(
                "import {{ useSlots }} from 'vue';\nconst 雪 = 1;\ndefineSlots<{{ summary(props: {{ total: number }}): any }}>();\nconst slots = useSlots();\n{expression}"
            );
            let allocator = Allocator::default();
            let parsed = parse_program_for_analysis(&allocator, &source, SourceType::ts());
            assert!(!parsed.panicked, "{expression}: {:?}", parsed.diagnostics);
            assert!(
                !parsed.diagnostics.is_empty(),
                "authored error must survive"
            );
            let result = parse_script_setup(&source);
            assert_eq!(result.import_statements.len(), 1, "{expression}");
            assert!(result.macros.define_slots().is_some(), "{expression}");
            let mut bindings: Vec<_> = result
                .bindings
                .bindings
                .keys()
                .map(|name| name.as_str())
                .collect();
            bindings.sort_unstable();
            assert_eq!(bindings, ["slots", "useSlots", "雪"], "{expression}");
            let import = &result.import_statements[0];
            assert_eq!(
                source
                    .get(import.start as usize..import.end as usize)
                    .unwrap_or_default(),
                "import { useSlots } from 'vue';"
            );
        }
    }

    #[test]
    fn valid_literals_and_comments_are_not_rewritten() {
        for source in [
            "const a = 1.;",
            "const a = 'slots.';",
            "// slots.\nconst a = 1;",
            "slots?.summary();",
        ] {
            let allocator = Allocator::default();
            let parsed = parse_program_for_analysis(&allocator, source, SourceType::ts());
            assert!(
                !parsed.panicked && parsed.diagnostics.is_empty(),
                "{source}"
            );
            assert_eq!(parsed.program.source_text, source);
        }
    }

    #[test]
    fn unrelated_parse_errors_do_not_invent_semantic_facts() {
        let allocator = Allocator::default();
        let parsed = parse_program_for_analysis(&allocator, "const a = ;", SourceType::ts());
        assert!(parsed.panicked);
    }
}
