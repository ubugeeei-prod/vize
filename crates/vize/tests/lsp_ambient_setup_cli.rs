use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

#[test]
fn ambient_global_types_keep_authored_navigation_and_unsaved_diagnostics() {
    for newline in ["\n", "\r\n"] {
        for script in [
            "node.focus();\ndeclare const node: HTMLElement;",
            "const prefix = '\u{1f600}'; node.focus(); declare const node: HTMLElement; const after = node.tagName;",
        ] {
            let source = format!("<script setup lang=\"ts\">\n{script}\n</script>\n<template><div>{{{{ '\u{1f600}' }}}}{{{{ node.tagName }}}}</div></template>\n")
            .replace('\n', newline);
            let mut fixture = Fixture::new(&source, false);
            assert_eq!(fixture.open(&source), json!([]));
            assert_navigation(&mut fixture, &source, "HTMLElement");

            let broken = source.replace("{{ node.tagName }}", "{{ node.toFixed() }}");
            let diagnostics = fixture.change(&broken, 2);
            assert_eq!(
                diagnostics,
                json!([{
                    "code": 2339,
                    "message": "Property 'toFixed' does not exist on type 'HTMLElement'.",
                    "range": token_range(&broken, "toFixed", "toFixed"),
                    "severity": 1,
                    "source": "vize/types"
                }])
            );
            assert_eq!(fixture.change(&source, 3), json!([]));

            let changed_type = source
                .replace("HTMLElement", "Date")
                .replace("node.focus()", "node.getFullYear()")
                .replace("node.tagName", "node.getFullYear()");
            assert_eq!(fixture.change(&changed_type, 4), json!([]));
            assert_navigation(&mut fixture, &changed_type, "Date");
            assert_eq!(fixture.change(&source, 5), json!([]));
            assert_navigation(&mut fixture, &source, "HTMLElement");
            fixture.shutdown();
        }
    }
}

fn assert_navigation(fixture: &mut Fixture, source: &str, ty: &str) {
    let usage = if ty == "Date" {
        "node.getFullYear() }}</div>"
    } else {
        "node.tagName"
    };
    let hover = fixture.request("textDocument/hover", source, usage);
    assert_eq!(
        hover,
        json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```typescript\nconst node: {ty}\n```")
            },
            "range": token_range(source, usage, "node")
        })
    );
    let definition = fixture.request("textDocument/definition", source, usage);
    assert_eq!(
        definition,
        json!({ "uri": fixture.uri, "range": token_range(source, "node:", "node") })
    );
}

fn token_range(source: &str, needle: &str, token: &str) -> Value {
    let start = position(source, needle);
    let end = json!({
        "line": start["line"],
        "character": start["character"].as_u64().unwrap() + token.encode_utf16().count() as u64
    });
    json!({ "start": start, "end": end })
}
