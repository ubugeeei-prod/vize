//! vue/this-in-template
//!
//! Disallow `this.` in template expressions.
//!
//! Vue resolves template identifiers against the component instance
//! automatically, so writing `this.` in a template expression is unnecessary
//! and is frequently a mistake (for example, copy-pasting code out of a method
//! body). This rule flags member access on `this` inside both interpolations
//! and directive expressions.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div>{{ this.message }}</div>
//! <div :class="this.className"></div>
//! <button @click="this.handleClick()"></button>
//! ```
//!
//! ### Valid
//! ```vue
//! <div>{{ message }}</div>
//! <div :class="className"></div>
//! <button @click="handleClick()"></button>
//! <div>{{ 'this.is.a.string' }}</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/this-in-template",
    description: "Disallow `this.` in template expressions",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow `this.` in template expressions.
pub struct ThisInTemplate;

impl ThisInTemplate {
    /// Pragmatic scan for a `this.` member access at an identifier boundary.
    ///
    /// Returns `true` when the expression references a member access on `this`
    /// (e.g. `this.foo`). The scan skips string literals so the literal text
    /// `this.` inside `'this.foo'` is not flagged, and it requires `this` to sit
    /// on an identifier boundary so substrings like `myThis` or `things` do not
    /// match.
    fn has_this_member_access(expr: &str) -> bool {
        let bytes = expr.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            let b = bytes[i];

            // Skip string literals so `this.` inside a string is never flagged.
            if b == b'\'' || b == b'"' || b == b'`' {
                let quote = b;
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' {
                        i += 2; // skip the escaped character
                        continue;
                    }
                    if bytes[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            // Identifier start: read the whole identifier so we only test `this`
            // when it stands on its own boundary (not part of `things`/`myThis`).
            if is_ident_start(b) {
                let start = i;
                i += 1;
                while i < len && is_ident_continue(bytes[i]) {
                    i += 1;
                }
                let ident = &expr[start..i];

                if ident == "this" {
                    // Look past whitespace for a `.` that begins a member access
                    // whose property is an identifier (e.g. `this.foo`). This
                    // intentionally ignores bare `this`, `this[...]`, and a
                    // trailing `this.` with nothing after it.
                    let mut j = i;
                    while j < len && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if j < len && bytes[j] == b'.' {
                        let mut k = j + 1;
                        while k < len && bytes[k].is_ascii_whitespace() {
                            k += 1;
                        }
                        if k < len && is_ident_start(bytes[k]) {
                            return true;
                        }
                    }
                }
                continue;
            }

            i += 1;
        }

        false
    }

    fn check_expression(ctx: &mut LintContext<'_>, exp: &ExpressionNode<'_>) {
        let content = match exp {
            ExpressionNode::Simple(s) => s.content.as_str(),
            ExpressionNode::Compound(_) => return,
        };

        if Self::has_this_member_access(content) {
            ctx.warn_with_help(
                ctx.t("vue/this-in-template.message"),
                exp.loc(),
                ctx.t("vue/this-in-template.help"),
            );
        }
    }
}

/// Whether `b` may start a JavaScript identifier (ASCII subset).
#[inline]
fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b'$'
}

/// Whether `b` may continue a JavaScript identifier (ASCII subset).
#[inline]
fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

impl Rule for ThisInTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        Self::check_expression(ctx, &interpolation.content);
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if let Some(exp) = &directive.exp {
            Self::check_expression(ctx, exp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThisInTemplate;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ThisInTemplate));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_interpolation_without_this() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ message }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_directive_without_this() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="className"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_this_inside_string_literal() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ 'this.is.a.string' }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_identifier_containing_this_substring() {
        let linter = create_linter();
        // `things` and `myThis` contain "this" but are not the `this` keyword.
        let result = linter.lint_template(r#"<div>{{ things.length + myThis }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_this_in_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ this.message }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_this_in_bind_directive() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="this.className"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_this_in_event_handler() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="this.handleClick()"></button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_this_with_whitespace_before_dot() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ this .message }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
