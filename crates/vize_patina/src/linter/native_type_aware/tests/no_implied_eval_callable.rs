//! #13000: no-implied-eval accepts every whole type with a call signature.
use super::{corsa_available, failure_context, lint_sfc_with_corsa};
use crate::{LintPreset, Linter, Severity};
use serde_json::json;
use std::path::Path;

const RULE: &str = "type/no-implied-eval";
const SOURCE: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/no-implied-eval-callable-13000/src/Timers.vue.txt"
);

#[test]
fn callable_composites_match_typescript_eslint() {
    if !corsa_available() {
        return;
    }

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("no-implied-eval-callable-13000-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(
        project.path().join("tsconfig.json"),
        include_str!(
            "../../../../../../tests/_fixtures/differential/linter/no-implied-eval-callable-13000/tsconfig.json"
        ),
    )
    .expect("tsconfig");
    let component = source_dir.join("Timers.vue");
    std::fs::write(&component, SOURCE).expect("component");

    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_rule_severity_overrides(vec![(RULE.into(), Severity::Error)]);
    let result = lint_sfc_with_corsa(
        &linter,
        SOURCE,
        component.to_str().expect("UTF-8 component path"),
    );
    let start = u32::try_from(SOURCE.find("\"plain()\"").expect("string handler")).unwrap();
    let end = start + u32::try_from("\"plain()\"".len()).unwrap();
    let packets = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            json!({
                "rule_name": diagnostic.rule_name,
                "severity": diagnostic.severity,
                "message": diagnostic.message.as_str(),
                "start": diagnostic.start,
                "end": diagnostic.end,
                "help": diagnostic.help.as_deref(),
                "labels": diagnostic.labels.iter().map(|label| json!({
                    "message": label.message.as_str(),
                    "start": label.start,
                    "end": label.end,
                })).collect::<Vec<_>>(),
                "fix": diagnostic.fix.as_ref().map(|fix| json!({
                    "message": fix.message.as_str(),
                    "edits": fix.edits.iter().map(|edit| json!({
                        "start": edit.start,
                        "end": edit.end,
                        "new_text": edit.new_text.as_str(),
                    })).collect::<Vec<_>>(),
                })),
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(
        packets,
        [json!({
            "rule_name": RULE,
            "severity": "error",
            "message": "Implied eval. Consider passing a function.",
            "start": start,
            "end": end,
            "help": null,
            "labels": [],
            "fix": null,
        })],
        "{}",
        failure_context(&result)
    );
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);
}
