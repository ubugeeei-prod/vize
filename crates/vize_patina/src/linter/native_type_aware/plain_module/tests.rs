//! Deferred plain-module plans preserve source ownership and failure ordering.

use super::super::{RULE_NO_FLOATING_PROMISES, TypeAwareFailure};
use super::{empty_result, prepare_plain_module};
use crate::{LintDiagnostic, LintPreset, Linter, Severity};
use vize_l0::String;

fn configured() -> Linter {
    Linter::with_preset(LintPreset::Opinionated)
        .with_type_aware_lint(true)
        .with_enabled_rules(Some(vec![RULE_NO_FLOATING_PROMISES.into()]))
}

fn assert_owned<T: Send + 'static>(_: &T) {}

#[test]
fn plain_plan_outlives_source_and_preserves_existing_findings_on_failure() {
    let linter = configured();
    let prepared = {
        let source =
            String::from("export async function save() {}\nexport function submit() { save(); }\n");
        let mut initial = empty_result("owned.ts");
        initial.diagnostics.push(LintDiagnostic::warn(
            "script/existing",
            "existing finding",
            0,
            1,
        ));
        initial.warning_count = 1;
        prepare_plain_module(&linter, &source, "owned.ts", initial)
    };
    assert_owned(&prepared);
    assert_eq!(
        prepared.virtual_content(),
        Some(
            "export async function save() {}\nexport function submit() { save();\nfunction __vize_patina_promise_0() { return (save()); }\n }\n"
        ),
    );
    let result = prepared.finish(
        &linter,
        Some(TypeAwareFailure::Runtime("checker failure".into())),
    );
    let rules = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect::<Vec<_>>();
    assert_eq!(rules, ["script/existing", "type/corsa-runtime"]);
    assert_eq!(result.warning_count, 2);
    assert_eq!(result.error_count, 0);
}

#[test]
fn no_query_plain_plan_keeps_type_severity_without_reporting_project_failure() {
    let linter = configured()
        .with_rule_severity_overrides(vec![(RULE_NO_FLOATING_PROMISES.into(), Severity::Error)]);
    let mut initial = empty_result("owned.ts");
    initial.diagnostics.push(LintDiagnostic::warn(
        RULE_NO_FLOATING_PROMISES,
        "existing type finding",
        0,
        1,
    ));
    initial.warning_count = 1;
    let prepared = prepare_plain_module(&linter, "export const value = 1;", "owned.ts", initial);
    assert!(prepared.virtual_content().is_none());
    let result = prepared.finish(
        &linter,
        Some(TypeAwareFailure::Runtime("unused project".into())),
    );
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn javascript_and_declaration_files_do_not_create_checker_plans() {
    let linter = configured();
    for filename in ["owned.js", "owned.d.ts", "owned.d.mts", "owned.d.cts"] {
        let prepared = prepare_plain_module(&linter, "save();", filename, empty_result(filename));
        assert!(prepared.virtual_content().is_none());
        let result = prepared.finish(
            &linter,
            Some(TypeAwareFailure::Runtime("unused project".into())),
        );
        assert!(result.diagnostics.is_empty());
    }
}
