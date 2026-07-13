use super::{LintPreset, Linter};

#[test]
fn test_lint_sfc_opinionated_reports_no_next_tick_when_rule_is_enabled() {
    let result = Linter::with_preset(LintPreset::Opinionated)
        .with_additional_rules(vec!["script/no-next-tick".into()])
        .lint_sfc(
            r#"<script setup lang="ts">
import { nextTick } from 'vue'

await nextTick()
</script>
"#,
            "test.vue",
        );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "script/no-next-tick"),
        "explicit script/no-next-tick should still report, got {:?}",
        result.diagnostics
    );
}

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

#[test]
fn shared_module_facts_and_diagnostics_are_authoritative() {
    let source = "import { ref } from '@vue/reactivity';";
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_additional_rules(vec!["script/prefer-import-from-vue".into()]);
    let mut module = vize_module::snapshot_module(
        "state.ts",
        source,
        vize_module::ModuleLanguage::TypeScript,
        0,
        None,
    );
    let baseline = linter.lint_script_with_shared_artifacts(&module, "state.ts");
    assert!(
        baseline
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/prefer-import-from-vue")
    );

    module.imports.clear();
    let without_import_fact = linter.lint_script_with_shared_artifacts(&module, "state.ts");
    assert!(
        without_import_fact
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "script/prefer-import-from-vue")
    );

    module.diagnostics.push(vize_module::ModuleDiagnostic {
        message: "authoritative parse failure".into(),
        span: vize_module::ModuleSpan::new(0, 6),
    });
    let malformed = linter.lint_script_with_shared_artifacts(&module, "state.ts");
    assert!(
        malformed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "parser/module")
    );
}
