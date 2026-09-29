use super::NoVHtml;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoVHtml));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_interpolation() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>{{ content }}</div>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_v_html() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div v-html="content"></div>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_disable_next_line_suppresses_single_line_element() {
    // Baseline: the suppression works when the element (and its `v-html`)
    // sit on the single line right after the comment.
    let linter = create_linter();
    let src = "<!-- eslint-disable-next-line vue/no-v-html -->\n<div v-html=\"content\"></div>";
    let result = linter.lint_template(src, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_disable_next_line_survives_wrapped_opening_tag() {
    // The formatter wraps a long opening tag, so `v-html` lands several
    // lines below the element's start. The `eslint-disable-next-line`
    // above the element must still suppress the finding, otherwise
    // formatting would introduce a warning (lint-agreement, #3252).
    let linter = create_linter();
    let src = "<!-- eslint-disable-next-line vue/no-v-html -->\n<div\n  class=\"a-really-long-class-name\"\n  v-html=\"content\"\n></div>";
    let result = linter.lint_template(src, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_multiline_v_html_without_suppression_still_warns() {
    // Regression guard: without a disable comment, a wrapped element with
    // `v-html` still reports exactly one warning.
    let linter = create_linter();
    let src = "<div\n  class=\"a-really-long-class-name\"\n  v-html=\"content\"\n></div>";
    let result = linter.lint_template(src, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_inner_html_bindings_are_reported() {
    let source = "\
<div v-html=\"html\" />
<div :innerHTML=\"html\" />
<div v-bind:innerHTML=\"html\" />
<div .innerHTML=\"html\" />
<div v-bind=\"{ innerHTML: html }\" />
<div :outerHTML=\"html\" />
<div v-bind:outerHTML=\"html\" />
<div .outerHTML=\"html\" />
<div v-bind=\"{ outerHTML: html }\" />
<div v-bind:innerHTML.prop=\"html\" />
<div :['innerHTML']=\"html\" />
<div v-bind=\"{ 'innerHTML': html, class: name }\" />
<div v-bind=\"{ innerHTML }\" />
<div v-bind=\"({ outerHTML: html })\" />
<div v-bind=\"{ ...{ innerHTML: html } }\" />";
    assert_eq!(
        sink_slices(source),
        vec![
            "v-html=\"html\"",
            "innerHTML",
            "innerHTML",
            "innerHTML",
            "innerHTML",
            "outerHTML",
            "outerHTML",
            "outerHTML",
            "outerHTML",
            "innerHTML",
            "'innerHTML'",
            "'innerHTML'",
            "innerHTML",
            "outerHTML",
            "innerHTML",
        ]
    );
}

#[test]
fn test_unrelated_attributes_and_static_inner_html_are_not_reported() {
    for source in [
        r#"<div innerHTML="html" />"#,
        r#"<div outerHTML="html" />"#,
        r#"<div :class="html" />"#,
        r#"<div v-bind:title="html" />"#,
        r#"<div :id="innerHTML" />"#,
        r#"<div v-bind="{ class: html }" />"#,
        r#"<div :class="{ innerHTML: true }" />"#,
        r#"<div :[innerHTML]="html" />"#,
        r#"<div v-bind="html" />"#,
        r#"<div v-bind="{ ...html }" />"#,
        r#"<div v-bind="{ innerhtml: html }" />"#,
        r#"<div v-bind="{ [innerHTML]: html }" />"#,
        r#"<div v-bind:title="{ innerHTML: html }" />"#,
        r#"<div>{{ innerHTML }}</div>"#,
    ] {
        assert_no_sinks(source);
    }
}

#[test]
fn test_disable_next_line_suppresses_wrapped_inner_html() {
    let linter = create_linter();
    let src =
        "<!-- eslint-disable-next-line vue/no-v-html -->\n<div\n  :innerHTML=\"content\"\n></div>";
    let result = linter.lint_template(src, "test.vue");
    assert_eq!(result.warning_count, 0);
}

fn sink_slices(source: &str) -> Vec<&str> {
    let linter = create_linter();
    let result = linter.lint_template(source, "test.vue");
    assert_eq!(result.error_count, 0, "{source}");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name == "vue/no-v-html"),
        "{source}: {:?}",
        result.diagnostics
    );
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or("")
        })
        .collect()
}

fn assert_no_sinks(source: &str) {
    let linter = create_linter();
    let result = linter.lint_template(source, "test.vue");
    assert_eq!(
        result.warning_count, 0,
        "{source}: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0, "{source}");
}
