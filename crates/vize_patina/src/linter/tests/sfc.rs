use super::{LintPreset, Linter};
use crate::telegraph::LspEmitter;

#[test]
fn test_lint_sfc_opinionated_reports_no_next_tick() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let sfc = r#"<script setup lang="ts">
import { nextTick } from 'vue'

await nextTick()
</script>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.rule_name == "script/no-next-tick"));
}

#[test]
fn test_lint_sfc_opinionated_reports_no_get_current_instance() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let sfc = r#"<script setup lang="ts">
import { getCurrentInstance } from 'vue'

const instance = getCurrentInstance()
</script>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.rule_name == "script/no-get-current-instance"));
}

#[test]
fn test_lint_sfc_byte_offset() {
    let linter = Linter::new();
    let sfc = r#"<script setup lang="ts">
const foo = 'bar';
</script>

<template>
  <ul><li v-for="item in items">{{ item }}</li></ul>
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert!(result.error_count > 0, "Should detect v-for without key");

    if let Some(diag) = result.diagnostics.first() {
        assert!(
            diag.start > 50,
            "Byte offset should be adjusted for template position"
        );
    }
}

#[test]
fn test_lint_sfc_offset_line_conversion() {
    let linter = Linter::new();
    let sfc = r#"<script setup lang="ts">
const foo = 'bar';
</script>

<template>
  <ul><li v-for="item in items">{{ item }}</li></ul>
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert!(result.error_count > 0);

    let lsp_diags = LspEmitter::to_lsp_diagnostics_with_source(&result, sfc);
    if let Some(lsp) = lsp_diags.first() {
        assert_eq!(
            lsp.range.start.line, 5,
            "First diagnostic should be on line 5 (0-indexed)"
        );
    }
}

#[test]
fn test_lint_sfc_with_nested_templates() {
    let linter = Linter::new();
    let sfc = r#"<script setup lang="ts">
const show = true;
</script>

<template>
  <div>
    <template v-if="show">
      <span>Visible</span>
    </template>
    <template v-else>
      <span>Hidden</span>
    </template>
  </div>
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert_eq!(
        result.error_count, 0,
        "Should not report errors for valid nested templates with directives"
    );
}

#[test]
fn test_lint_sfc_with_nested_template_extraction() {
    let linter = Linter::new();
    let sfc = r#"<script></script>
<template>
  <div>
    <template v-if="x">nested</template>
  </div>
</template>"#;

    let result = linter.lint_sfc(sfc, "test.vue");
    assert_eq!(
        result.error_count, 0,
        "Should properly extract and lint nested templates"
    );
}
