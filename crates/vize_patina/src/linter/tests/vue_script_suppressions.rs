//! Vue compatibility IDs must suppress only their registered script counterpart.

use super::Linter;
use vize_l0::{String, cstr};

const CASES: [(&str, &str, &str, &str); 4] = [
    (
        "script/no-deprecated-dollar-listeners-api",
        "vue/no-deprecated-dollar-listeners-api",
        "export default { created() { return this.$listeners } }",
        "export default { created() { return this.$attrs } }",
    ),
    (
        "script/no-deprecated-dollar-scopedslots-api",
        "vue/no-deprecated-dollar-scopedslots-api",
        "export default { created() { return this.$scopedSlots } }",
        "export default { created() { return this.$slots } }",
    ),
    (
        "script/no-use-computed-property-like-method",
        "vue/no-use-computed-property-like-method",
        "export default { computed: { size() { return 3 } }, methods: { read() { return this.size() } } }",
        "export default { computed: { size() { return 3 } }, methods: { read() { return this.size } } }",
    ),
    (
        "script/require-valid-default-prop",
        "vue/require-valid-default-prop",
        "export default { props: { enabled: { type: Boolean, default: 3 } } }",
        "export default { props: { enabled: { type: Boolean, default: false } } }",
    ),
];

fn component(script: &str) -> String {
    cstr!("<script>\n{script}\n</script>\n<template><div /></template>\n")
}

fn count(rule: &str, script: &str) -> usize {
    let linter = Linter::new().with_enabled_rules(Some(vec![rule.into()]));
    let result = linter.lint_sfc(&component(script), "AliasControls.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|finding| finding.rule_name == rule),
        "unexpected finding for {rule}: {:?}",
        result.diagnostics
    );
    result.diagnostics.len()
}

#[test]
fn vue_script_alias_controls_retain_unsuppressed_positives_and_clean_sources() {
    for (native, _, positive, clean) in CASES {
        assert_eq!(count(native, positive), 1, "{native}");
        assert_eq!(count(native, clean), 0, "{native}");
        assert_eq!(
            count(
                native,
                &cstr!("/* eslint-disable vue/no-v-html */\n{positive}")
            ),
            1,
            "an unrelated rule must not suppress {native}"
        );
    }
}

#[test]
fn vue_script_aliases_work_for_block_next_line_and_line_pragmas() {
    for (native, alias, positive, _) in CASES {
        for (marker, prefix) in [
            ("eslint", ""),
            ("oxlint", ""),
            ("eslint", "vize/"),
            ("oxlint", "vize/"),
        ] {
            for script in [
                cstr!("/* {marker}-disable {prefix}{alias} */\n{positive}"),
                cstr!("// {marker}-disable-next-line {prefix}{alias}\n{positive}"),
                cstr!("{positive} // {marker}-disable-line {prefix}{alias}"),
            ] {
                assert_eq!(count(native, &script), 0, "{native}: {script}");
            }
        }
    }
}

#[test]
fn vue_and_native_script_names_share_region_enable_state() {
    for (native, alias, positive, _) in CASES {
        for (disabled, enabled) in [(native, alias), (alias, native)] {
            let script =
                cstr!("/* eslint-disable {disabled} */\n/* eslint-enable {enabled} */\n{positive}");
            assert_eq!(count(native, &script), 1, "{native}: {script}");
        }
    }
}

#[test]
fn each_vue_script_alias_preserves_every_other_enabled_finding() {
    let body = "export default { props: { count: { type: Number, default: 'bad' } }, computed: { total() { return 1 } }, methods: { read() { this.$listeners; this.$scopedSlots; return this.total() } } }";
    let linter = Linter::new().with_enabled_rules(Some(
        CASES
            .iter()
            .map(|(native, _, _, _)| (*native).into())
            .collect(),
    ));
    for (native, alias, _, _) in CASES {
        let result = linter.lint_sfc(
            &component(&cstr!("/* eslint-disable {alias} */\n{body}")),
            "OtherFindings.vue",
        );
        let mut actual: Vec<_> = result
            .diagnostics
            .iter()
            .map(|finding| finding.rule_name)
            .collect();
        actual.sort_unstable();
        let mut expected: Vec<_> = CASES
            .iter()
            .map(|(rule, _, _, _)| *rule)
            .filter(|rule| *rule != native)
            .collect();
        expected.sort_unstable();
        assert_eq!(actual, expected, "{alias}");
    }
}
