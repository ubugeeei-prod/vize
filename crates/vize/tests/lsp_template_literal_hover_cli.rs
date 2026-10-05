#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/template-literal-hover/StatusBadge.vue.txt"
);

#[test]
fn original_literal_hovers_are_null_and_interpolated_tone_keeps_full_native_type_and_range() {
    for newline in ["\n", "\r\n"] {
        let source = SOURCE.replace('\n', newline);
        let mut fixture = Fixture::new_with_vue(&source);
        assert_eq!(fixture.open(&source), json!([]));
        for (line, character) in [(5, 20), (5, 36), (5, 69), (6, 9)] {
            let offset = byte_offset(&source, line, character);
            assert_eq!(
                position(&source, &source[offset..]),
                json!({ "line": line, "character": character })
            );
            assert_eq!(hover_at(&mut fixture, &source, offset), Value::Null);
        }
        let offset = byte_offset(&source, 5, 51);
        assert_eq!(
            hover_at(&mut fixture, &source, offset),
            json!({
                "contents": { "kind": "markdown", "value": "```typescript\nconst tone: \"accent\" | \"neutral\"\n```" },
                "range": { "start": { "line": 5, "character": 50 }, "end": { "line": 5, "character": 54 } }
            })
        );
        assert_expression_hovers(&mut fixture, &source, "status-badge");
        fixture.shutdown();
    }
}

#[test]
fn native_empty_hovers_keep_same_spelling_literals_comments_and_unsaved_utf16_separate() {
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "<!-- 😀 -->\n"] {
            let controls = SOURCE
                .replace("  <span ", "  <span data-emoji=\"😀\" ")
                .replace(
                    "{{ `current: ${tone}` }}",
                    "{{ `current: ${tone}` }} {{ 'tone' }} {{ `tone` }} {{ tone /* tone */ }}",
                );
            let source = format!("{prefix}{controls}").replace('\n', newline);
            let mut fixture = Fixture::new_with_vue(&source);
            assert_eq!(fixture.open(&source), json!([]));
            assert_control_hovers(&mut fixture, &source, "status-badge");
            let changed = source.replace("status-badge", "tone");
            assert_eq!(fixture.change(&changed, 2), json!([]));
            assert_control_hovers(&mut fixture, &changed, "tone");
            assert_eq!(fixture.change(&source, 3), json!([]));
            assert_control_hovers(&mut fixture, &source, "status-badge");
            fixture.shutdown();
        }
    }
}

fn assert_control_hovers(fixture: &mut Fixture, source: &str, class_name: &str) {
    assert_expression_hovers(fixture, source, class_name);
    for (needle, delta) in [("{{ 'tone' }}", 5), ("{{ `tone` }}", 5), ("/* tone */", 4)] {
        let offset = source.find(needle).unwrap() + delta;
        assert_eq!(hover_at(fixture, source, offset), Value::Null, "{needle}");
    }
    let offset = source.find("{{ tone /*").unwrap() + 3;
    assert_tone_hover(fixture, source, offset, offset);
}

fn assert_expression_hovers(fixture: &mut Fixture, source: &str, class_name: &str) {
    let literal_markers = [
        format!("'{class_name}'"),
        format!("`{class_name}--"),
        "`tone: ".to_owned(),
        "`current: ".to_owned(),
    ];
    for marker in literal_markers {
        let offset = source.find(&marker).unwrap() + 2;
        assert_eq!(hover_at(fixture, source, offset), Value::Null, "{marker}");
    }
    let substitutions: Vec<_> = source
        .match_indices("${tone}")
        .map(|(start, _)| start)
        .collect();
    assert_eq!(substitutions.len(), 3);
    for start in substitutions {
        assert_eq!(
            hover_at(fixture, source, start),
            Value::Null,
            "dollar delimiter"
        );
        assert_eq!(
            hover_at(fixture, source, start + 1),
            Value::Null,
            "opening brace"
        );
        assert_tone_hover(fixture, source, start + 3, start + 2);
    }
}

fn assert_tone_hover(fixture: &mut Fixture, source: &str, cursor: usize, token_start: usize) {
    assert_eq!(
        hover_at(fixture, source, cursor),
        json!({
            "contents": { "kind": "markdown", "value": "```typescript\nconst tone: \"accent\" | \"neutral\"\n```" },
            "range": { "start": position(source, &source[token_start..]), "end": position(source, &source[token_start + 4..]) }
        })
    );
}

fn hover_at(fixture: &mut Fixture, source: &str, offset: usize) -> Value {
    fixture.request("textDocument/hover", source, &source[offset..])
}

fn byte_offset(source: &str, line: usize, units: usize) -> usize {
    let line_offset: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
    let mut seen = 0;
    for (index, character) in source[line_offset..].char_indices() {
        if seen == units {
            return line_offset + index;
        }
        seen += character.len_utf16();
        assert!(seen <= units, "request splits UTF-16 character");
    }
    assert_eq!(seen, units);
    source.len()
}
