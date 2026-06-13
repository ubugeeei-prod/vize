use super::Linter;

#[test]
fn test_lint_sfc_no_top_level_ref_skips_script_setup() {
    // In `<script setup>` top-level reactive state is fresh per component
    // instance, so `const count = ref(0)` is idiomatic and must NOT be flagged.
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["script/no-top-level-ref-in-script".into()]));
    let sfc = r#"<script setup>
const count = ref(0)
</script>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "script/no-top-level-ref-in-script")
            .count(),
        0,
        "no-top-level-ref-in-script must not fire on <script setup>"
    );
}

#[test]
fn test_lint_sfc_no_top_level_ref_reports_plain_script() {
    // Regression guard: a plain (module-scoped) `<script>` still leaks reactive
    // state across SSR requests, so the rule must keep firing there.
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["script/no-top-level-ref-in-script".into()]));
    let sfc = r#"<script>
const count = ref(0)
</script>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "script/no-top-level-ref-in-script")
            .count(),
        1,
        "no-top-level-ref-in-script must still fire on a plain <script>"
    );
}
