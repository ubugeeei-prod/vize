//! Existing script-setup facts close the six original n8n shadowing misses.
use serde_json::{Value, json};
use std::sync::LazyLock;
use vize_patina::{HelpLevel, LintResult, Linter, Locale};

const RULE: &str = "vue/no-template-shadow";
static CORPUS: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/linter/template-script-shadow-10822/cases.json"
    ))
    .unwrap()
});

pub(super) fn successor_for(historical: &Value) -> Option<&'static Value> {
    let corpus: &'static Value = &CORPUS;
    let successors = corpus["successors"].as_array().unwrap();
    assert_eq!(successors.len(), 3);
    let current = successors
        .iter()
        .find(|case| case["id"] == historical["id"])?;
    assert_eq!(current["source"], historical["source"]);
    assert_eq!(current["historicalNative"], historical["expectedNative"]);
    assert_eq!(current["historicalNative"]["warning_count"], 0);
    assert_eq!(current["historicalNative"]["diagnostics"], json!([]));
    Some(&current["expectedNative"])
}

fn complete(result: &LintResult) -> Value {
    json!({
        "filename": result.filename.as_str(),
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity,
            "message": d.message.as_str(), "start": d.start, "end": d.end,
            "help": d.help.as_ref().map(|help| help.as_str()),
            "labels": d.labels.iter().map(|label| json!({
                "message": label.message.as_str(), "start": label.start, "end": label.end
            })).collect::<Vec<_>>(),
            "fix": d.fix
        })).collect::<Vec<_>>()
    })
}

fn point(source: &str, offset: u32) -> (u32, u32) {
    let mut chars = source.get(..offset as usize).unwrap().chars().peekable();
    let (mut line, mut column) = (1, 1);
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            line += 1;
            column = 1;
        } else if matches!(ch, '\n' | '\u{2028}' | '\u{2029}') {
            line += 1;
            column = 1;
        } else {
            column += ch.len_utf16() as u32;
        }
    }
    (line, column)
}

#[test]
fn original_n8n_and_setup_controls_match_complete_native_packets_and_reference_ranges() {
    assert_eq!(CORPUS["versions"]["eslint-plugin-vue"], "10.9.2");
    assert_eq!(
        CORPUS["originalMatrix"]["budget"]["maxFalsePositiveCount"],
        0
    );
    assert_eq!(
        CORPUS["originalMatrix"]["budget"]["maxFalseNegativeCount"],
        0
    );
    assert_eq!(
        CORPUS["originalMatrix"]["falseNegatives"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(CORPUS["originalMatrix"]["falsePositives"], json!([]));
    let linter = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    for section in ["originals", "successors", "controls"] {
        for case in CORPUS[section].as_array().unwrap() {
            let authored = match (section, case["path"].as_str()) {
                ("originals", Some("originals/DropdownMenuItem.vue.txt")) => Some(include_str!(
                    "../../../tests/_fixtures/differential/linter/template-script-shadow-10822/originals/DropdownMenuItem.vue.txt"
                )),
                ("originals", Some("originals/RunDataJson.vue.txt")) => Some(include_str!(
                    "../../../tests/_fixtures/differential/linter/template-script-shadow-10822/originals/RunDataJson.vue.txt"
                )),
                ("originals", _) => panic!("unprovided original source"),
                _ => None,
            };
            let source = authored.unwrap_or_else(|| case["source"].as_str().unwrap());
            assert_eq!(source, case["source"].as_str().unwrap());
            let filename = case["filename"].as_str().unwrap();
            let result = linter.lint_sfc(source, filename);
            assert_eq!(complete(&result), case["expectedNative"], "{}", case["id"]);
            assert_eq!(
                complete(&linter.lint_sfc(source, filename)),
                case["expectedNative"],
                "unchanged re-query {}",
                case["id"]
            );
            let reference = case["reference"].as_array().unwrap();
            assert_eq!(result.diagnostics.len(), reference.len());
            for (diagnostic, expected) in result.diagnostics.iter().zip(reference) {
                let (line, column) = point(source, diagnostic.start);
                let (end_line, end_column) = point(source, diagnostic.end);
                assert_eq!(
                    json!({"ruleId": diagnostic.rule_name, "severity": 1,
                        "line": line, "column": column,
                        "endLine": end_line, "endColumn": end_column}),
                    json!({"ruleId": expected["ruleId"], "severity": expected["severity"],
                        "line": expected["line"], "column": expected["column"],
                        "endLine": expected["endLine"], "endColumn": expected["endColumn"]}),
                    "{}",
                    case["id"]
                );
            }
        }
    }
}
