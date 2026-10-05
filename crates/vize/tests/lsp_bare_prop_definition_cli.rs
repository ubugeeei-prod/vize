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

const IMPORTED: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/bare-prop-definition/StatusBox.vue.txt"
);
const LOCAL: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/bare-prop-definition/StatusBoxLocal.vue.txt"
);
const TYPES: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/bare-prop-definition/status-box.types.ts.txt"
);

#[test]
fn original_bare_and_qualified_props_resolve_to_the_complete_same_interface_member() {
    for newline in ["\n", "\r\n"] {
        for (source, line, imported) in [(IMPORTED, 7, true), (LOCAL, 9, false)] {
            let source = source.replace('\n', newline);
            let types = TYPES.replace('\n', newline);
            let mut fixture = Fixture::new_with_vue(&source);
            let type_uri = fixture.write_file("status-box.types.ts", &types);
            assert_eq!(fixture.open(&source), json!([]));
            let expected = if imported {
                json!({ "uri": type_uri, "range": { "start": { "line": 1, "character": 2 }, "end": { "line": 1, "character": 8 } } })
            } else {
                json!({ "uri": fixture.uri, "range": { "start": { "line": 2, "character": 2 }, "end": { "line": 2, "character": 8 } } })
            };
            for character in [31, 61] {
                let offset = byte_offset(&source, line, character);
                assert_eq!(
                    position(&source, &source[offset..]),
                    json!({ "line": line, "character": character })
                );
                assert_eq!(definition_at(&mut fixture, &source, offset), expected);
            }
            assert_prop_results(&mut fixture, &source, &expected);
            fixture.shutdown();
        }
    }
}

#[test]
fn actual_prop_owners_survive_utf16_unsaved_defaults_and_same_name_lexical_shadows() {
    for newline in ["\n", "\r\n"] {
        for (source, imported) in [(IMPORTED, true), (LOCAL, false)] {
            let source = source
                .replace("  status?:", "  /* 😀 */ status?:")
                .replace(
                    "</script>",
                    "import type { ForeignProps } from \"./foreign.types\";\nfunction label(status: string) { return status; }\n</script>",
                )
                .replace(
                    "</template>",
                    "  <div v-for=\"status in ['default']\">{{ status }}</div>\n</template>",
                );
            let source = format!("<!-- 😀 -->\n{source}").replace('\n', newline);
            let types = TYPES
                .replace("  status?:", "  /* 😀 */ status?:")
                .replace('\n', newline);
            let mut fixture = Fixture::new_with_vue(&source);
            let type_uri = fixture.write_file("status-box.types.ts", &types);
            let foreign = "export interface ForeignProps { status?: number; }\n";
            let foreign_uri = fixture.write_file("foreign.types.ts", foreign);
            assert_eq!(fixture.open(&source), json!([]));
            let expected = if imported {
                json!({ "uri": type_uri, "range": token_range(&types, types.find("status?:").unwrap(), 6) })
            } else {
                json!({ "uri": fixture.uri, "range": token_range(&source, source.find("status?:").unwrap(), 6) })
            };
            assert_ne!(expected["uri"], foreign_uri);
            assert_prop_results(&mut fixture, &source, &expected);
            assert_shadow_results(&mut fixture, &source);
            let changed = format!(
                "<!-- 😀 current -->{newline}{}",
                source.replace("status: \"default\"", "status: \"error\"")
            );
            assert_eq!(fixture.change(&changed, 2), json!([]));
            let changed_expected = if imported {
                expected.clone()
            } else {
                json!({ "uri": fixture.uri, "range": token_range(&changed, changed.find("status?:").unwrap(), 6) })
            };
            assert_prop_results(&mut fixture, &changed, &changed_expected);
            assert_shadow_results(&mut fixture, &changed);
            assert_eq!(fixture.change(&source, 3), json!([]));
            assert_prop_results(&mut fixture, &source, &expected);
            assert_shadow_results(&mut fixture, &source);
            fixture.shutdown();
        }
    }
}

fn assert_prop_results(fixture: &mut Fixture, source: &str, expected: &Value) {
    let bare = source.find("${status}").unwrap() + 2;
    let qualified = source.find("props.status\"").unwrap() + "props.".len();
    for offset in [bare, qualified] {
        assert_eq!(definition_at(fixture, source, offset + 1), *expected);
    }
    assert_eq!(
        fixture.request("textDocument/hover", source, &source[bare + 1..]),
        json!({
            "contents": { "kind": "markdown", "value": "```typescript\nconst status: \"default\" | \"error\"\n```" },
            "range": token_range(source, bare, 6)
        })
    );
}

fn assert_shadow_results(fixture: &mut Fixture, source: &str) {
    let parameter = source.find("status: string").unwrap();
    let parameter_use = source.find("status;").unwrap();
    let for_binding = source.find("v-for=\"status in").unwrap() + "v-for=\"".len();
    let for_use = source.find("{{ status }}").unwrap() + 3;
    for (offset, declaration) in [
        (parameter, parameter),
        (parameter_use, parameter),
        (for_binding, for_binding),
        (for_use, for_binding),
    ] {
        assert_eq!(
            definition_at(fixture, source, offset + 1),
            json!({
                "uri": fixture.uri, "range": token_range(source, declaration, 6)
            })
        );
    }
}

fn token_range(source: &str, offset: usize, len: usize) -> Value {
    json!({ "start": position(source, &source[offset..]), "end": position(source, &source[offset + len..]) })
}

fn definition_at(fixture: &mut Fixture, source: &str, offset: usize) -> Value {
    fixture.request("textDocument/definition", source, &source[offset..])
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
