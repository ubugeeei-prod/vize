use super::{NoDeprecatedFilter, has_filter_pipe};
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDeprecatedFilter));
    Linter::with_registry(registry)
}

/// Wrap markup in a petite-vue document so `ctx.dialect()` resolves to
/// petite-vue and the rule gates itself off.
fn petite_doc(markup: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
  <body>
    <div v-scope="{{ count: 0 }}">
{markup}
    </div>
    <script src="https://unpkg.com/petite-vue" init></script>
  </body>
</html>"#
    )
}

#[test]
fn reports_filter_in_interpolation() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>{{ message | capitalize }}</div>"#, "App.vue");
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn reports_filter_in_v_bind_shorthand() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div :id="rawId | toId"></div>"#, "App.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn reports_filter_in_v_bind_full_syntax() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div v-bind:id="rawId | toId"></div>"#, "App.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn reports_chained_filters_once_per_expression() {
    // A chain `a | b | c` is a single deprecated expression; eslint-plugin-vue
    // reports it once.
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>{{ a | b | c }}</div>"#, "App.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn reports_filter_with_arguments() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>{{ value | format('YYYY') }}</div>"#, "App.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn allows_logical_or_in_interpolation() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>{{ a || b }}</div>"#, "App.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn allows_logical_or_in_v_bind() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div :title="a || b"></div>"#, "App.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn allows_method_call_replacement() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>{{ capitalize(message) }}</div>"#, "App.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn allows_pipe_inside_string_literal() {
    // A `|` inside a string is data, not a filter.
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>{{ "a | b" }}</div>"#, "App.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn ignores_filter_in_petite_vue() {
    // petite-vue never supported filters; never flag it there.
    let linter = create_linter();
    let result = linter.lint_standalone_html(
        &petite_doc(r#"<div>{{ message | capitalize }}</div>"#),
        "index.html",
    );
    assert_eq!(result.error_count, 0);
}

// --- Unit tests for the pipe scanner itself ---

#[test]
fn scanner_detects_simple_filter() {
    assert!(has_filter_pipe("message | capitalize"));
    assert!(has_filter_pipe("a|b"));
    assert!(has_filter_pipe("a | b | c"));
}

#[test]
fn scanner_ignores_logical_or() {
    assert!(!has_filter_pipe("a || b"));
    assert!(!has_filter_pipe("a || b || c"));
    assert!(!has_filter_pipe("ok"));
}

#[test]
fn scanner_ignores_pipe_in_string() {
    assert!(!has_filter_pipe(r#""a | b""#));
    assert!(!has_filter_pipe(r#"'x | y'"#));
    assert!(!has_filter_pipe(r#"foo + "| literal""#));
}

#[test]
fn scanner_ignores_pipe_in_template_literal() {
    assert!(!has_filter_pipe("`a | b`"));
    assert!(!has_filter_pipe("`${a | b}`"));
}

#[test]
fn scanner_ignores_pipe_in_regex() {
    assert!(!has_filter_pipe("/a|b/.test(x)"));
    assert!(!has_filter_pipe("str.replace(/a|b/g, '')"));
}

#[test]
fn scanner_handles_mixed_or_and_filter() {
    // A real filter following a logical OR must still be caught.
    assert!(has_filter_pipe("a || b | c"));
    // Division is not a regex; a following filter is still caught.
    assert!(has_filter_pipe("a / b | c"));
}
