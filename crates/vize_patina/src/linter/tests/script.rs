use super::{LintPreset, Linter};

#[test]
fn lint_script_runs_script_rules() {
    let result = Linter::with_preset(LintPreset::Opinionated).lint_script(
        r#"import { getCurrentInstance } from "vue";

const instance = getCurrentInstance();
"#,
        "vite.config.ts",
    );

    assert!(result.error_count > 0, "{:?}", result.diagnostics);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-get-current-instance")
    );
}
