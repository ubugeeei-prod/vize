//! script/require-symbol-provide
//!
//! Recommend using Symbol as injection key for provide/inject.
//!
//! Using Symbol keys for provide/inject avoids naming collisions and makes
//! the dependency injection more explicit and type-safe.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // String keys can collide
//! provide('user', user)
//! const user = inject('user')
//!
//! // Magic strings are error-prone
//! provide('theme', { dark: true })
//! ```
//!
//! ### Valid
//! ```ts
//! // Define injection key with Symbol
//! export const UserKey: InjectionKey<User> = Symbol('user')
//!
//! // Provide with Symbol
//! provide(UserKey, user)
//!
//! // Inject with Symbol
//! const user = inject(UserKey)
//! ```

use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Argument, CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-symbol-provide",
    description: "Recommend using Symbol as injection key for provide/inject",
    default_severity: Severity::Warning,
};

/// Require Symbol for provide/inject keys
pub struct RequireSymbolProvide;

impl ScriptRule for RequireSymbolProvide {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let mut visitor = RequireSymbolProvideVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct RequireSymbolProvideVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for RequireSymbolProvideVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(span) = string_key_provide_inject_span(it) {
            let start = self.offset as u32 + span.start;
            let end = self.offset as u32 + span.end;
            self.result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Consider using a Symbol key instead of a string literal",
                    start,
                    end,
                )
                .with_help(
                    "Define an InjectionKey with Symbol: \
                     `export const MyKey: InjectionKey<MyType> = Symbol('myKey')`",
                ),
            );
        }

        walk_call_expression(self, it);
    }
}

fn string_key_provide_inject_span(call: &CallExpression<'_>) -> Option<Span> {
    if !is_static_string_argument(call.arguments.first()?) {
        return None;
    }
    match &call.callee {
        Expression::Identifier(identifier)
            if matches!(identifier.name.as_str(), "provide" | "inject") =>
        {
            Some(Span::new(identifier.span.start, identifier.span.end + 1))
        }
        _ => None,
    }
}

fn is_static_string_argument(argument: &Argument<'_>) -> bool {
    match argument {
        Argument::StringLiteral(_) => true,
        Argument::TemplateLiteral(template) => template.single_quasi().is_some(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::RequireSymbolProvide;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(RequireSymbolProvide));
        linter
    }

    #[test]
    fn test_valid_symbol_provide() {
        let linter = create_linter();
        let result = linter.lint("provide(UserKey, user)", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_symbol_inject() {
        let linter = create_linter();
        let result = linter.lint("const user = inject(UserKey)", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_string_provide() {
        let linter = create_linter();
        let result = linter.lint("provide('user', userData)", 0);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_string_inject() {
        let linter = create_linter();
        let result = linter.lint("const user = inject('user')", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_template_literal() {
        let linter = create_linter();
        let result = linter.lint("provide(`theme`, theme)", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_no_provide_inject() {
        let linter = create_linter();
        let result = linter.lint("const x = ref(0)", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_ignores_string_literal_source() {
        let linter = create_linter();
        let result = linter.lint(r#"const source = "provide('user', userData)""#, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_ignores_comment_source() {
        let linter = create_linter();
        let result = linter.lint("// inject('user')\nconst user = inject(UserKey)", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_ignores_member_named_provide() {
        let linter = create_linter();
        let result = linter.lint("container.provide('user', userData)", 0);
        assert_eq!(result.warning_count, 0);
    }
}
