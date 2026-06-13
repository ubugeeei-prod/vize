//! script/valid-next-tick
//!
//! Require the result of a `nextTick()` call to be used.
//!
//! `nextTick()` (the function imported from `vue`) and `this.$nextTick()` return
//! a promise that resolves after the next DOM flush. A bare call whose result is
//! ignored — not `await`ed, not chained with `.then(...)`, and given no callback
//! argument — schedules nothing observable and is almost always a mistake: the
//! code that was meant to run after the flush never does.
//!
//! A call is considered *used* when any of the following holds:
//! - it is passed a callback argument: `nextTick(() => {})`;
//! - its result is `await`ed: `await nextTick()`;
//! - its result is chained: `nextTick().then(() => {})`;
//! - its result is assigned/returned/otherwise consumed (it is then not the whole
//!   expression of an `ExpressionStatement`).
//!
//! Only a bare `nextTick()` / `this.$nextTick()` standing alone as an expression
//! statement with no callback argument is flagged.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { nextTick } from 'vue'
//!
//! nextTick()          // result ignored
//! this.$nextTick()    // result ignored
//! ```
//!
//! ### Valid
//! ```ts
//! import { nextTick } from 'vue'
//!
//! await nextTick()
//! nextTick().then(() => focusInput())
//! nextTick(() => focusInput())
//! this.$nextTick(() => focusInput())
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier, Program, Statement,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_import_declaration, walk_statement},
};
use oxc_span::Span;
use vize_carton::{CompactString, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/valid-next-tick",
    description: "Require the result of a nextTick() call to be awaited, chained, or given a callback",
    default_severity: Severity::Warning,
};

const MESSAGE: &str = "The result of this nextTick() call is ignored.";
const HELP: &str = "nextTick() returns a promise that resolves after the DOM updates. \
     `await` it, chain `.then(...)`, or pass a callback (`nextTick(() => {})`); a bare call does nothing useful.";

/// Require the result of a `nextTick()` call to be used.
pub struct ValidNextTick;

impl ScriptRule for ValidNextTick {
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
        let mut visitor = ValidNextTickVisitor {
            offset,
            result,
            imported_aliases: FxHashSet::default(),
        };
        visitor.visit_program(program);
    }
}

struct ValidNextTickVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    imported_aliases: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for ValidNextTickVisitor<'_> {
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.source.value.as_str() == "vue"
            && let Some(specifiers) = &it.specifiers
        {
            for specifier in specifiers {
                let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                    continue;
                };
                if specifier.imported.name().as_str() == "nextTick" {
                    self.imported_aliases
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
            }
        }
        walk_import_declaration(self, it);
    }

    fn visit_statement(&mut self, it: &Statement<'a>) {
        // A bare `nextTick()` whose value is discarded surfaces as the whole
        // expression of an `ExpressionStatement`. When it is awaited, chained,
        // or assigned, the statement's expression is something else (an
        // `AwaitExpression`, a `.then(...)` member call, a declaration, ...), so
        // those forms are never reached here.
        if let Statement::ExpressionStatement(statement) = it
            && let Some(span) = bare_next_tick_call(&statement.expression, &self.imported_aliases)
        {
            let start = self.offset as u32 + span.start;
            let end = self.offset as u32 + span.end;
            self.result.add_diagnostic(
                LintDiagnostic::warn(META.name, MESSAGE, start, end)
                    .with_label("ignored nextTick() result", start, end)
                    .with_help(HELP),
            );
        }
        walk_statement(self, it);
    }
}

/// If `expression` (a statement's whole expression) is a bare `nextTick()` /
/// `this.$nextTick()` call with no callback argument, return its span.
fn bare_next_tick_call(
    expression: &Expression<'_>,
    imported_aliases: &FxHashSet<CompactString>,
) -> Option<Span> {
    let call = unwrap_to_call(expression)?;
    if !call.arguments.is_empty() {
        // A callback argument (or any argument) means the caller is scheduling
        // work; that form is allowed.
        return None;
    }
    next_tick_callee_span(call, imported_aliases)
}

/// Strip parentheses / `void` to reach a `CallExpression`, if the expression is
/// (or wraps) one. A `void nextTick()` still discards the result.
fn unwrap_to_call<'a, 'b>(expression: &'b Expression<'a>) -> Option<&'b CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(paren) => unwrap_to_call(&paren.expression),
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_syntax::operator::UnaryOperator::Void =>
        {
            unwrap_to_call(&unary.argument)
        }
        _ => None,
    }
}

/// The span of the callee when this call is `nextTick(...)` (a vue import alias)
/// or `<expr>.$nextTick(...)`.
fn next_tick_callee_span(
    call: &CallExpression<'_>,
    imported_aliases: &FxHashSet<CompactString>,
) -> Option<Span> {
    match &call.callee {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if name == "nextTick" || imported_aliases.contains(name) {
                Some(call.span)
            } else {
                None
            }
        }
        callee if callee.is_member_expression() => callee
            .as_member_expression()
            .and_then(|member| member.static_property_name())
            .filter(|name| *name == "$nextTick")
            .map(|_| call.span),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ValidNextTick;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ValidNextTick));
        linter
    }

    #[test]
    fn test_valid_awaited() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'\nasync function f() { await nextTick() }",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_then_chained() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'\nnextTick().then(() => focusInput())",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_callback_argument() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'\nnextTick(() => focusInput())",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_instance_callback() {
        let result = create_linter().lint("this.$nextTick(() => focusInput())", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_assigned_result() {
        // Assigning the promise is a use; this is a declaration, not a bare
        // expression statement.
        let result =
            create_linter().lint("import { nextTick } from 'vue'\nconst p = nextTick()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_returned_result() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'\nfunction f() { return nextTick() }",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_bare_call() {
        let result = create_linter().lint("import { nextTick } from 'vue'\nnextTick()", 0);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_bare_instance_call() {
        let result = create_linter().lint("this.$nextTick()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_aliased_bare_call() {
        let result = create_linter().lint("import { nextTick as tick } from 'vue'\ntick()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_void_bare_call() {
        let result = create_linter().lint("import { nextTick } from 'vue'\nvoid nextTick()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_unrelated_identifier_not_flagged() {
        // `nextTick` not imported from vue and not `$nextTick`: a local function
        // named `tick` is unrelated.
        let result = create_linter().lint("tick()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_offset_applied() {
        let source = "this.$nextTick()";
        let result = create_linter().lint(source, 50);
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].start, 50);
    }
}
