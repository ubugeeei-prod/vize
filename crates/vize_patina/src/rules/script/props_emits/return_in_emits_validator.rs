//! script/return-in-emits-validator
//!
//! Require a return value in every Options API emits validator.
//!
//! The object form of the `emits` option maps each event name to a validator
//! function: `emits: { submit(payload) { return !!payload } }`. Vue calls the
//! validator and treats its boolean return as "is this emit valid?". A
//! validator that can finish without returning a value yields `undefined`
//! (falsy), so the emit is reported invalid on that path — almost always a
//! forgotten `return`.
//!
//! This flags any emits validator (method shorthand or `name: (p) => {...}`
//! arrow with a block body) whose body has no value-returning `return`. A
//! concise-body arrow (`(p) => expr`) always returns its expression and is never
//! flagged; a `return` inside a nested function is that function's own and does
//! not count. Only the Options API `emits` option is covered (the
//! `<script setup>` `defineEmits` validator object is the same shape and is
//! handled too when it resolves to an object literal).
//!
//! Port of [`vue/return-in-emits-validator`](https://eslint.vuejs.org/rules/return-in-emits-validator.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   emits: {
//!     submit(payload) {
//!       if (!payload.email) {
//!         // missing return
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   emits: {
//!     submit(payload) {
//!       return !!payload.email
//!     }
//!   }
//! }
//! ```

use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, Function, ObjectExpression, ObjectPropertyKind, Program,
    ReturnStatement, Statement,
};
use oxc_ast_visit::Visit;
use oxc_span::Span;

use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use super::emits_source::resolve_emits_object;
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/return-in-emits-validator",
    description: "Require a return value in every Options API emits validator",
    default_severity: Severity::Error,
};

/// Require a return value in every emits validator function.
pub struct ReturnInEmitsValidator;

impl ScriptRule for ReturnInEmitsValidator {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let Some(emits) = resolve_emits_object(program) else {
            return;
        };
        check_emits_object(emits, offset, result);
    }
}

/// A validator whose block body must contain a value-returning `return`.
enum Validator<'a> {
    Function(&'a Function<'a>),
    Arrow(&'a ArrowFunctionExpression<'a>),
}

fn check_emits_object(emits: &ObjectExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    for property in &emits.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if let Some(validator) = validator_from_value(&property.value) {
            check_validator(validator, offset, result);
        }
    }
}

/// Interpret an emits-entry value as a validator function. A `null` value
/// (`submit: null`, "no validation") and any non-function value are skipped.
fn validator_from_value<'a>(value: &'a Expression<'a>) -> Option<Validator<'a>> {
    match value {
        Expression::FunctionExpression(function) => Some(Validator::Function(function)),
        Expression::ArrowFunctionExpression(arrow) => Some(Validator::Arrow(arrow)),
        _ => None,
    }
}

/// Report the validator unless its block body has a value-returning `return`.
fn check_validator(validator: Validator<'_>, offset: usize, result: &mut ScriptLintResult) {
    let (span, statements): (Span, _) = match validator {
        Validator::Function(function) => match function.body.as_ref() {
            Some(body) => (function.span, &body.statements),
            None => return,
        },
        // A concise-body arrow (`(p) => expr`) implicitly returns its
        // expression; only block bodies can be missing a return.
        Validator::Arrow(arrow) if arrow.expression => return,
        Validator::Arrow(arrow) => (arrow.span, &arrow.body.statements),
    };

    if body_returns_value(statements) {
        return;
    }

    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::error(
            META.name,
            "Expected to return a boolean value in this emits validator.",
            start,
            end,
        )
        .with_label("emits validator without a return value", start, end)
        .with_help(
            "An emits validator must return whether the payload is valid. Add a `return` that \
             yields a boolean on every path through the validator.",
        ),
    );
}

/// Whether the statement list contains a `return <expr>` reachable as the
/// validator's own return (not inside a nested function scope).
fn body_returns_value(statements: &[Statement<'_>]) -> bool {
    let mut finder = ReturnFinder { found: false };
    for statement in statements {
        finder.visit_statement(statement);
        if finder.found {
            return true;
        }
    }
    false
}

/// Walks a validator body for a value-returning `return`. Nested function and
/// arrow scopes are not traversed: their `return` is their own.
struct ReturnFinder {
    found: bool,
}

impl<'a> Visit<'a> for ReturnFinder {
    fn visit_function(&mut self, _it: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        // `return;` produces no value; only `return <expr>` counts.
        if it.argument.is_some() {
            self.found = true;
        }
    }
}

#[cfg(test)]
mod tests;
