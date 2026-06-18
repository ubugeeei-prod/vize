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
