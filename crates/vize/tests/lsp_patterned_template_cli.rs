#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

const SOURCE: &str = r#"<script setup lang="ts">
const result = {} as { kind: 'ok'; value: number } | { kind: 'err'; message: string }
</script>
<template v-match="result">
  <p v-when="{ kind: 'ok', const value }">{{ value.toUpperCase() }}</p>
  <p v-when="{ kind: 'err', const message }">{{ message }}</p>
</template>"#;

#[test]
fn native_patterns_report_authored_errors_hover_definition_and_unsaved_repairs() {
    let mut fixture = Fixture::new(SOURCE, true);
    let diagnostics = fixture.open(SOURCE);
    assert_eq!(diagnostics.as_array().unwrap().len(), 1, "{diagnostics:#}");
    assert_eq!(diagnostics[0]["code"], 2339, "{diagnostics:#}");
    assert_eq!(diagnostics[0]["severity"], 1);
    assert_eq!(
        diagnostics[0]["range"]["start"],
        position(SOURCE, "toUpperCase")
    );

    let hover = fixture.request("textDocument/hover", SOURCE, "value.toUpperCase");
    assert_eq!(
        hover["contents"],
        json!({"kind": "markdown", "value": "```typescript\nconst value: number\n```"}),
        "{hover:#}"
    );
    let definition = fixture.request("textDocument/definition", SOURCE, "value.toUpperCase");
    assert_eq!(definition["uri"], fixture.uri);
    let declaration = position(SOURCE, "value }");
    assert_eq!(definition["range"]["start"], declaration, "{definition:#}");
    assert_eq!(
        definition["range"]["end"]["character"].as_u64(),
        declaration["character"].as_u64().map(|n| n + 5)
    );

    let repaired = SOURCE.replace("toUpperCase", "toFixed");
    assert_eq!(fixture.change(&repaired, 2), json!([]));
    let incomplete = repaired.replace(
        "  <p v-when=\"{ kind: 'err', const message }\">{{ message }}</p>\n",
        "",
    );
    let missing = fixture.change(&incomplete, 3);
    assert_eq!(missing.as_array().unwrap().len(), 1, "{missing:#}");
    assert_eq!(missing[0]["code"], 2322);
    assert_eq!(missing[0]["severity"], 1);
    assert_eq!(
        missing[0]["range"]["start"],
        position(&incomplete, "result\">")
    );
    assert_eq!(fixture.change(&repaired, 4), json!([]));
    fixture.shutdown();
}

#[test]
fn native_patterns_keep_unreachable_warnings_separate_from_type_errors() {
    let source = r#"<script setup lang="ts">
const state = 'a' as 'a' | 'b'
const count: number = 'wrong'
</script>
<template v-match="state"><p v-when="'a'"/><p v-when="'a'"/><p v-when="'b'"/></template>"#;
    let mut fixture = Fixture::new(source, true);
    let diagnostics = fixture.open(source);
    let diagnostics = diagnostics.as_array().unwrap();
    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
    assert!(
        diagnostics
            .iter()
            .any(|d| d["severity"] == 1 && d["range"]["start"]["line"] == 2)
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d["severity"] == 2 && d["range"]["start"]["line"] == 4)
    );
    let repaired = source.replace("'wrong'", "1");
    let warnings = fixture.change(&repaired, 2);
    assert_eq!(warnings.as_array().unwrap().len(), 1, "{warnings:#}");
    assert_eq!(warnings[0]["severity"], 2);
    fixture.shutdown();
}

#[test]
fn native_patterns_opt_out_and_malformed_syntax_never_report_clean() {
    let valid = SOURCE.replace("toUpperCase", "toFixed");
    let mut disabled = Fixture::new(&valid, false);
    let diagnostics = disabled.open(&valid);
    assert_eq!(diagnostics.as_array().unwrap().len(), 1, "{diagnostics:#}");
    assert_eq!(diagnostics[0]["code"], "V_MATCH_SYNTAX");
    assert_eq!(
        diagnostics[0]["message"],
        "`v-match` / `v-when` patterned templates require `experimentals.patternedTemplate`."
    );
    disabled.shutdown();

    let malformed = valid.replace("const value", "const");
    let mut enabled = Fixture::new(&malformed, true);
    let diagnostics = enabled.open(&malformed);
    assert!(
        diagnostics
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "patterned-template" && d["severity"] == 1),
        "{diagnostics:#}"
    );
    assert_eq!(enabled.change(&valid, 2), Value::Array(vec![]));
    enabled.shutdown();
}

#[test]
fn native_pattern_diagnostic_ranges_count_utf16_after_astral_characters() {
    let source = SOURCE.replace("{{ value.", "{{ '\u{1f600}' }}{{ value.");
    let mut fixture = Fixture::new(&source, true);
    let diagnostics = fixture.open(&source);
    assert_eq!(diagnostics.as_array().unwrap().len(), 1, "{diagnostics:#}");
    assert_eq!(diagnostics[0]["code"], 2339);
    assert_eq!(
        diagnostics[0]["range"]["start"],
        position(&source, "toUpperCase")
    );
    let repaired = source.replace("toUpperCase", "toFixed");
    assert_eq!(fixture.change(&repaired, 2), json!([]));
    fixture.shutdown();
}

#[test]
fn native_pattern_guard_comments_keep_hover_definition_and_unsaved_updates() {
    for newline in [
        "\n",
        "\r",
        "\r\n",
        "\u{2028}",
        "\u{2029}",
        "&#10;",
        "&#13;&#10;",
    ] {
        check_guard_comment(newline);
    }
}

fn check_guard_comment(newline: &str) {
    let source = format!(
        "<script setup lang=\"ts\">\r\nconst state = undefined as string | undefined;\r\n</script>\r\n<template v-match=\"state\">\r\n<p v-when=\"const text if (text !== undefined // guard ){newline})\">{{{{ '\u{1f600}' }}}}{{{{ text.toFixed() }}}}</p>\r\n<p v-when=\"_\"/>\r\n</template>"
    );
    let source = source.as_str();
    let mut fixture = Fixture::new_with_vue_and_patterns(source);
    let diagnostics = fixture.open(source);
    assert_eq!(diagnostics.as_array().unwrap().len(), 1, "{diagnostics:#}");
    assert_eq!(diagnostics[0]["code"], 2551);
    assert_eq!(
        diagnostics[0]["range"]["start"],
        position(source, "toFixed"),
        "{newline:?}: {diagnostics:#}"
    );
    let hover = fixture.request("textDocument/hover", source, "text.toFixed");
    assert_eq!(hover["range"]["start"], position(source, "text.toFixed"));
    assert_eq!(
        hover["contents"],
        json!({"kind": "markdown", "value": "```typescript\nconst text: string\n```"}),
        "{hover:#}"
    );
    let definition = fixture.request("textDocument/definition", source, "text.toFixed");
    assert_eq!(definition["uri"], fixture.uri);
    assert_eq!(definition["range"]["start"], position(source, "text if"));
    let repaired = source.replace("toFixed", "toUpperCase");
    assert_eq!(fixture.change(&repaired, 2), json!([]));
    let malformed = repaired.replace(&format!("// guard ){newline})"), "// guard )");
    let diagnostics = fixture.change(&malformed, 3);
    assert!(
        diagnostics.as_array().unwrap().iter().any(|diagnostic| {
            diagnostic["code"] == "patterned-template" && diagnostic["severity"] == 1
        }),
        "{diagnostics:#}"
    );
    assert_eq!(fixture.change(&repaired, 4), json!([]));
    fixture.shutdown();
}
