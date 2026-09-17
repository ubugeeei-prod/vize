use serde_json::json;

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

#[test]
fn native_unicode_diagnostics_preserve_messages_and_survive_unsaved_repairs() {
    // TS2322 includes the literal, placing byte 100 inside a three-byte character.
    let literal = format!("x{}", "\u{3042}".repeat(40));
    let message = format!("Type '\"{literal}\"' is not assignable to type '\"ok\"'.");
    assert!(!message.is_char_boundary(100));
    let source = format!(
        "<script setup lang=\"ts\">\nconst label: 'ok' = '{literal}'\n</script>\n<template>{{{{ label }}}}</template>"
    );
    let mut fixture = Fixture::new(&source, false);
    let diagnostics = fixture.open(&source);
    assert_eq!(diagnostics.as_array().unwrap().len(), 1, "{diagnostics:#}");
    assert_eq!(diagnostics[0]["code"], 2322);
    assert_eq!(diagnostics[0]["severity"], 1);
    assert_eq!(diagnostics[0]["message"], message);
    let start = position(&source, "label:");
    let end =
        json!({ "line": start["line"], "character": start["character"].as_u64().unwrap() + 5 });
    assert_eq!(
        diagnostics[0]["range"],
        json!({ "start": start, "end": end })
    );

    let repaired = source.replace(&literal, "ok");
    assert_eq!(fixture.change(&repaired, 2), json!([]));
    let hover = fixture.request("textDocument/hover", &repaired, "label }}");
    assert_eq!(
        hover["contents"],
        json!({
            "kind": "markdown", "value": "```typescript\nconst label: \"ok\"\n```"
        })
    );
    // Re-introducing the same error must publish it again in the same process.
    assert_eq!(fixture.change(&source, 3), diagnostics);
    assert_eq!(fixture.change(&repaired, 4), json!([]));
    fixture.shutdown();
}
