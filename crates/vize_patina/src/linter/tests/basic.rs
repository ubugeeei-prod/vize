use super::{Allocator, LintPreset, Linter, ToCompactString};
use crate::Severity;

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
fn test_lint_template_reports_fatal_parser_errors_and_gates_semantic_rules() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let source = r#"<ul>
  <li v-for="(item, index) in items" :key="item">{{ item }}</li>
"#;
    let result = linter.lint_template(source, "test.vue");

    let parser_diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_name == "parser/template")
        .expect("template parser error should be reported");
    assert_eq!(parser_diagnostic.severity, Severity::Error);
    assert!(parser_diagnostic.message.contains("missing end tag"));
    assert!(parser_diagnostic.end > parser_diagnostic.start);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/no-unused-vars"),
        "semantic diagnostics should be gated by fatal template parse errors: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lint_template_reports_recoverable_parser_errors_without_gating_rules() {
    let linter = Linter::new();
    let result = linter.lint_template(r#"<div id="a" id="b"></div>"#, "test.vue");

    let parser_diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_name == "parser/template")
        .expect("recoverable template parser diagnostic should be reported");
    assert_eq!(parser_diagnostic.severity, Severity::Warning);
    assert!(parser_diagnostic.message.contains("Duplicate attribute"));
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "vue/no-duplicate-attributes"),
        "ordinary lint rules should still run after recoverable parser diagnostics"
    );
}

#[test]
fn test_lint_template_ignores_compat_self_closing_rewrite_warning() {
    let linter = Linter::new();
    let result = linter.lint_template(r#"<div />"#, "test.vue");

    assert_eq!(result.warning_count, 0);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "parser/template"),
        "compatibility rewrite warnings should not consume lint warning budget: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lint_with_allocator_reuse() {
    let linter = Linter::new();
    let allocator = Allocator::with_capacity(1024);

    let result1 = linter.lint_template_with_allocator(&allocator, "<div>Hello</div>", "test1.vue");
    assert!(!result1.has_errors());
}

#[test]
fn test_lint_template_uses_semantic_analysis_for_unused_v_for_vars() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let result = linter.lint_template(
        r#"<ul><li v-for="(_item, index) in items" :key="_item.id">{{ _item }}</li></ul>"#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-unused-vars");
    assert!(result.diagnostics[0].message.contains("index"));
}

#[test]
fn test_lint_template_reports_unused_v_for_alias_at_alias_span() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let source = r#"<template>
<div>
  <span v-for="(item, i) in items" :key="item">{{ item }}</span>
  <span v-for="(entry, j) in items" :key="entry">{{ entry }}</span>
</div>
</template>
<script setup>
const items = ['a', 'b'];
</script>"#;
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(result.warning_count, 2);

    let i_start = source.find(", i)").unwrap() as u32 + 2;
    let j_start = source.find(", j)").unwrap() as u32 + 2;
    let i_diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("'i'"))
        .expect("unused i alias should be reported");
    let j_diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("'j'"))
        .expect("unused j alias should be reported");

    assert_eq!(
        (i_diagnostic.start, i_diagnostic.end),
        (i_start, i_start + 1)
    );
    assert_eq!(
        (j_diagnostic.start, j_diagnostic.end),
        (j_start, j_start + 1)
    );
}

#[test]
fn test_ecosystem_template_rules_are_enabled_by_default() {
    let source = r#"<template><RouterLink>Home</RouterLink></template>"#;

    let happy_path = Linter::with_preset(LintPreset::HappyPath);
    let happy_path_result = happy_path.lint_sfc(source, "test.vue");
    assert_eq!(happy_path_result.error_count, 0);
    assert_eq!(happy_path_result.warning_count, 0);

    let result = Linter::new().lint_sfc(source, "test.vue");
    assert_eq!(result.error_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "ecosystem/router-link-require-to"
    );
}

#[test]
fn test_ecosystem_template_source_hints_use_full_sfc() {
    let source = r#"<script setup>
import { Link } from "@void/vue";
</script>
<template><Link>Home</Link></template>"#;

    let result = Linter::new().lint_sfc(source, "test.vue");
    assert_eq!(result.error_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "ecosystem/void-link-require-href"
    );
}

#[test]
fn test_ecosystem_script_rules_are_enabled_by_default() {
    let source = r#"<script setup lang="ts">
router.push('/settings')
</script>"#;

    let happy_path = Linter::with_preset(LintPreset::HappyPath);
    let happy_path_result = happy_path.lint_sfc(source, "test.vue");
    assert_eq!(happy_path_result.error_count, 0);
    assert_eq!(happy_path_result.warning_count, 0);

    let result = Linter::new().lint_sfc(source, "test.vue");
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "ecosystem/vue-router-prefer-named-push"
    );
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
<li v-for="_item in items">{{ _item }}</li></ul>"#,
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
fn test_lint_sfc_reports_template_parser_errors_at_sfc_offsets() {
    let linter = Linter::new();
    let source = r#"<script setup lang="ts">
const msg = "hello";
</script>

<template>
  <div>
    <span>{{ msg }}
  </div>
</template>
"#;
    let result = linter.lint_sfc(source, "test.vue");
    let parser_diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_name == "parser/template")
        .expect("template parser error should be reported for SFC templates");

    let template_start = source.find("<template>").unwrap() as u32;
    assert_eq!(parser_diagnostic.severity, Severity::Error);
    assert!(parser_diagnostic.start > template_start);
    assert!(parser_diagnostic.end > parser_diagnostic.start);
}

#[test]
fn test_lint_sfc_reports_sfc_parser_errors() {
    let linter = Linter::new();
    let source = r#"<template>
  <div v-html="userInput"></div>
"#;
    let result = linter.lint_sfc(source, "test.vue");
    let parser_diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_name == "parser/sfc")
        .expect("SFC parser error should be reported");

    assert_eq!(parser_diagnostic.severity, Severity::Error);
    assert!(parser_diagnostic.message.contains("closing tag is missing"));
    assert!(parser_diagnostic.end > parser_diagnostic.start);
    assert!(result.has_errors());
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

#[test]
fn test_lint_sfc_explicit_opt_in_script_rule_enablement_works() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["script/no-async-in-computed".into()]));
    let sfc = r#"<script setup>
const data = computed(async () => await fetch('/api'))
</script>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert_eq!(result.error_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "script/no-async-in-computed"
    );
}

#[test]
fn test_lint_sfc_additional_opt_in_script_rule_preserves_preset_rules() {
    let linter = Linter::new().with_additional_rules(vec!["script/no-async-in-computed".into()]);
    let sfc = r#"<script setup>
router.push('/settings')
const data = computed(async () => await fetch('/api'))
</script>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 1);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "ecosystem/vue-router-prefer-named-push")
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-async-in-computed")
    );
}

#[test]
fn test_lint_standalone_html_reports_cdn_options_api() {
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec!["script/no-options-api".into()]));
    let source = r##"<!doctype html>
<html>
<head>
  <script src="https://unpkg.com/vue@3/dist/vue.global.js"></script>
</head>
<body>
  <div id="app">{{ count }}</div>
  <script>
Vue.createApp({
  data() {
    return { count: 0 }
  }
}).mount("#app")
  </script>
</body>
</html>
"##;

    let result = linter.lint_standalone_html(source, "index.html");
    assert_eq!(result.error_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "script/no-options-api");
    assert!(
        result.diagnostics[0].start > source.find("Vue.createApp").unwrap() as u32,
        "diagnostic should use standalone HTML source offsets"
    );
}

#[test]
fn test_lint_standalone_html_allows_petite_vue_create_app_scope() {
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec!["script/no-options-api".into()]));
    let source = r##"<!doctype html>
<html>
<body>
  <div v-scope>{{ count }}</div>
  <script src="https://unpkg.com/petite-vue" defer init></script>
  <script>
PetiteVue.createApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
  </script>
</body>
</html>
"##;

    let result = linter.lint_standalone_html(source, "index.html");
    assert_eq!(result.error_count, 0);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "script/no-options-api")
    );
}
