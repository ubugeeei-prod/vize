//! Full original files and pinned official packets for ref member operands.

use serde_json::Value;
use vize_patina::{HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter, Locale};

const RULE: &str = "script/no-ref-as-operand";
const ORACLE: &str = include_str!("fixtures/ref-member-access/oracle.json");

fn byte_offset(source: &str, line: &Value, column: &Value) -> u32 {
    let line_index = line.as_u64().unwrap() as usize - 1;
    let column_index = column.as_u64().unwrap() as usize - 1;
    let prefix: usize = source
        .split_inclusive('\n')
        .take(line_index)
        .map(str::len)
        .sum();
    let mut utf16 = 0;
    for (offset, character) in source.get(prefix..).unwrap().char_indices() {
        if utf16 == column_index {
            return (prefix + offset) as u32;
        }
        utf16 += character.len_utf16();
    }
    assert_eq!(utf16, column_index);
    source.len() as u32
}

#[test]
fn original_and_control_member_operands_retain_complete_native_findings() {
    let fixture: Value = serde_json::from_str(ORACLE).unwrap();
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    let cases = fixture["captures"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let id = case["id"].as_str().unwrap();
        let filename = if id.ends_with(".vue") {
            id.to_owned()
        } else {
            format!("{id}.vue")
        };
        let packet = &case["packets"][0];
        let diagnostics = packet["messages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|message| {
                assert_eq!(message["ruleId"], "vue/no-ref-as-operand");
                let start = byte_offset(source, &message["line"], &message["column"]);
                let end = byte_offset(source, &message["endLine"], &message["endColumn"]);
                let name = source.get(start as usize..end as usize).unwrap();
                LintDiagnostic::error(
                    RULE,
                    format!("'{name}' is a ref and must be unwrapped with `.value` here."),
                    start,
                    end,
                )
                .with_label("ref used directly as an operand", start, end)
                .with_help("A ref holds its value behind `.value`. Read it as `<name>.value` in this position; passing the ref itself (e.g. to `watch`) does not need `.value`.")
            })
            .collect();
        let expected = LintResult {
            filename: filename.as_str().into(),
            diagnostics,
            error_count: packet["errorCount"].as_u64().unwrap() as usize,
            warning_count: packet["warningCount"].as_u64().unwrap() as usize,
        };
        let actual = linter.lint_sfc(source, &filename);
        assert_eq!(
            format!("{actual:#?}"),
            format!("{expected:#?}"),
            "{filename}"
        );
    }
}
