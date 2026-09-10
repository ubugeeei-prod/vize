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
fn lint_script_allows_generated_default_exported_data_objects() {
    let source = r#"
/**
 * Do not edit directly, this file was auto-generated.
 */
export default {
  breakpoint: {
    m: {
      key: '{breakpoint.m}',
      value: 960,
      type: 'dimension'
    }
  }
}
"#;
    let result = Linter::with_preset(LintPreset::Opinionated).lint_script(source, "tokens.js");

    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "script/no-options-api")
    );
}

#[test]
fn lint_script_allows_plain_config_object_with_option_like_keys() {
    let source = r#"
const config = {
  ssg: { siteName: "docs" },
  data: { items: [] }
};

export default config;
"#;
    let result = Linter::with_preset(LintPreset::Opinionated).lint_script(source, "site.config.ts");

    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "script/no-options-api")
    );
}
