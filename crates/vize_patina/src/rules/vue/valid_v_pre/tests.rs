//! Cases mirror eslint-plugin-vue's `valid-v-pre` suite. Templates are written
//! at the `lint_template` call so the markup differential battery picks them
//! up as fixtures.

use super::ValidVPre;
use crate::linter::{LintResult, Linter};
use crate::rule::RuleRegistry;

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVPre));
    Linter::with_registry(registry)
}

/// `(message, reported source text)` for every finding.
fn findings<'a>(source: &'a str, result: &'a LintResult) -> Vec<(&'a str, &'a str)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.rule_name, "vue/valid-v-pre");
            let text = source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default();
            (diagnostic.message.as_str(), text)
        })
        .collect()
}

/// Formerly excluded argument/modifier fixtures now require the complete
/// Relief/L1-L2 hook trace before their unchanged diagnostic assertions.
fn lint_with_markup_parity(source: &str) -> LintResult {
    use crate::markup::differential::{TemplateComparison, compare_template};
    assert!(
        matches!(compare_template(source), Ok(TemplateComparison::Compared(lines)) if lines > 0),
        "markup projections diverged for {source}"
    );
    linter().lint_template(source, "test.vue")
}

const NO_ARGUMENT: &str = "'v-pre' directives require no argument.";
const NO_MODIFIER: &str = "'v-pre' directives require no modifier.";
const NO_VALUE: &str = "'v-pre' directives require no attribute value.";

#[test]
fn accepts_bare_v_pre() {
    let result = linter().lint_template(r#"<div v-pre>{{ raw }}</div>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn accepts_bare_v_pre_on_a_nested_element() {
    let result = linter().lint_template(r#"<div><span v-pre /></div>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn accepts_names_that_only_start_like_v_pre() {
    let result = linter().lint_template(
        r#"<div :v-pre-like="a" data-v-preview="b" v-prefix:arg.mod="c"></div>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn ignores_the_spelling_inside_the_raw_subtree() {
    let result = linter().lint_template(
        r#"<div v-pre><span v-pre:arg.mod="x" v-if></span></div>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn ignores_v_pre_text_in_other_attribute_values() {
    let result = linter().lint_template(r#"<div title="v-pre:arg" v-pre></div>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn reports_argument() {
    let source = r#"<div v-pre:aaa></div>"#;
    let result = lint_with_markup_parity(source);
    assert_eq!(findings(source, &result), [(NO_ARGUMENT, "aaa")]);
}

#[test]
fn reports_all_modifiers_once() {
    let source = r#"<div v-pre.aaa.bbb></div>"#;
    let result = lint_with_markup_parity(source);
    assert_eq!(findings(source, &result), [(NO_MODIFIER, "aaa.bbb")]);
}

#[test]
fn reports_quoted_value_with_its_quotes() {
    let source = r#"<div v-pre="aaa"></div>"#;
    let result = linter().lint_template(r#"<div v-pre="aaa"></div>"#, "test.vue");
    assert_eq!(findings(source, &result), [(NO_VALUE, r#""aaa""#)]);
}

#[test]
fn reports_unquoted_value() {
    let source = r#"<div v-pre=aaa></div>"#;
    let result = linter().lint_template(r#"<div v-pre=aaa></div>"#, "test.vue");
    assert_eq!(findings(source, &result), [(NO_VALUE, "aaa")]);
}

#[test]
fn reports_argument_modifier_and_value_together() {
    let source = r#"<div class="a" v-pre:arg.mod='foo' id="b"></div>"#;
    let result = lint_with_markup_parity(source);
    assert_eq!(
        findings(source, &result),
        [
            (NO_ARGUMENT, "arg"),
            (NO_MODIFIER, "mod"),
            (NO_VALUE, "'foo'")
        ]
    );
}

#[test]
fn reports_nested_and_sibling_elements() {
    let source =
        "<section>\n  <p v-pre.a>one</p>\n  <p v-pre>two</p>\n  <p v-pre:b>three</p>\n</section>";
    let result = lint_with_markup_parity(source);
    assert_eq!(
        findings(source, &result),
        [(NO_MODIFIER, "a"), (NO_ARGUMENT, "b")]
    );
}

#[test]
fn reports_self_closing_and_multiline_tags() {
    let source = "<img\n  src=\"a.png\"\n  v-pre=\"x\"\n/>";
    let result = linter().lint_template("<img\n  src=\"a.png\"\n  v-pre=\"x\"\n/>", "test.vue");
    assert_eq!(findings(source, &result), [(NO_VALUE, "\"x\"")]);
}

#[test]
fn reports_sfc_absolute_ranges() {
    let source = "<script setup lang=\"ts\">\nconst a = 1;\n</script>\n\n<template>\n  <p v-pre:arg>{{ a }}</p>\n</template>\n";
    let result = linter().lint_sfc(source, "Raw.vue");
    let reported: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/valid-v-pre")
        .map(|diagnostic| {
            source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(reported, ["arg"]);
}
