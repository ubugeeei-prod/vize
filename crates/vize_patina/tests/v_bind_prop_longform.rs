//! Whole public-rule vectors derived from the pinned, independent ESLint corpus.
use serde_json::{Value, json};
use vize_l0::{String, config::ConfigLintRuleOptions};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter};

const RULE: &str = "vue/v-bind-style";
const FILENAME: &str = "VBindProp.vue";
const CORPUS: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/cases.json"
);

fn source(file: &str) -> &'static str {
    match file {
        "Original.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Original.vue.txt"
        ),
        "Explicit.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Explicit.vue.txt"
        ),
        "Dynamic.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Dynamic.vue.txt"
        ),
        "Controls.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Controls.vue.txt"
        ),
        "Colon.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Colon.vue.txt"
        ),
        _ => panic!("unknown pinned v-bind input"),
    }
}
fn linter(row: &Value) -> Linter {
    let options: ConfigLintRuleOptions = serde_json::from_value(if row["style"].is_null() {
        json!({})
    } else {
        json!({ RULE: row["style"] })
    })
    .unwrap();
    Linter::with_preset(LintPreset::Opinionated)
        .with_type_aware_lint(false)
        .with_help_level(HelpLevel::Full)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_config_rule_options(&options)
}
fn packet(result: &LintResult) -> Value {
    json!({
        "filename": result.filename, "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "fix": d.fix,
            "labels": d.labels.iter().map(|label| json!({
                "message": label.message, "start": label.start, "end": label.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}
fn byte_offset(source: &str, utf16: usize) -> usize {
    let mut seen = 0;
    for (offset, character) in source.char_indices() {
        if seen == utf16 {
            return offset;
        }
        seen += character.len_utf16();
        assert!(
            seen <= utf16,
            "reference boundary must not split a surrogate pair"
        );
    }
    assert_eq!(seen, utf16);
    source.len()
}
fn location(source: &str, finding: &Value, line_key: &str, column_key: &str) -> usize {
    let line = finding[line_key].as_u64().unwrap() as usize;
    let column = finding[column_key].as_u64().unwrap() as usize;
    let preceding = source
        .split('\n')
        .take(line - 1)
        .map(|line| line.encode_utf16().count() + 1)
        .sum::<usize>();
    byte_offset(source, preceding + column - 1)
}
fn expected(source: &str, row: &Value) -> Value {
    let diagnostics = row["reference"]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            let (message, help, fix_message) = match finding["messageId"].as_str().unwrap() {
                "expectedLonghandForProp" => (
                    "Expected 'v-bind:' instead of '.'.",
                    "Use `v-bind:attr.prop=\"value\"` instead of `.attr=\"value\"`",
                    "Use longform syntax",
                ),
                "expectedLonghand" => (
                    "Prefer `v-bind:` over shorthand `:`",
                    "Use `v-bind:attr=\"value\"` instead of `:attr=\"value\"`",
                    "Use longform syntax",
                ),
                "unexpectedLonghand" => (
                    "Prefer shorthand `:` over `v-bind:`",
                    "Use `:attr=\"value\"` instead of `v-bind:attr=\"value\"`",
                    "Use shorthand syntax",
                ),
                _ => panic!("unknown independent finding"),
            };
            let range = finding["fix"]["range"].as_array().unwrap();
            let [start, end] = range.as_slice() else {
                panic!("complete reference range")
            };
            let offset = location(source, finding, "line", "column");
            // The legacy ':' / 'v-bind:' edit shape is an existing public contract;
            // only the new '.' edits match the reference's merged edit directly.
            let (start, end, text) = match finding["messageId"].as_str().unwrap() {
                "expectedLonghandForProp" => (
                    byte_offset(source, start.as_u64().unwrap() as usize),
                    byte_offset(source, end.as_u64().unwrap() as usize),
                    finding["fix"]["text"].as_str().unwrap(),
                ),
                "unexpectedLonghand" => (offset, offset + 7, ":"),
                "expectedLonghand" => (offset, offset + 1, "v-bind:"),
                _ => unreachable!(),
            };
            json!({"rule_name":RULE,"severity":"warning","message":message,
            "start":location(source,finding,"line","column"),
            "end":location(source,finding,"endLine","endColumn"),"help":help,"labels":[],
            "fix":{"message":fix_message,"edits":[{
                    "start":start,
                    "end":end,
                    "new_text":text
            }]}})
        })
        .collect::<Vec<_>>();
    json!({"filename":FILENAME,"error_count":0,"warning_count":diagnostics.len(),"diagnostics":diagnostics})
}
fn clean() -> Value {
    json!({"filename":FILENAME,"error_count":0,"warning_count":0,"diagnostics":[]})
}

#[test]
fn all_nine_reference_cases_preserve_whole_diagnostics_fixes_and_fixed_points() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 9);
    let mut findings = 0;
    for row in cases {
        let source = source(row["file"].as_str().unwrap());
        let linter = linter(row);
        let actual = linter.lint_sfc(source, FILENAME);
        let whole = packet(&actual);
        assert_eq!(whole, expected(source, row), "{}", row["id"]);
        findings += actual.diagnostics.len();
        let mut edits = actual
            .diagnostics
            .iter()
            .flat_map(|d| d.fix.as_ref().unwrap().edits.iter())
            .collect::<Vec<_>>();
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
        let mut fixed = String::from(source);
        for edit in edits {
            fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
        }
        assert_eq!(
            fixed.as_str(),
            row["reference"]["fixResult"]["output"].as_str().unwrap(),
            "{}",
            row["id"]
        );
        for _ in 0..2 {
            assert_eq!(
                packet(&linter.lint_sfc(&fixed, FILENAME)),
                clean(),
                "{}",
                row["id"]
            );
        }
        assert_eq!(
            packet(
                &linter
                    .with_enabled_rules(Some(vec![]))
                    .lint_sfc(source, FILENAME)
            ),
            clean()
        );
    }
    assert_eq!(findings, 9);
}
