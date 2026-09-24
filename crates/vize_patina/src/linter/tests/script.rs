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
fn non_vapor_composable_allows_get_current_instance_in_bundled_presets() {
    let source = r#"import { getCurrentInstance } from "vue";

export const useInstanceProxy = () => {
  const instance = getCurrentInstance();
  return instance?.proxy;
};
"#;
    for preset in [LintPreset::HappyPath, LintPreset::Opinionated] {
        let result =
            Linter::with_preset(preset).lint_script(source, "src/composables/useInstance.ts");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "script/no-get-current-instance"),
            "{preset:?}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn explicitly_enabled_get_current_instance_rule_runs_on_plain_script() {
    let source = "import { getCurrentInstance } from 'vue'; getCurrentInstance();";
    let result = Linter::with_preset(LintPreset::Opinionated)
        .with_additional_rules(vec!["script/no-get-current-instance".into()])
        .lint_script(source, "src/composables/useInstance.ts");
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "script/no-get-current-instance")
            .count(),
        2,
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn project_vapor_mode_runs_get_current_instance_rule_on_plain_script() {
    let source = "import { getCurrentInstance } from 'vue'; getCurrentInstance();";
    let result = Linter::with_preset(LintPreset::Opinionated)
        .with_vapor_mode(Some(true))
        .lint_script(source, "src/composables/useInstance.ts");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-get-current-instance"),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn lint_script_allows_generated_default_exported_data_objects() {
    let source = r##"
/**
 * Do not edit directly, this file was auto-generated.
 */
export default {
  color: {
    white: { value: "#ffffff" },
    primary: { value: "#4bc4cc" },
  },
};
"##;
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
