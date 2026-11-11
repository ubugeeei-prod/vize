use crate::linter::Linter;
use crate::preset::LintPreset;
use vize_l0::config::ConfigLintRuleOptions;

const LONG: &str = "\
<script setup lang=\"ts\">\n\
const label = \"a\";\n\
</script>\n\
\n\
<template>\n\
  <ul>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
  </ul>\n\
</template>\n";

fn messages(source: &str, options: serde_json::Value) -> Vec<(usize, String)> {
    let parsed: ConfigLintRuleOptions = serde_json::from_value(options).unwrap();
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/max-lines-per-block".into()]))
        .with_config_rule_options(&parsed)
        .lint_sfc(source, "Long.vue");
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/max-lines-per-block")
        .map(|diagnostic| (diagnostic.start as usize, diagnostic.message.to_string()))
        .collect()
}

#[test]
fn the_issue_template_reports_12_lines_at_the_opening_tag() {
    let start = LONG.find("<template>").unwrap();
    assert_eq!(
        messages(
            LONG,
            serde_json::json!({ "vue/max-lines-per-block": { "template": 10 } })
        ),
        vec![(
            start,
            "Block has too many lines (12). Maximum allowed is 10.".into()
        )]
    );
}

#[test]
fn an_empty_object_and_the_default_instance_check_nothing() {
    assert!(messages(LONG, serde_json::json!({ "vue/max-lines-per-block": {} })).is_empty());
    let enabled = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/max-lines-per-block".into()]))
        .lint_sfc(LONG, "Long.vue");
    assert!(
        enabled
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/max-lines-per-block")
    );
}

#[test]
fn skip_blank_lines_subtracts_empty_lines_of_the_whole_element() {
    let source = "<template>\n\n<p />\n\n</template>\n";
    assert_eq!(
        messages(
            source,
            serde_json::json!({ "vue/max-lines-per-block": { "template": 2 } })
        ),
        vec![(
            source.find("<template>").unwrap(),
            "Block has too many lines (3). Maximum allowed is 2.".into()
        )]
    );
    assert!(
        messages(
            source,
            serde_json::json!({ "vue/max-lines-per-block": { "template": 2, "skipBlankLines": true } })
        )
        .is_empty()
    );
}

#[test]
fn script_and_script_setup_share_the_script_limit_and_custom_blocks_do_not() {
    let source = "\
<script>\n\
const a = 1;\n\
const b = 2;\n\
</script>\n\
<script setup>\n\
const c = 3;\n\
const d = 4;\n\
</script>\n\
<docs>\n\
one\n\
two\n\
three\n\
</docs>\n";
    let found = messages(
        source,
        serde_json::json!({ "vue/max-lines-per-block": { "script": 1 } }),
    );
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found
            .iter()
            .all(|(_, message)| message == "Block has too many lines (2). Maximum allowed is 1.")
    );
    assert!(
        messages(
            source,
            serde_json::json!({ "vue/max-lines-per-block": { "template": 1 } })
        )
        .is_empty()
    );
}

#[test]
fn a_style_block_uses_the_style_limit() {
    let source = "<style>\na {}\nb {}\n</style>\n";
    assert_eq!(
        messages(
            source,
            serde_json::json!({ "vue/max-lines-per-block": { "style": 1 } })
        ),
        vec![(
            source.find("<style>").unwrap(),
            "Block has too many lines (2). Maximum allowed is 1.".into()
        )]
    );
}
