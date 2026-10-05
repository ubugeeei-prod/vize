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
//! <Draggable :item-key="(item: A | B | C) => item.id" />
//! ```

mod assertion_types;
mod scan;
use scan::{
    find_arrow_after_type, is_arrow_at, push_param_type_spans, regex_allowed, skip_regex,
    skip_string, skip_template,
};

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-filter",
    description: "Disallow deprecated Vue 2 filter syntax using the pipe operator",
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

        if has_filter_pipe(exp.content) {
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
        if directive.name != "bind" {
            return;
        }

        let Some(ExpressionNode::Simple(exp)) = &directive.exp else {
            return;
        };

        if has_filter_pipe(exp.content) {
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
/// filter, so both bytes are consumed together. A `|` inside an arrow-function
/// parameter or return type (`(item: A | B | C) => item.id`) is a TypeScript
/// union, not a filter. Any remaining single `|` is a filter pipe — Vue 3 has
/// no bitwise-OR meaning for template expressions that would clash, and
/// eslint-plugin-vue treats a lone `|` the same way.
fn has_filter_pipe(expr: &str) -> bool {
    let mut type_spans = arrow_param_type_spans(expr);
    let mut checked_assertion_types = false;
    let bytes = expr.as_bytes();
    let mut i = 0;
    // Tracks whether a `/` begins a regex literal (start of expression or right
    // after an operator) versus a division operator (right after a value).
    let mut prev_significant: u8 = 0;

    while let Some(&c) = bytes.get(i) {
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
                if bytes.get(i + 1) == Some(&b'|') {
                    i += 2;
                    prev_significant = b'|';
                    continue;
                }
                // A `|` inside `(item: A | B) => …` is a type union, not a filter.
                if type_spans.iter().any(|&(start, end)| i >= start && i < end) {
                    i += 1;
                    prev_significant = b'|';
                    continue;
                }
                // Only a remaining candidate with assertion keywords needs
                // the existing TS expression parser. Its exact union spans
                // cannot exempt a runtime pipe outside the type grammar.
                if !checked_assertion_types {
                    checked_assertion_types = true;
                    assertion_types::extend_union_spans(expr, &mut type_spans);
                    if type_spans.iter().any(|&(start, end)| i >= start && i < end) {
                        i += 1;
                        prev_significant = b'|';
                        continue;
                    }
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

/// Spans of TypeScript annotations in arrow parameters and return types.
///
/// `(item: A | B | C) => item.id` and `(item): A | B => item` both put `|`
/// in type grammar. The body after `=>` stays an expression, so a filter
/// there is still visible.
fn arrow_param_type_spans(expr: &str) -> Vec<(usize, usize)> {
    let bytes = expr.as_bytes();
    let mut spans = Vec::new();
    let mut paren_stack = Vec::new();
    let mut i = 0;
    let mut prev_significant = 0u8;

    while i < bytes.len() {
        let Some(&c) = bytes.get(i) else {
            break;
        };
        match c {
            b'\'' | b'"' => {
                i = skip_string(bytes, i, c);
                prev_significant = c;
            }
            b'`' => {
                i = skip_template(bytes, i);
                prev_significant = c;
            }
            b'/' if regex_allowed(prev_significant) => {
                i = skip_regex(bytes, i);
                prev_significant = b'/';
            }
            b'(' => {
                paren_stack.push(i);
                prev_significant = c;
                i += 1;
            }
            b')' => {
                let open = paren_stack.pop();
                let mut j = i + 1;
                while j < bytes.len() && bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
                    j += 1;
                }
                if bytes.get(j) == Some(&b':') {
                    let type_start = j + 1;
                    if let Some(arrow) = find_arrow_after_type(bytes, type_start) {
                        if let Some(open) = open {
                            push_param_type_spans(bytes, open + 1, i, &mut spans);
                        }
                        spans.push((type_start, arrow));
                    }
                } else if is_arrow_at(bytes, j)
                    && let Some(open) = open
                {
                    push_param_type_spans(bytes, open + 1, i, &mut spans);
                }
                prev_significant = c;
                i += 1;
            }
            _ => {
                if !c.is_ascii_whitespace() {
                    prev_significant = c;
                }
                i += 1;
            }
        }
    }

    spans
}

#[cfg(test)]
mod tests;
