use super::{LintPreset, Linter};

#[test]
fn nuxt_preset_allows_options_api_components() {
    let linter = Linter::with_preset(LintPreset::Nuxt);
    let sfc = r#"<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'AppLoader',
  props: {
    active: Boolean
  }
})
</script>
"#;
    let result = linter.lint_sfc(sfc, "components/AppLoader.vue");

    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn nuxt_preset_allows_next_tick_when_vapor_is_explicitly_disabled() {
    let linter = Linter::with_preset(LintPreset::Nuxt).with_vapor_mode(Some(false));
    let sfc = r#"<script setup lang="ts">
import { nextTick } from 'vue'

await nextTick()
</script>
"#;
    let result = linter.lint_sfc(sfc, "components/Dialog.vue");

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-next-tick"),
        "non-Vapor Nuxt projects should not report script/no-next-tick, got {:?}",
        result.diagnostics
    );
}

#[test]
fn nuxt_preset_keeps_next_tick_diagnostic_when_vapor_is_unspecified() {
    let linter = Linter::with_preset(LintPreset::Nuxt);
    let sfc = r#"<script setup lang="ts">
import { nextTick } from 'vue'

await nextTick()
</script>
"#;
    let result = linter.lint_sfc(sfc, "components/Dialog.vue");

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-next-tick"),
        "default Nuxt preset should remain unchanged, got {:?}",
        result.diagnostics
    );
}
