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
