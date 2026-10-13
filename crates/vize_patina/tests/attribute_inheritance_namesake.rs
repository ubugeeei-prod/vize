//! A configured Rule namesake must not acquire native scriptless execution.

use vize_patina::{LintResult, Linter, Severity};

/// The full identity of every finding: rule, severity, byte range, message.
fn findings(result: &LintResult) -> Vec<(&'static str, Severity, u32, u32, &str)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect()
}

fn none() -> Vec<(&'static str, Severity, u32, u32, &'static str)> {
    Vec::new()
}

#[test]
fn registered_rule_namesake_is_not_the_native_scriptless_builtin() {
    use vize_patina::{Rule, RuleCategory, RuleMeta, RuleRegistry};
    struct Namesake;
    static META: RuleMeta = RuleMeta {
        name: "script/no-duplicate-attr-inheritance",
        description: "foreign namesake",
        category: RuleCategory::Essential,
        fixable: false,
        default_severity: Severity::Warning,
    };
    impl Rule for Namesake {
        fn meta(&self) -> &'static RuleMeta {
            &META
        }
    }
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(Namesake));
    let linter = Linter::with_registry(registry).with_enabled_rules(Some(vec![META.name.into()]));
    assert_eq!(
        findings(&linter.lint_sfc(
            "<template><div v-bind=\"$attrs\" /></template>",
            "Probe.vue"
        )),
        none()
    );
}
