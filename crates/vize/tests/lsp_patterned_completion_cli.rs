use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::Fixture;

const SOURCE: &str = r#"<script setup lang="ts">
const rows = 'outer'
const result = {} as { kind: 'ok'; rows: number[]; marker: number } | { kind: 'err'; message: string }
</script>
<template v-match="result">
  <template v-when="{ kind: 'ok', const rows, marker: const armOnly } if (rows.length > 0)">
    <p>{{ rows.length }}{{ armOnly }}</p>
    <p v-for="rows in rows">{{ rows.toFixed() }}</p>
  </template>
  <p v-when="{ kind: 'err', const message }">{{ message.toUpperCase() }}</p>
  <p v-when="_">{{ rows.toUpperCase() }}</p>
</template>"#;

fn complete(fixture: &mut Fixture, source: &str, needle: &str) -> Vec<Value> {
    let result = fixture.request("textDocument/completion", source, needle);
    result
        .as_array()
        .or_else(|| result["items"].as_array())
        .unwrap_or_else(|| panic!("missing completion: {result:#}"))
        .clone()
}

fn labels(items: &[Value], include: &[&str], exclude: &[&str]) {
    let labels: Vec<_> = items
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect();
    for (names, expected) in [(include, 1), (exclude, 0)] {
        for name in names {
            assert_eq!(
                labels.iter().filter(|label| *label == name).count(),
                expected,
                "completion count for {name}: {labels:?}"
            );
        }
    }
    let helpers: Vec<_> = labels
        .iter()
        .filter(|label| label.starts_with("__vize_match_") || label.starts_with("__VizePatterns"))
        .collect();
    assert_eq!(helpers, Vec::<&&str>::new(), "generated helper candidates");
}

#[test]
fn native_pattern_completions_follow_root_arm_guard_and_loop_visibility() {
    let mut fixture = Fixture::new(SOURCE, true);
    assert_eq!(fixture.open(SOURCE), json!([]));
    let root = complete(&mut fixture, SOURCE, "result\">");
    labels(&root, &["result", "rows"], &["armOnly", "message"]);
    for (needle, kind) in [
        ("rows.length >", "v-when"),
        ("rows.length }}", "v-when"),
        ("rows\">", "v-when"),
        ("rows.toFixed", "v-for"),
    ] {
        let items = complete(&mut fixture, SOURCE, needle);
        labels(&items, &["rows", "armOnly", "result"], &["message"]);
        let rows: Vec<_> = items
            .iter()
            .filter(|item| item["label"] == "rows")
            .collect();
        assert_eq!(rows.len(), 1, "{items:#?}");
        assert_eq!(rows[0]["detail"], format!("Local {kind} binding"));
        assert_eq!(rows[0]["sortText"], "00rows");
    }
    let sibling = complete(&mut fixture, SOURCE, "message.toUpperCase");
    labels(&sibling, &["message", "rows", "result"], &["armOnly"]);
    let fallback = complete(&mut fixture, SOURCE, "rows.toUpperCase");
    labels(&fallback, &["rows", "result"], &["armOnly", "message"]);
    assert_eq!(fixture.change(SOURCE, 2), json!([]));
    fixture.shutdown();
}

#[test]
fn native_pattern_member_completion_preserves_checker_narrowing() {
    let mut fixture = Fixture::new(SOURCE, true);
    assert_eq!(fixture.open(SOURCE), json!([]));
    let array = complete(&mut fixture, SOURCE, "length }}");
    labels(
        &array,
        &["length", "map", "filter"],
        &["toUpperCase", "toFixed", "armOnly"],
    );
    let number = complete(&mut fixture, SOURCE, "toFixed");
    labels(
        &number,
        &["toFixed", "toExponential"],
        &["map", "toUpperCase", "armOnly"],
    );
    let string = complete(&mut fixture, SOURCE, "toUpperCase");
    labels(
        &string,
        &["toUpperCase", "charAt"],
        &["map", "toFixed", "armOnly"],
    );
    let changed = SOURCE
        .replace("rows: number[]", "rows: string[]")
        .replace("rows.toFixed()", "rows.toLowerCase()");
    assert_eq!(fixture.change(&changed, 2), json!([]));
    let string = complete(&mut fixture, &changed, "toLowerCase");
    labels(
        &string,
        &["toLowerCase", "charAt"],
        &["toFixed", "toExponential"],
    );
    fixture.shutdown();
}
