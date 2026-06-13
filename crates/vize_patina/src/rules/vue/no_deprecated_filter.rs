//! vue/no-deprecated-filter
//!
//! Disallow Vue 2 filter syntax (the pipe `|` used as a filter), removed in
//! Vue 3.
//!
//! In Vue 2 you could post-process a value inside a template binding with a
//! "filter": `{{ message | capitalize }}` or `:id="rawId | toId"`. Vue 3 removed
//! filters entirely in favour of plain method calls and computed properties, so
//! a lingering filter pipe is no longer interpreted as a filter — it is parsed
//! as a JavaScript bitwise OR, silently producing the wrong value.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-filter`. The pipe is only
//! flagged in template *expression* positions: mustache interpolations
//! (`{{ … }}`) and bound attribute values (`v-bind` / `:attr`). A real bitwise
//! OR (`||`, or a `|` inside a string/regex literal) is left alone.
//!
//! ## Dialect gating
//!
//! The rule fires only for the default Vue 3 dialect. petite-vue never supported
//! filters, and the legacy Vue 2 / 2.7 dialect still understands them, so
//! neither should be flagged here.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! {{ message | capitalize }}
//! <div :id="rawId | toId" />
//! {{ a | b | c }}
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! {{ capitalize(message) }}
//! <div :id="toId(rawId)" />
//! {{ a || b }}
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-filter",
    description: "Disallow deprecated Vue 2 filter syntax (the `|` pipe)",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow deprecated Vue 2 filter syntax.
pub struct NoDeprecatedFilter;

impl Rule for NoDeprecatedFilter {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        // Filters were never part of petite-vue, and the legacy Vue 2 dialect
        // still resolves them, so only flag the default Vue 3 dialect.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        let ExpressionNode::Simple(exp) = &interpolation.content else {
            return;
        };

        if has_filter_pipe(exp.content.as_str()) {
            ctx.error_with_help(
                ctx.t("vue/no-deprecated-filter.message"),
                &interpolation.loc,
                ctx.t("vue/no-deprecated-filter.help"),
            );
        }
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Filters only live in `v-bind` / `:` expression values. Other
        // directives (`v-on`, `v-if`, …) are out of scope for this rule, just
        // like eslint-plugin-vue.
        if directive.name.as_str() != "bind" {
            return;
        }

        let Some(ExpressionNode::Simple(exp)) = &directive.exp else {
            return;
        };

        if has_filter_pipe(exp.content.as_str()) {
            ctx.error_with_help(
                ctx.t("vue/no-deprecated-filter.message"),
                &exp.loc,
                ctx.t("vue/no-deprecated-filter.help"),
            );
        }
    }
}

/// Whether `expr` contains a Vue 2 filter pipe (`|` used as a filter operator).
///
/// Scans the raw expression text once, skipping over string literals, template
/// literals and regular-expression literals so a `|` inside any of them is never
/// mistaken for a filter. A doubled `||` is the logical-OR operator, never a
/// filter, so both bytes are consumed together. Any remaining single `|` is a
/// filter pipe — Vue 3 has no bitwise-OR meaning for template expressions that
/// would clash, and eslint-plugin-vue treats a lone `|` the same way.
fn has_filter_pipe(expr: &str) -> bool {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    // Tracks whether a `/` begins a regex literal (start of expression or right
    // after an operator) versus a division operator (right after a value).
    let mut prev_significant: u8 = 0;

    while i < len {
        let c = bytes[i];
        match c {
            b'\'' | b'"' => {
                i = skip_string(bytes, i, c);
                prev_significant = c;
            }
            b'`' => {
                i = skip_template(bytes, i);
                prev_significant = c;
            }
            b'/' => {
                // A `/` is a regex literal when nothing value-like precedes it;
                // otherwise it is division. Treat the regex body as opaque.
                if regex_allowed(prev_significant) {
                    i = skip_regex(bytes, i);
                } else {
                    i += 1;
                }
                prev_significant = b'/';
            }
            b'|' => {
                // `||` is logical OR — consume both bytes, not a filter.
                if i + 1 < len && bytes[i + 1] == b'|' {
                    i += 2;
                    prev_significant = b'|';
                    continue;
                }
                // A `|` preceded by `|` (the second half of `||`) was already
                // consumed above, so any `|` reaching here is a lone pipe.
                return true;
            }
            _ => {
                if !c.is_ascii_whitespace() {
                    prev_significant = c;
                }
                i += 1;
            }
        }
    }

    false
}

/// Returns whether a `/` at this position starts a regex literal, based on the
/// previous significant byte. A regex can begin at the start of the expression
/// or after an operator/opening bracket, but not after a value (identifier,
/// number, `)`, `]`, etc.) where `/` means division.
fn regex_allowed(prev: u8) -> bool {
    match prev {
        // No preceding token: start of expression.
        0 => true,
        // After a closing bracket / paren or a word char or `$`, `/` is division.
        b')' | b']' | b'}' => false,
        _ => !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$'),
    }
}

/// Advance past a `'`/`"` string literal starting at the opening quote `i`.
/// Returns the index just past the closing quote (or end of input).
fn skip_string(bytes: &[u8], i: usize, quote: u8) -> usize {
    let len = bytes.len();
    let mut j = i + 1;
    while j < len {
        match bytes[j] {
            b'\\' => j += 2,
            c if c == quote => return j + 1,
            _ => j += 1,
        }
    }
    len
}

/// Advance past a template literal starting at the backtick `i`. Nested `${ … }`
/// interpolations are skipped with brace counting so a `|` inside `${a|b}` is
/// also ignored (template-literal contents are opaque to filter detection).
fn skip_template(bytes: &[u8], i: usize) -> usize {
    let len = bytes.len();
    let mut j = i + 1;
    while j < len {
        match bytes[j] {
            b'\\' => j += 2,
            b'`' => return j + 1,
            b'$' if j + 1 < len && bytes[j + 1] == b'{' => {
                // Skip the balanced `${ … }` interpolation block.
                let mut depth = 1;
                j += 2;
                while j < len && depth > 0 {
                    match bytes[j] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
            }
            _ => j += 1,
        }
    }
    len
}

/// Advance past a regex literal starting at the `/` at `i`. Character classes
/// `[ … ]` are honoured so a `/` inside them does not end the literal early.
fn skip_regex(bytes: &[u8], i: usize) -> usize {
    let len = bytes.len();
    let mut j = i + 1;
    let mut in_class = false;
    while j < len {
        match bytes[j] {
            b'\\' => j += 2,
            b'[' => {
                in_class = true;
                j += 1;
            }
            b']' => {
                in_class = false;
                j += 1;
            }
            b'/' if !in_class => return j + 1,
            _ => j += 1,
        }
    }
    len
}

#[cfg(test)]
mod tests {
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
}
