//! petite-vue/valid-v-scope
//!
//! Require `v-scope` to bind a (possibly empty) object literal.
//!
//! petite-vue's `v-scope` directive introduces a reactive scope for the
//! element's subtree from the **object literal** it is given:
//! `v-scope="{ count: 0 }"`. A non-object value (`v-scope="count"`,
//! `v-scope="foo()"`, `v-scope="a + b"`, `v-scope="123"`) does not declare any
//! scope keys and is almost always a bug: petite-vue evaluates the expression
//! but no top-level bindings become available to the template.
//!
//! A bare `v-scope` with no value (`v-scope=""`) is allowed: it simply marks an
//! element as a petite-vue root without introducing bindings.
//!
//! This rule only runs on documents detected as petite-vue (see
//! `ctx.is_petite_vue()`); it has zero effect on normal Vue SFC linting.
//!
//! ## Examples
//!
//! ### Invalid (petite-vue document)
//! ```html
//! <div v-scope="count"></div>
//! <div v-scope="foo()"></div>
//! <div v-scope="a + b"></div>
//! <div v-scope="123"></div>
//! ```
//!
//! ### Valid (petite-vue document)
//! ```html
//! <div v-scope></div>
//! <div v-scope="{}"></div>
//! <div v-scope="{ count: 0 }"></div>
//! <div v-scope="({ count: 0 })"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "petite-vue/valid-v-scope",
    description: "Require v-scope to bind an object literal",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Require petite-vue `v-scope` to bind an object literal.
#[derive(Default)]
pub struct ValidVScope;

impl Rule for ValidVScope {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only active for petite-vue documents; normal Vue SFCs are untouched.
        if !ctx.is_petite_vue() {
            return;
        }

        if directive.name.as_str() != "scope" {
            return;
        }

        // A bare `v-scope` (no value) is a valid scope root; nothing to check.
        let Some(ExpressionNode::Simple(exp)) = directive.exp.as_ref() else {
            return;
        };
        let value = exp.content.trim();
        if value.is_empty() {
            return;
        }

        if is_object_literal(value) {
            return;
        }

        ctx.error_with_help(
            ctx.t("petite-vue/valid-v-scope.message"),
            &exp.loc,
            ctx.t("petite-vue/valid-v-scope.help"),
        );
    }
}

/// Whether `expr` parses to a (possibly parenthesized) object literal.
///
/// Mirrors croquis' v-scope parsing: wrap the expression as the initializer of
/// a declaration so a leading `{` parses as an object literal rather than a
/// block statement, then inspect the parsed initializer.
fn is_object_literal(expr: &str) -> bool {
    // Wrap as the initializer of a declaration so the object literal parses as
    // an expression (a bare leading `{` would otherwise be a block statement).
    #[allow(clippy::disallowed_macros)]
    let wrapped = format!("const __vize_scope = {expr}");

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = Parser::new(&allocator, &wrapped, source_type).parse();

    // A syntactically broken value is not a valid object literal.
    if !ret.errors.is_empty() {
        return false;
    }

    let Some(Statement::VariableDeclaration(var_decl)) = ret.program.body.first() else {
        return false;
    };
    let Some(init) = var_decl.declarations.first().and_then(|d| d.init.as_ref()) else {
        return false;
    };

    // Unwrap any parentheses the source carried (`v-scope="({ ... })"`).
    let mut expr_node = init;
    while let Expression::ParenthesizedExpression(paren) = expr_node {
        expr_node = &paren.expression;
    }

    matches!(expr_node, Expression::ObjectExpression(_))
}

#[cfg(test)]
mod tests {
    use super::ValidVScope;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVScope));
        Linter::with_registry(registry)
    }

    /// Wrap markup in a petite-vue document so `ctx.is_petite_vue()` is true.
    /// The outer `v-scope` is itself a valid object literal so it is never the
    /// diagnostic under test.
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

    /// Wrap markup in a plain (non-petite) Vue-loaded document.
    fn vue_doc(markup: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
  <body>
    <div>
{markup}
    </div>
    <script src="https://unpkg.com/vue"></script>
  </body>
</html>"#
        )
    }

    #[test]
    fn reports_identifier_value_in_petite_vue() {
        let linter = create_linter();
        let result = linter
            .lint_standalone_html(&petite_doc(r#"<div v-scope="count"></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_call_value_in_petite_vue() {
        let linter = create_linter();
        let result = linter
            .lint_standalone_html(&petite_doc(r#"<div v-scope="foo()"></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_binary_value_in_petite_vue() {
        let linter = create_linter();
        let result = linter
            .lint_standalone_html(&petite_doc(r#"<div v-scope="a + b"></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_numeric_value_in_petite_vue() {
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<div v-scope="123"></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_array_value_in_petite_vue() {
        let linter = create_linter();
        let result = linter
            .lint_standalone_html(&petite_doc(r#"<div v-scope="[1, 2]"></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_malformed_value_in_petite_vue() {
        let linter = create_linter();
        let result = linter
            .lint_standalone_html(&petite_doc(r#"<div v-scope="{ a: }"></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_empty_object_in_petite_vue() {
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<div v-scope="{}"></div>"#), "index.html");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_object_literal_in_petite_vue() {
        let linter = create_linter();
        let result = linter.lint_standalone_html(
            &petite_doc(r#"<div v-scope="{ count: 0, msg: 'x' }"></div>"#),
            "index.html",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_parenthesized_object_in_petite_vue() {
        let linter = create_linter();
        let result = linter.lint_standalone_html(
            &petite_doc(r#"<div v-scope="({ count: 0 })"></div>"#),
            "index.html",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_bare_v_scope_in_petite_vue() {
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<div v-scope></div>"#), "index.html");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_empty_string_v_scope_in_petite_vue() {
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<div v-scope=""></div>"#), "index.html");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_non_object_v_scope_in_non_petite_document() {
        let linter = create_linter();
        // The same non-object v-scope in a plain Vue document must not be
        // flagged: this rule is petite-vue-only.
        let result =
            linter.lint_standalone_html(&vue_doc(r#"<div v-scope="count"></div>"#), "index.html");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_non_object_v_scope_in_sfc_template() {
        let linter = create_linter();
        // A bare SFC template fragment is not a petite-vue document.
        let result = linter.lint_template(r#"<div v-scope="count"></div>"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }
}
