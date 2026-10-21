use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter};

fn whole(result: &LintResult) -> Value {
    json!({
        "filename": result.filename,
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "fix": d.fix,
            "labels": d.labels.iter().map(|l| json!({
                "message": l.message, "start": l.start, "end": l.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

#[test]
fn setup_emits_return_checks_preserve_owned_macros_and_sibling_offsets() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/setup-emits-6097/cases.json"
    ))
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 51);
    let rules = corpus["rules"].as_array().unwrap();
    let disabled = rules
        .iter()
        .map(|rule| rule.as_str().unwrap().into())
        .collect::<Vec<_>>();
    for case in cases {
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let diagnostics = case["diagnostics"].as_array().unwrap();
        for preset in [
            LintPreset::HappyPath,
            LintPreset::Essential,
            LintPreset::Ecosystem,
        ] {
            let result = Linter::with_preset(preset)
                .with_help_level(HelpLevel::None)
                .lint_sfc(source, filename);
            assert_eq!(
                whole(&result),
                json!({
                    "filename": filename,
                    "error_count": diagnostics.iter().filter(|d| d["severity"] == "error").count(),
                    "warning_count": diagnostics.iter().filter(|d| d["severity"] == "warning").count(),
                    "diagnostics": diagnostics
                }),
                "{} / {}",
                case["id"],
                preset.as_str()
            );
        }
        for (linter, findings) in [
            (
                Linter::new()
                    .with_help_level(HelpLevel::None)
                    .with_disabled_rules(disabled.clone()),
                case["disabledDiagnostics"].clone(),
            ),
            (Linter::with_preset(LintPreset::Incremental), json!([])),
        ] {
            let diagnostics = findings.as_array().unwrap();
            assert_eq!(
                whole(&linter.lint_sfc(source, filename)),
                json!({
                    "filename": filename,
                    "error_count": diagnostics.iter().filter(|d| d["severity"] == "error").count(),
                    "warning_count": diagnostics.iter().filter(|d| d["severity"] == "warning").count(),
                    "diagnostics": diagnostics
                }),
                "{}",
                case["id"]
            );
        }
    }
}

#[test]
fn standalone_and_plain_scripts_do_not_gain_compiler_macro_ownership() {
    use vize_patina::rules::script::{
        ReturnInEmitsValidator, ScriptLintResult, ScriptLinter, ScriptRule,
    };
    let source = "const emit = defineEmits({ submit(payload) { void payload } })";
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(ReturnInEmitsValidator));
    assert!(linter.lint(source, 0).diagnostics.is_empty());
    let mut result = ScriptLintResult::default();
    ReturnInEmitsValidator.check(source, 17, &mut result);
    assert!(result.diagnostics.is_empty());
}
