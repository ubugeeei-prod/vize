//! Differential coverage for `eslint-plugin-vue@10.9.2`.

use super::NoUseVElseWithVFor;
use crate::Locale;
use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::preset::LintPreset;
use crate::rule::{Rule, RuleRegistry};
use vize_s0::String;
use vize_s0::i18n::Translator;

const RULE: &str = "vue/no-use-v-else-with-v-for";
const ELSE_MESSAGE: &str = "Unexpected `v-else` and `v-for` on the same element. Move `v-else` to a wrapper element instead.";
const ELSE_IF_MESSAGE: &str = "Unexpected `v-else-if` and `v-for` on the same element. Move `v-else-if` to a wrapper element instead.";

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoUseVElseWithVFor));
    Linter::with_registry(registry)
}

fn assert_single_finding(source: &str, message: &str, expected_slice: &str) {
    let result = linter().lint_template(source, "test.vue");
    assert_eq!(result.warning_count, 1, "source: {source}");
    assert_eq!(result.error_count, 0, "source: {source}");
    assert_eq!(result.diagnostics.len(), 1, "source: {source}");

    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, RULE);
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(diagnostic.message.as_str(), message);
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        expected_slice,
    );
    assert!(diagnostic.fix.is_none(), "upstream rule is not fixable");
}

#[test]
fn reports_v_else_with_v_for_in_both_attribute_orders() {
    for (source, expected) in [
        (
            r#"<div v-if="foo"></div><div v-else v-for="x in xs">{{ x }}</div>"#,
            r#"<div v-else v-for="x in xs">{{ x }}</div>"#,
        ),
        (
            r#"<div v-if="foo"></div><div v-for="x in xs" v-else>{{ x }}</div>"#,
            r#"<div v-for="x in xs" v-else>{{ x }}</div>"#,
        ),
    ] {
        assert_single_finding(source, ELSE_MESSAGE, expected);
    }
}

#[test]
fn reports_v_else_if_with_v_for_in_both_attribute_orders() {
    for (source, expected) in [
        (
            r#"<div v-if="foo"></div><div v-else-if="bar" v-for="x in xs">{{ x }}</div>"#,
            r#"<div v-else-if="bar" v-for="x in xs">{{ x }}</div>"#,
        ),
        (
            r#"<div v-if="foo"></div><div v-for="x in xs" v-else-if="bar">{{ x }}</div>"#,
            r#"<div v-for="x in xs" v-else-if="bar">{{ x }}</div>"#,
        ),
    ] {
        assert_single_finding(source, ELSE_IF_MESSAGE, expected);
    }
}

#[test]
fn reports_the_complete_nested_element_span() {
    let source = r#"<div v-if="foo"></div><section v-else v-for="x in xs"><section>{{ x }}</section></section>"#;
    let expected = r#"<section v-else v-for="x in xs"><section>{{ x }}</section></section>"#;
    assert_single_finding(source, ELSE_MESSAGE, expected);
}

#[test]
fn sfc_diagnostic_uses_absolute_offsets() {
    let source = "<template>\n  <div v-if=\"foo\"></div>\n  <div v-else v-for=\"x in xs\">{{ x }}</div>\n</template>\n";
    let result = linter().lint_sfc(source, "test.vue");
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.start, 38);
    assert_eq!(diagnostic.end, 79);
    assert_eq!(
        &source[38..79],
        r#"<div v-else v-for="x in xs">{{ x }}</div>"#
    );
}

#[test]
fn accepts_each_control_flow_shape_when_it_is_not_mixed() {
    let valid = [
        r#"<div v-if="foo" v-for="x in xs">{{ x }}</div>"#,
        r#"<div v-if="foo"></div><div v-else-if="bar"></div><div v-else></div>"#,
        r#"<div v-for="x in xs">{{ x }}</div>"#,
        r#"<div v-if="foo"></div><template v-else-if="bar"><div v-for="x in xs">{{ x }}</div></template>"#,
        r#"<div v-if="foo"></div><template v-else><div v-for="x in xs">{{ x }}</div></template>"#,
    ];

    for source in valid {
        let result = linter().lint_template(source, "test.vue");
        assert!(result.diagnostics.is_empty(), "source: {source}");
    }
}

#[test]
fn remains_opt_in_and_can_be_enabled_by_exact_name() {
    assert!(!RuleRegistry::default().has_rule(RULE));
    assert!(!RuleRegistry::with_happy_path().has_rule(RULE));
    assert!(!RuleRegistry::with_essential().has_rule(RULE));
    assert!(!RuleRegistry::with_opinionated().has_rule(RULE));
    assert!(!RuleRegistry::with_ecosystem().has_rule(RULE));
    assert!(!RuleRegistry::with_nuxt().has_rule(RULE));
    assert!(RuleRegistry::with_opt_in_rules().has_rule(RULE));

    let source = r#"<div v-if="foo"></div><div v-else v-for="x in xs">{{ x }}</div>"#;
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![String::from(RULE)]))
        .lint_template(source, "test.vue");
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].rule_name, RULE);
}

#[test]
fn exact_rule_does_not_enable_the_related_v_if_rule() {
    let source = r#"<div v-if="foo" v-for="x in xs">{{ x }}</div>"#;
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![String::from(RULE)]))
        .lint_template(source, "test.vue");
    assert!(result.diagnostics.is_empty());
}

#[test]
fn metadata_pins_severity_and_no_fix_contract() {
    let rule = NoUseVElseWithVFor;
    let meta = rule.meta();
    assert_eq!(meta.name, RULE);
    assert_eq!(meta.default_severity, Severity::Warning);
    assert!(!meta.fixable);
}

#[test]
fn every_locale_owns_the_rule_messages_without_fallback() {
    let translator = Translator::new();
    for &locale in Locale::ALL {
        for key in [
            "vue/no-use-v-else-with-v-for.description",
            "vue/no-use-v-else-with-v-for.message",
            "vue/no-use-v-else-with-v-for.help",
        ] {
            assert!(
                translator.has_key(locale, key),
                "{locale:?} is missing {key}"
            );
        }
    }
}

#[test]
fn directive_name_is_localized_in_every_supported_locale() {
    let source = r#"<div v-if="foo"></div><div v-else v-for="x in xs">{{ x }}</div>"#;
    let cases = [
        (Locale::En, ELSE_MESSAGE),
        (
            Locale::Ja,
            "同じ要素で`v-else`と`v-for`を使用しないでください。`v-else`をラッパー要素に移動してください。",
        ),
        (
            Locale::Zh,
            "同一元素上不应同时使用`v-else`和`v-for`。请将`v-else`移到包装元素上。",
        ),
    ];

    for (locale, message) in cases {
        let result = linter()
            .with_locale(locale)
            .lint_template(source, "test.vue");
        assert_eq!(result.diagnostics[0].message.as_str(), message);
    }
}
