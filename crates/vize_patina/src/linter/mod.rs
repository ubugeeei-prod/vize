//! Main linter entry point.
//!
//! High-performance Vue template linter with arena allocation.
//! Split into:
//! - [`config`]: `Linter` struct, builder methods, and `LintResult`
//! - [`engine`]: Core linting methods and template extraction

mod config;
mod engine;
#[cfg(not(target_arch = "wasm32"))]
mod native_type_aware;
pub(crate) mod script_rules;

pub use config::{LintResult, Linter};

#[cfg(test)]
mod tests {
    use super::Linter;
    use crate::LintPreset;
    use vize_carton::{Allocator, ToCompactString};

    #[test]
    fn test_lint_empty_template() {
        let linter = Linter::new();
        let result = linter.lint_template("", "test.vue");
        assert!(!result.has_errors());
        assert!(!result.has_diagnostics());
    }

    #[test]
    fn test_lint_simple_template() {
        let linter = Linter::new();
        let result = linter.lint_template("<div>Hello</div>", "test.vue");
        assert!(!result.has_errors());
    }

    #[test]
    fn test_lint_with_allocator_reuse() {
        let linter = Linter::new();
        let allocator = Allocator::with_capacity(1024);

        let result1 =
            linter.lint_template_with_allocator(&allocator, "<div>Hello</div>", "test1.vue");
        assert!(!result1.has_errors());

        // Allocator is borrowed, can't reset here, but demonstrates the API
    }

    #[test]
    fn test_lint_files_batch() {
        let linter = Linter::new();
        let files = vec![
            (
                "test1.vue".to_compact_string(),
                "<div>Hello</div>".to_compact_string(),
            ),
            (
                "test2.vue".to_compact_string(),
                "<span>World</span>".to_compact_string(),
            ),
        ];

        let (results, summary) = linter.lint_files(&files);
        assert_eq!(results.len(), 2);
        assert_eq!(summary.file_count, 2);
    }

    #[test]
    fn test_vize_forget_suppresses_next_element() {
        let linter = Linter::new();
        // Without @vize:forget - should have error
        let result = linter.lint_template(
            r#"<ul><li v-for="item in items">{{ item }}</li></ul>"#,
            "test.vue",
        );
        assert!(result.error_count > 0, "Should have error without key");

        // With @vize:forget - should suppress error on the next element
        let result = linter.lint_template(
            r#"<ul><!-- @vize:forget v-for key not needed here -->
<li v-for="item in items">{{ item }}</li></ul>"#,
            "test.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "Error should be suppressed by @vize:forget"
        );
    }

    #[test]
    fn test_vize_forget_without_reason_warns() {
        let linter = Linter::new();
        let result = linter.lint_template(
            r#"<ul><!-- @vize:forget -->
<li v-for="item in items">{{ item }}</li></ul>"#,
            "test.vue",
        );
        // The v-for error should still be suppressed
        assert_eq!(result.error_count, 0, "v-for error should be suppressed");
        // But a warning should be emitted for missing reason
        assert_eq!(result.warning_count, 1, "Should warn about missing reason");
        assert_eq!(result.diagnostics[0].rule_name, "vize/forget");
    }

    #[test]
    fn test_vize_forget_multiline_element() {
        let linter = Linter::new();
        let result = linter.lint_template(
            r#"<ul><!-- @vize:forget complex rendering -->
<li
  v-for="item in items"
  class="item"
>{{ item }}</li></ul>"#,
            "test.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "Multiline element should be fully suppressed"
        );
    }

    #[test]
    fn test_vize_forget_suppresses_template_v_for() {
        let linter = Linter::new();
        // <template v-for> becomes a ForNode in the AST, not an ElementNode.
        // @vize:forget should suppress diagnostics on ForNode too.
        let result = linter.lint_template(
            r#"<div><!-- @vize:forget template key is valid in Vue 3 -->
<template v-for="item in items" :key="item.id">
  <li>{{ item.name }}</li>
</template></div>"#,
            "test.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "ForNode should be suppressed by @vize:forget"
        );
    }

    #[test]
    fn test_vize_forget_suppresses_template_v_if() {
        let linter = Linter::new();
        // <div v-if> becomes part of an IfNode in the AST.
        // @vize:forget should suppress diagnostics on IfNode too.
        let result = linter.lint_template(
            r#"<div><!-- @vize:forget conditional rendering -->
<span v-if="show" v-for="item in items">{{ item }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "IfNode should be suppressed by @vize:forget"
        );
    }

    #[test]
    fn test_lint_sfc_extracts_template() {
        let linter = Linter::new();
        // SFC with script and template - should only lint template content
        let sfc = r#"<script setup lang="ts">
interface Props {
  schema?: BaseSchema<FormShape, FormShape, any>;
}
</script>

<template>
  <div>Hello World</div>
</template>
"#;
        let result = linter.lint_sfc(sfc, "test.vue");
        // Should not report errors for TypeScript code in <script>
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_lint_sfc_no_template() {
        let linter = Linter::new();
        // SFC without template - should return empty result
        let sfc = r#"<script setup lang="ts">
const foo = 'bar';
</script>
"#;
        let result = linter.lint_sfc(sfc, "test.vue");
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_lint_sfc_opinionated_reports_no_options_api() {
        let linter = Linter::with_preset(LintPreset::Opinionated);
        let sfc = r#"<script>
export default {
  methods: {
    increment() {}
  }
}
</script>
"#;
        let result = linter.lint_sfc(sfc, "test.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "script/no-options-api");
    }

    #[test]
    fn test_lint_sfc_happy_path_skips_no_options_api() {
        let linter = Linter::new();
        let sfc = r#"<script>
export default {
  methods: {
    increment() {}
  }
}
</script>
"#;
        let result = linter.lint_sfc(sfc, "test.vue");
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_lint_sfc_explicit_script_rule_enablement_works() {
        let linter = Linter::with_preset(LintPreset::Opinionated)
            .with_enabled_rules(Some(vec!["script/no-options-api".into()]));
        let sfc = r#"<script>
export default {
  props: {
    count: Number
  }
}
</script>
"#;
        let result = linter.lint_sfc(sfc, "test.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "script/no-options-api");
    }

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
        // SFC where template has an error - byte offset should be adjusted
        let sfc = r#"<script setup lang="ts">
const foo = 'bar';
</script>

<template>
  <ul><li v-for="item in items">{{ item }}</li></ul>
</template>
"#;
        let result = linter.lint_sfc(sfc, "test.vue");
        // Should have error for missing :key
        assert!(result.error_count > 0, "Should detect v-for without key");

        // The byte offset should point to the correct location in the original file
        if let Some(diag) = result.diagnostics.first() {
            // The diagnostic should point somewhere in the template section
            // Template starts after "<script>...</script>\n\n<template>\n"
            assert!(
                diag.start > 50,
                "Byte offset should be adjusted for template position"
            );
        }
    }

    #[test]
    fn test_lint_sfc_offset_line_conversion() {
        use crate::telegraph::LspEmitter;

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

        // Debug: show template start
        let template_start = sfc.find("<template>").unwrap();
        eprintln!("Template <template> starts at byte: {}", template_start);

        // Debug: show content start (after <template>)
        let content_start = sfc.find("<template>").unwrap() + "<template>\n".len();
        eprintln!("Template content starts at byte: {}", content_start);

        // Debug: show diagnostics
        for (i, diag) in result.diagnostics.iter().enumerate() {
            eprintln!(
                "Diag[{}] rule={}, start={}, end={}",
                i, diag.rule_name, diag.start, diag.end
            );

            // Count newlines before start to get line number
            let before = &sfc[..diag.start as usize];
            let line_count = before.matches('\n').count();
            eprintln!("  -> Line (0-indexed): {}", line_count);
        }

        // Test LspEmitter conversion
        let lsp_diags = LspEmitter::to_lsp_diagnostics_with_source(&result, sfc);
        for (i, lsp) in lsp_diags.iter().enumerate() {
            eprintln!(
                "LSP[{}] line={}, col={}",
                i, lsp.range.start.line, lsp.range.start.character
            );
        }

        // Expected: line should be around 5 (0-indexed) for template content
        // Line 0: <script setup lang="ts">
        // Line 1: const foo = 'bar';
        // Line 2: </script>
        // Line 3: (empty)
        // Line 4: <template>
        // Line 5:   <ul>...
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
        // SFC with nested template elements - should extract full content
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
        // Should not have errors - nested templates have v-if/v-else directives
        // Most importantly, should not report "no-lone-template" on the root <template>
        assert_eq!(
            result.error_count, 0,
            "Should not report errors for valid nested templates with directives"
        );
    }

    #[test]
    fn test_lint_sfc_with_nested_template_extraction() {
        // Test that nested templates are properly handled via parse_sfc
        let linter = Linter::new();
        let sfc = r#"<script></script>
<template>
  <div>
    <template v-if="x">nested</template>
  </div>
</template>"#;

        let result = linter.lint_sfc(sfc, "test.vue");
        // Should not report errors for the nested template with v-if directive
        assert_eq!(
            result.error_count, 0,
            "Should properly extract and lint nested templates"
        );
    }

    #[test]
    fn test_vize_todo_emits_warning() {
        let linter = Linter::new();
        let result = linter.lint_template(
            r#"<div><!-- @vize:todo fix this --><span>hello</span></div>"#,
            "test.vue",
        );
        assert_eq!(
            result.warning_count, 1,
            "Should emit 1 warning for @vize:todo"
        );
        assert_eq!(result.diagnostics[0].rule_name, "vize/todo");
        assert!(result.diagnostics[0].message.contains("TODO"));
    }

    #[test]
    fn test_vize_fixme_emits_error() {
        let linter = Linter::new();
        let result = linter.lint_template(
            r#"<div><!-- @vize:fixme broken --><span>hello</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1, "Should emit 1 error for @vize:fixme");
        assert_eq!(result.diagnostics[0].rule_name, "vize/fixme");
        assert!(result.diagnostics[0].message.contains("FIXME"));
    }

    #[test]
    fn test_vize_deprecated_emits_warning() {
        let linter = Linter::new();
        let result = linter.lint_template(
            r#"<div><!-- @vize:deprecated use NewComp --><span>hello</span></div>"#,
            "test.vue",
        );
        assert_eq!(
            result.warning_count, 1,
            "Should emit 1 warning for @vize:deprecated"
        );
        assert_eq!(result.diagnostics[0].rule_name, "vize/deprecated");
        assert!(result.diagnostics[0].message.contains("Deprecated"));
    }

    #[test]
    fn test_vize_expected_suppresses_error() {
        let linter = Linter::new();
        // Without @vize:expected - should have error
        let result = linter.lint_template(
            r#"<ul><li v-for="item in items">{{ item }}</li></ul>"#,
            "test.vue",
        );
        assert!(result.error_count > 0, "Should have error without key");

        // With @vize:expected - should suppress error on next line
        let result = linter.lint_template(
            r#"<ul><!-- @vize:expected -->
<li v-for="item in items">{{ item }}</li></ul>"#,
            "test.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "Error should be suppressed by @vize:expected"
        );
    }

    #[test]
    fn test_vize_ignore_start_end_region() {
        let linter = Linter::new();
        // With @vize:ignore-start/end - should suppress errors in region
        let result = linter.lint_template(
            r#"<!-- @vize:ignore-start -->
<ul><li v-for="item in items">{{ item }}</li></ul>
<!-- @vize:ignore-end -->"#,
            "test.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "Errors in ignore region should be suppressed"
        );
    }

    #[test]
    fn test_vize_docs_no_lint_effect() {
        let linter = Linter::new();
        let result = linter.lint_template(
            r#"<div><!-- @vize:docs Component documentation --><span>hello</span></div>"#,
            "test.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "Docs directive should not produce errors"
        );
        assert_eq!(
            result.warning_count, 0,
            "Docs directive should not produce warnings"
        );
    }
}
