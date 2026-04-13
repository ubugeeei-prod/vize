use super::{Allocator, LintPreset, Linter, ToCompactString};

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

    let result1 = linter.lint_template_with_allocator(&allocator, "<div>Hello</div>", "test1.vue");
    assert!(!result1.has_errors());
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
    let result = linter.lint_template(
        r#"<ul><li v-for="item in items">{{ item }}</li></ul>"#,
        "test.vue",
    );
    assert!(result.error_count > 0, "Should have error without key");

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
    assert_eq!(result.error_count, 0, "v-for error should be suppressed");
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
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_lint_sfc_no_template() {
    let linter = Linter::new();
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
