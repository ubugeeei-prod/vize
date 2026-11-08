use super::{LintPreset, Linter};

fn sorted_rules(result: &crate::LintResult) -> Vec<&'static str> {
    let mut rules = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect::<Vec<_>>();
    rules.sort_unstable();
    rules
}

#[test]
fn rule_selector_keeps_peer_diagnostics_on_the_same_node() {
    let linter = Linter::new().with_enabled_rules(Some(vec![
        "a11y/click-events-have-key-events".into(),
        "a11y/no-static-element-interactions".into(),
        "vue/no-bare-strings-in-template".into(),
    ]));
    let result = linter.lint_template(
        r#"<!-- @vize:forget a11y/click-events-have-key-events -->
<div @click="handleClick">click</div>"#,
        "test.vue",
    );

    assert_eq!(
        sorted_rules(&result),
        vec![
            "a11y/no-static-element-interactions",
            "vue/no-bare-strings-in-template",
        ]
    );
}

#[test]
fn rule_selector_matches_the_lsp_generated_shape() {
    let result = Linter::with_preset(LintPreset::Opinionated)
        .with_editor_callback_order()
        .lint_template(
            r#"<!-- @vize:forget a11y/alt-text -->
<img src="/logo.png">"#,
            "test.vue",
        );

    assert_eq!(
        sorted_rules(&result),
        vec!["vue/multi-word-component-names"]
    );
}

#[test]
fn rule_selector_never_reaches_a_same_line_sibling() {
    let linter = Linter::new().with_enabled_rules(Some(vec![
        "vue/require-v-for-key".into(),
        "vue/no-multiple-template-root".into(),
    ]));
    for newline in ["\n", "\r\n"] {
        let source = r#"<template>
  <!-- @vize:forget vue/require-v-for-key --><li v-for="item in items">{{ item }}</li><li v-for="item in items">{{ item }}</li>
</template>"#
            .replace('\n', newline);
        let result = linter.lint_sfc(&source, "test.vue");

        assert_eq!(
            sorted_rules(&result),
            vec!["vue/no-multiple-template-root", "vue/require-v-for-key"],
            "{source:?}"
        );
    }
}

#[test]
fn authored_text_and_interpolation_are_not_target_nodes() {
    let linter = Linter::new()
        .with_enabled_rules(Some(vec![
            "vue/comment-directive".into(),
            "vue/require-v-for-key".into(),
        ]))
        .with_report_unused_disable_directives(true);
    for authored in ["authored text", "{{ value }}"] {
        let source = format!(
            "<template><!-- @vize:forget vue/require-v-for-key -->{authored}<li v-for=\"item in items\">{{{{ item }}}}</li></template>"
        );
        let result = linter.lint_sfc(&source, "AuthoredBeforeElement.vue");

        assert_eq!(
            sorted_rules(&result),
            vec!["vue/comment-directive", "vue/require-v-for-key"],
            "{authored}"
        );
    }
}
