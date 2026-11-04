use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, process::Command};

const FIELD_BODY: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/slot-conditional-arms-10822/FieldBody.vue.txt"
);
const CASES: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/slot-typeof-presence-10822/cases.json"
);

const CONDITIONAL_CASES: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/slot-conditional-arms-10822/cases.json"
);
const CONDITIONAL_SUCCESSORS: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/slot-conditional-arms-10822/semantic-successors.json"
);

#[test]
fn original_and_boundaries_keep_complete_cli_json() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(
        temp.path().join("vite.config.mjs"),
        vite_config::source(
            r#"{"linter":{"preset":"incremental","rules":{"script/require-slots-as-functions":"error"}}}"#,
        ),
    )
    .unwrap();
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let successor: Value = serde_json::from_str(CONDITIONAL_SUCCESSORS).unwrap();
    assert_eq!(
        successor["schema"],
        "vize.slot-conditional-arm-semantic-successors.v1"
    );
    assert_eq!(
        successor["feature"],
        "immediate-authenticated-conditional-arms-v1"
    );
    let original = &successor["originalCli"];
    assert_eq!(original["id"], "original-field-body");
    assert_eq!(
        original["inputSha256"],
        "549dd10935821983608154160721b6a600b8e4f74b74a60902872df81e934598"
    );
    let before: Value = serde_json::from_str(
        original["beforeProducerLine"]
            .as_str()
            .unwrap()
            .strip_prefix("vize-slot-typeof-cli ")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(before["source"], FIELD_BODY);
    assert_eq!(before["status"], 1);
    assert_eq!(original["currentNativeSpans"], json!([]));
    println!("vize-slot-conditional-original-successor complete={original}");
    let mut rows = vec![(
        "original-field-body",
        FIELD_BODY.to_owned(),
        original["currentNativeSpans"].as_array().unwrap().clone(),
    )];
    for id in [
        "direct-typeof",
        "known-function-alias",
        "nested-member-unicode",
    ] {
        let row = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap();
        rows.push((
            id,
            format!("<script>\n{}\n</script>\n", row["script"].as_str().unwrap()),
            row["nativeSpans"].as_array().unwrap().clone(),
        ));
    }
    let arms: Value = serde_json::from_str(CONDITIONAL_CASES).unwrap();
    for id in [
        "conditional-arm-consequent",
        "conditional-arm-parenthesized-both",
        "conditional-arm-nested-array",
        "conditional-arm-report-order",
    ] {
        let row = arms["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap();
        rows.push((
            id,
            format!("<script>\n{}\n</script>\n", row["script"].as_str().unwrap()),
            row["nativeSpans"].as_array().unwrap().clone(),
        ));
    }
    let mut captured = Vec::new();
    for (id, source, spans) in rows {
        let filename = format!("src/{id}.vue");
        let path = temp.path().join(&filename);
        fs::write(&path, &source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(temp.path())
            .args([
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                &filename,
            ])
            .output()
            .unwrap();
        let after = fs::read(&path).unwrap();
        println!(
            "vize-slot-typeof-cli {}",
            json!({"id":id,"source":source,"status":output.status.code(),
                "stdoutBytes":output.stdout,"stderrBytes":output.stderr,"fileAfter":after}),
        );
        captured.push((id, filename, source, spans, output, after));
    }
    for (id, filename, source, spans, output, after) in captured {
        assert_eq!(after, source.as_bytes(), "{id}");
        assert!(output.stderr.is_empty(), "{id}: {output:?}");
        assert_eq!(
            output.status.code(),
            Some(i32::from(!spans.is_empty())),
            "{id}: {output:?}"
        );
        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
        let messages = spans
            .iter()
            .map(|span| {
                let needle = span["needle"].as_str().unwrap();
                assert_eq!(source.matches(needle).count(), 1);
                let start = source.find(needle).unwrap() + span["shift"].as_u64().unwrap() as usize;
                let end = start + span["length"].as_u64().unwrap() as usize;
                let position = |offset: usize| {
                    let before = &source[..offset];
                    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
                    // Patina CLI JSON uses Unicode scalar columns; ESLint uses UTF-16 separately.
                    let column = before.rsplit('\n').next().unwrap().chars().count() + 1;
                    (line, column)
                };
                let (line, column) = position(start);
                let (end_line, end_column) = position(end);
                json!({"ruleId":"script/require-slots-as-functions",
                    "ruleDocsPath":"docs/content/rules/reference/script-require-slots-as-functions.md","severity":2,
                    "message":"[vize:script/require-slots-as-functions] `$slots` properties are functions in Vue 3 and must be called.",
                    "line":line,"column":column,"endLine":end_line,"endColumn":end_column})
            })
            .collect::<Vec<_>>();
        let expected = json!([{"file":filename,"errorCount":messages.len(),"warningCount":0,"messages":messages}]);
        assert_eq!(actual, expected, "{id}: {output:?}");
    }
}
