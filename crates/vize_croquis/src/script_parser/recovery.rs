//! Keep semantic context while a member name or call is being edited.

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use vize_carton::String;

/// Parse facts for analysis without discarding an entire script at `value.` or
/// `value?.`. Diagnostics and emitted code must still use the authored source.
///
/// Valid programs take the ordinary parser path. An unfinished call followed
/// by a declaration can produce a non-fatal partial AST, so its opening
/// parenthesis is blanked before analysis. On a fatal error, only a member or
/// call operator at the parser diagnostic is blanked in an arena-owned view.
/// Every existing byte offset is preserved; comments, strings, numeric literals
/// and valid optional chains are untouched. A bounded retry also handles
/// independent holes in nested expressions.
pub fn parse_program_for_analysis<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
) -> ParserReturn<'a> {
    let original = Parser::new(allocator, source, source_type).parse();
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
        let mut parsed = Parser::new(allocator, view, source_type).parse();
        if !parsed.panicked {
            // The recovered AST only borrows the repaired view to establish
            // structure. Its spans are byte-for-byte compatible with the
            // authored source, so every later source slice must use the
            // authored bytes and keep the incomplete operator visible to the
            // native TypeScript diagnostic owner.
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

/// A fatal parse can report only the unmatched call's eventual closing point,
/// so no diagnostic label necessarily remains beside the authored `(`. The
/// line boundary is still deterministic: a declaration cannot continue an
/// unfinished call whose preceding line ends at its opening parenthesis.
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

/// TypeScript ends an unfinished call at a line break when the following line
/// starts a declaration. Blank only the unmatched opening parenthesis in the
/// analysis view so imports and later bindings remain available to projection;
/// emitted code and diagnostics continue to use the authored `(`.
fn incomplete_call_operator(parsed: &ParserReturn<'_>, source: &str) -> Option<Range<usize>> {
    parsed.diagnostics.iter().rev().find_map(|error| {
        let label = error
            .labels
            .iter()
            .find(|label| label.primary())
            .or_else(|| error.labels.first())?;
        let position = (label.offset() as usize).min(source.len());
        let line_break = source
            .get(..position)?
            .char_indices()
            .rev()
            .find(|(_, character)| matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
            .map(|(offset, _)| offset)?;
        let end = member_trivia_start(parsed, source, line_break)?;
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

fn starts_declaration(source: &str) -> bool {
    [
        "const",
        "let",
        "var",
        "function",
        "class",
        "interface",
        "type",
        "enum",
        "import",
        "export",
    ]
    .into_iter()
    .any(|keyword| {
        source.strip_prefix(keyword).is_some_and(|rest| {
            rest.chars().next().is_some_and(|character| {
                character.is_whitespace() || matches!(character, '{' | '*')
            })
        })
    })
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
    let mut end = member_trivia_start(parsed, source, position.min(source.len()))?;
    // The unexpected token must delimit the unfinished member name. A line
    // break also does so when the next statement has already been typed:
    // TypeScript ends `value.` there for recovery and reports TS1003 at the
    // dot, while OXC can point its fatal diagnostic inside `const` / `function`.
    let followed_by_line_break = if source.get(..end)?.ends_with('.') {
        false
    } else {
        let line_break = source
            .get(..position)?
            .char_indices()
            .rev()
            .find(|(_, character)| matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
            .map(|(offset, _)| offset)?;
        end = member_trivia_start(parsed, source, line_break)?;
        true
    };
    if source
        .as_bytes()
        .get(position)
        .is_some_and(|byte| !matches!(byte, b';' | b',' | b'(' | b')' | b']' | b'}'))
        && !followed_by_line_break
    {
        return None;
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

fn member_trivia_start(parsed: &ParserReturn<'_>, source: &str, mut end: usize) -> Option<usize> {
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
            return Some(end);
        }
    }
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
    fn incomplete_members_before_the_next_statement_keep_the_whole_script() {
        for next in ["const next = 1;", "function next() {}"] {
            let source = vize_carton::cstr!(
                "import {{ ref }} from 'vue';\nconst items = ref([]);\nitems.value.\n{next}"
            );
            let allocator = Allocator::default();
            let parsed = parse_program_for_analysis(&allocator, &source, SourceType::ts());
            assert!(!parsed.panicked, "{next}: {:?}", parsed.diagnostics);
            assert_eq!(parsed.program.body.len(), 4, "{next}");
            assert_eq!(parsed.program.source_text, source.as_str(), "{next}");
            assert!(
                !parsed.diagnostics.is_empty(),
                "authored error must survive"
            );
            let result = parse_script_setup(&source);
            assert_eq!(result.import_statements.len(), 1, "{next}");
            assert!(result.bindings.bindings.contains_key("items"), "{next}");
            assert!(result.bindings.bindings.contains_key("next"), "{next}");
        }
    }

    #[test]
    fn incomplete_calls_before_the_next_declaration_keep_the_whole_script() {
        for next in ["const selected = null;", "function pick() {}"] {
            let source = vize_carton::cstr!(
                "import {{ ref }} from 'vue';\nconst items = ref([]);\npick(\n{next}"
            );
            let allocator = Allocator::default();
            let parsed = parse_program_for_analysis(&allocator, &source, SourceType::ts());
            assert!(!parsed.panicked, "{next}: {:?}", parsed.diagnostics);
            assert_eq!(parsed.program.body.len(), 4, "{next}");
            assert_eq!(parsed.program.source_text, source.as_str(), "{next}");
            assert!(
                !parsed.diagnostics.is_empty(),
                "authored error must survive"
            );
            let result = parse_script_setup(&source);
            assert_eq!(result.import_statements.len(), 1, "{next}");
            assert!(result.bindings.bindings.contains_key("items"), "{next}");
            let binding = if next.starts_with("const") {
                "selected"
            } else {
                "pick"
            };
            assert!(result.bindings.bindings.contains_key(binding), "{next}");
        }
    }

    #[test]
    fn fatal_incomplete_call_keeps_rich_setup_bindings() {
        let source = r#"import { ref } from "vue";
import MyButton from "./MyButton.vue";
import type { Item } from "./types";

const items = ref<Item[]>([]);
const selected = ref<Item | null>(null);
pick(
function pick(item: Item): void {
  selected.value = item;
}
"#;
        let allocator = Allocator::default();
        let raw = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(raw.panicked);

        let parsed = parse_program_for_analysis(&allocator, source, SourceType::ts());
        assert!(!parsed.panicked, "{:?}", parsed.diagnostics);
        assert_eq!(parsed.program.body.len(), 7);
        assert_eq!(parsed.program.source_text, source);
        let result = parse_script_setup(source);
        assert_eq!(result.import_statements.len(), 3);
        for binding in ["ref", "MyButton", "items", "selected", "pick"] {
            assert!(result.bindings.bindings.contains_key(binding), "{binding}");
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
