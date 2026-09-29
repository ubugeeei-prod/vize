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
    let type_spans = arrow_param_type_spans(expr);
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

fn is_arrow_at(bytes: &[u8], index: usize) -> bool {
    bytes.get(index) == Some(&b'=') && bytes.get(index + 1) == Some(&b'>')
}

/// Index of `=>` after a return-type colon, if the type actually ends in an arrow.
fn find_arrow_after_type(bytes: &[u8], mut i: usize) -> Option<usize> {
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut angle = 0i32;
    while i < bytes.len() {
        let Some(&byte) = bytes.get(i) else {
            break;
        };
        match byte {
            b'\'' | b'"' => {
                i = skip_string(bytes, i, byte);
                continue;
            }
            b'`' => {
                i = skip_template(bytes, i);
                continue;
            }
            b'(' => paren += 1,
            b')' => {
                if paren == 0 {
                    return None;
                }
                paren -= 1;
            }
            b'[' => bracket += 1,
            b']' if bracket > 0 => bracket -= 1,
            b'{' => brace += 1,
            b'}' if brace > 0 => brace -= 1,
            b'<' => angle += 1,
            b'>' if angle > 0 => angle -= 1,
            b'=' if paren == 0
                && bracket == 0
                && brace == 0
                && angle == 0
                && bytes.get(i + 1) == Some(&b'>') =>
            {
                return Some(i);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Record each top-level `:` annotation inside an arrow parameter list.
fn push_param_type_spans(bytes: &[u8], mut i: usize, end: usize, spans: &mut Vec<(usize, usize)>) {
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut angle = 0i32;
    while i < end {
        let Some(&byte) = bytes.get(i) else {
            break;
        };
        match byte {
            b'\'' | b'"' => {
                i = skip_string(bytes, i, byte).min(end);
                continue;
            }
            b'`' => {
                i = skip_template(bytes, i).min(end);
                continue;
            }
            b':' if paren == 0 && bracket == 0 && brace == 0 && angle == 0 => {
                let type_start = i + 1;
                i += 1;
                let mut type_paren = 0i32;
                let mut type_bracket = 0i32;
                let mut type_brace = 0i32;
                let mut type_angle = 0i32;
                while i < end {
                    let Some(&byte) = bytes.get(i) else {
                        break;
                    };
                    match byte {
                        b'\'' | b'"' => {
                            i = skip_string(bytes, i, byte).min(end);
                            continue;
                        }
                        b'`' => {
                            i = skip_template(bytes, i).min(end);
                            continue;
                        }
                        b'(' => type_paren += 1,
                        b')' => {
                            if type_paren == 0 {
                                break;
                            }
                            type_paren -= 1;
                        }
                        b'[' => type_bracket += 1,
                        b']' if type_bracket > 0 => type_bracket -= 1,
                        b'{' => type_brace += 1,
                        b'}' if type_brace > 0 => type_brace -= 1,
                        b'<' => type_angle += 1,
                        b'>' if type_angle > 0 => type_angle -= 1,
                        b',' | b'='
                            if type_paren == 0
                                && type_bracket == 0
                                && type_brace == 0
                                && type_angle == 0 =>
                        {
                            break;
                        }
                        _ => {}
                    }
                    i += 1;
                }
                spans.push((type_start, i));
                continue;
            }
            b'(' => paren += 1,
            b')' if paren > 0 => paren -= 1,
            b'[' => bracket += 1,
            b']' if bracket > 0 => bracket -= 1,
            b'{' => brace += 1,
            b'}' if brace > 0 => brace -= 1,
            b'<' => angle += 1,
            b'>' if angle > 0 => angle -= 1,
            _ => {}
        }
        i += 1;
    }
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
    while let Some(&byte) = bytes.get(j) {
        match byte {
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
    while let Some(&byte) = bytes.get(j) {
        match byte {
            b'\\' => j += 2,
            b'`' => return j + 1,
            b'$' if bytes.get(j + 1) == Some(&b'{') => {
                // Skip the balanced `${ … }` interpolation block.
                let mut depth = 1;
                j += 2;
                while let Some(&byte) = bytes.get(j)
                    && depth > 0
                {
                    match byte {
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
    while let Some(&byte) = bytes.get(j) {
        match byte {
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
mod tests;
