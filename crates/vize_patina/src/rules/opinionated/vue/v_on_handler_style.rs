//! vue/v-on-handler-style
//!
//! Enforce writing `v-on` handlers as a method reference or an inline function
//! rather than an inline statement.
//!
//! A handler such as `@click="handler"` (method reference) or
//! `@click="() => count++"` (inline function) keeps the template declarative
//! and the logic testable. An inline statement such as `@click="count++"`
//! mixes imperative code into the template and is harder to reuse or test.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <button @click="count++"></button>
//! <button @click="doThis(); doThat()"></button>
//! <button @click="foo = bar"></button>
//! ```
//!
//! ### Valid
//! ```vue
//! <button @click="handler"></button>
//! <button @click="foo.bar"></button>
//! <button @click="() => count++"></button>
//! <button @click="function () { count++ }"></button>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/v-on-handler-style",
    description: "Enforce writing v-on handlers as a method reference or an inline function",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Enforce v-on handler style (method reference or inline function).
pub struct VOnHandlerStyle;

impl Rule for VOnHandlerStyle {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "on" {
            return;
        }

        // Only a static event argument (`@click`), not `v-on="obj"` or
        // a dynamic argument (`@[event]`).
        let Some(ExpressionNode::Simple(_)) = &directive.arg else {
            return;
        };

        // The handler expression itself must be a simple expression.
        let Some(ExpressionNode::Simple(exp)) = &directive.exp else {
            return;
        };

        let expr = exp.content.trim();
        if expr.is_empty() {
            return;
        }

        // Allowed: a bare method reference (`foo` / `foo.bar`) or an inline
        // function (`() => ...` / `function ...`). Everything else is an
        // inline statement.
        if is_method_reference(expr) || is_inline_function(expr) {
            return;
        }

        ctx.warn_with_help(
            ctx.t("vue/v-on-handler-style.message"),
            &directive.loc,
            ctx.t("vue/v-on-handler-style.help"),
        );
    }
}

/// Whether `expr` is a bare method reference: a single identifier or a
/// member-access path such as `foo` or `foo.bar`, with no call parentheses
/// or operators. Pragmatic scan: every character must be a letter, digit,
/// `_`, `$`, or `.`, it must not start or end with `.`, and it must not
/// contain an empty path segment (`foo..bar`).
fn is_method_reference(expr: &str) -> bool {
    if expr.is_empty() {
        return false;
    }
    if expr.starts_with('.') || expr.ends_with('.') {
        return false;
    }
    if expr.contains("..") {
        return false;
    }
    let mut chars = expr.chars();
    // A path segment must not start with a digit; check the leading char of
    // the whole expression and of each segment.
    let mut segment_start = true;
    for ch in chars.by_ref() {
        match ch {
            '.' => segment_start = true,
            'a'..='z' | 'A'..='Z' | '_' | '$' => segment_start = false,
            '0'..='9' => {
                if segment_start {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}

/// Whether `expr` is an inline function: it starts with `function` (as a
/// keyword) or has an arrow-function shape (contains `=>`).
fn is_inline_function(expr: &str) -> bool {
    if expr.contains("=>") {
        return true;
    }
    if let Some(rest) = expr.strip_prefix("function") {
        // `function`, `function foo`, `function(` — but not `functionish`.
        return rest
            .chars()
            .next()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '$'));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::VOnHandlerStyle;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VOnHandlerStyle));
        Linter::with_registry(registry)
    }

    #[test]
    fn allows_method_reference() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="handler"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_member_access_reference() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="foo.bar"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_arrow_function() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="() => count++"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_arrow_function_with_arg() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button @click="(e) => onClick(e)"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_function_expression() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="function () { count++ }"></button>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn warns_on_inline_increment() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="count++"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn warns_on_inline_call() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="foo()"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_on_assignment() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="a = b"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_on_multiple_statements() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="doThis(); doThat()"></button>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn ignores_object_syntax() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button v-on="{ click: handler }"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn ignores_handler_without_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<form @submit.prevent></form>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn warns_on_member_call() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="foo.bar()"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }
}
