//! script/no-multiple-slot-args
//!
//! Disallow passing more than one argument to a scoped-slot function call.
//!
//! A scoped slot is rendered by calling the slot function and passing it a
//! single "slot props" object: `slots.default({ item })`. Vue only forwards the
//! **first** argument to the slot's binding (`v-slot="slotProps"` /
//! `v-slot="{ item }"`); any further positional arguments are silently dropped.
//! Passing several arguments is therefore always a mistake — the extra values
//! never reach the slot. The fix is to wrap them in one object:
//! `slots.default({ a, b })`.
//!
//! ## Scope
//!
//! This is the script-side analysis (`<script setup>` / render functions). It
//! reports a call expression whose callee is a member access on a recognised
//! slots source and that is passed more than one argument, or a spread argument.
//! Recognised slots sources are:
//!
//! - a bare `slots` / `$slots` / `$scopedSlots` identifier
//!   (`slots.default(a, b)`, `$slots.foo(a, b)`),
//! - `this.$slots` / `this.$scopedSlots` (Options API render function:
//!   `this.$scopedSlots.foo(a, b)`),
//! - the result of a `useSlots()` call (`useSlots().foo(a, b)`).
//!
//! The slot name itself can be a static identifier (`slots.default(...)`) or a
//! string-literal computed key (`slots['default'](...)`). Optional chaining on
//! the access or the call (`slots.default?.(a, b)`) is covered.
//!
//! Matching is deliberately conservative: the callee must root in one of those
//! slots sources, so an arbitrary two-argument method call on an unrelated
//! object is never reported. Port of
//! [`vue/no-multiple-slot-args`](https://eslint.vuejs.org/rules/no-multiple-slot-args.html),
//! extended to the Composition-API `slots` / `useSlots()` forms.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! slots.default(foo, bar)
//! $slots.header(a, b)
//! this.$scopedSlots.item(x, y)
//! useSlots().default(a, b)
//! slots.default(...args)
//! ```
//!
//! ### Valid
//! ```ts
//! slots.default({ foo, bar })
//! slots.default(slotProps)
//! slots.default()
//! ```

use oxc_ast::ast::{Argument, CallExpression, Expression, MemberExpression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-multiple-slot-args",
    description: "Disallow passing more than one argument to a scoped-slot function call",
    default_severity: Severity::Warning,
};

/// Disallow passing more than one argument to a scoped-slot function call.
pub struct NoMultipleSlotArgs;

impl ScriptRule for NoMultipleSlotArgs {
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
        let mut visitor = SlotCallVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct SlotCallVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for SlotCallVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        // Only member-call callees (`<slots-source>.<name>(...)`) can be a slot
        // invocation. Other callees (bare functions, deeper chains) are skipped.
        if let Some(member) = as_member_expression(&it.callee)
            && is_slot_member(member)
        {
            self.check_arguments(it);
        }

        walk_call_expression(self, it);
    }
}

impl SlotCallVisitor<'_> {
    /// Report a slot call that receives a spread argument or more than one
    /// argument. A spread is reported even when it is the only argument because
    /// it may expand to several values; the slot still only receives the first.
    fn check_arguments(&mut self, call: &CallExpression<'_>) {
        if matches!(call.arguments.first(), Some(Argument::SpreadElement(_))) {
            self.report_spread(call.span);
        } else if call.arguments.len() > 1 {
            self.report_multiple(call.span);
        }
    }

    fn report_multiple(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result.add_diagnostic(
            LintDiagnostic::warn(
                META.name,
                "Unexpected multiple arguments passed to a scoped-slot function.",
                start,
                end,
            )
            .with_label("only the first argument reaches the slot", start, end)
            .with_help(
                "A slot function forwards a single \"slot props\" value. Pass one object \
                 instead, e.g. `slots.default({ a, b })`.",
            ),
        );
    }

    fn report_spread(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result.add_diagnostic(
            LintDiagnostic::warn(
                META.name,
                "Unexpected spread argument passed to a scoped-slot function.",
                start,
                end,
            )
            .with_label("a slot receives a single slot-props value", start, end)
            .with_help(
                "A slot function forwards a single \"slot props\" value, so a spread may \
                 leak extra values. Pass one object instead, e.g. `slots.default({ ... })`.",
            ),
        );
    }
}

/// View a callee expression as a member access, unwrapping a parenthesized
/// expression (`(slots.default)(a, b)`). Returns `None` for any non-member
/// callee.
fn as_member_expression<'a, 'b>(
    expression: &'b Expression<'a>,
) -> Option<&'b MemberExpression<'a>> {
    match expression {
        Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => expression.as_member_expression(),
        Expression::ParenthesizedExpression(paren) => as_member_expression(&paren.expression),
        _ => None,
    }
}

/// Whether `<object>.<slot-name>` accesses a slot off a recognised slots source.
///
/// The slot name (`.<slot-name>`) is whatever member is being accessed; it is
/// the *object* that must be a slots source. A computed slot name is allowed
/// only when it is a string literal so the access still statically names a slot
/// (`slots['default']`); a dynamic computed key is treated as not-a-slot to stay
/// conservative.
fn is_slot_member(member: &MemberExpression<'_>) -> bool {
    match member {
        MemberExpression::StaticMemberExpression(member) => is_slots_source(&member.object),
        MemberExpression::ComputedMemberExpression(member) => {
            matches!(&member.expression, Expression::StringLiteral(_))
                && is_slots_source(&member.object)
        }
        MemberExpression::PrivateFieldExpression(_) => false,
    }
}

/// Whether `object` denotes a slots collection: a bare `slots` / `$slots` /
/// `$scopedSlots` identifier, `this.$slots` / `this.$scopedSlots`, or the value
/// returned by a `useSlots()` call.
fn is_slots_source(object: &Expression<'_>) -> bool {
    match object {
        Expression::Identifier(identifier) => is_slots_identifier(identifier.name.as_str()),
        Expression::StaticMemberExpression(member) => {
            matches!(&member.object, Expression::ThisExpression(_))
                && is_dollar_slots(member.property.name.as_str())
        }
        Expression::ComputedMemberExpression(member) => {
            matches!(&member.object, Expression::ThisExpression(_))
                && matches!(
                    &member.expression,
                    Expression::StringLiteral(key) if is_dollar_slots(key.value.as_str())
                )
        }
        Expression::CallExpression(call) => is_use_slots_call(call),
        Expression::ParenthesizedExpression(paren) => is_slots_source(&paren.expression),
        _ => false,
    }
}

/// A bare slots identifier: `slots`, `$slots`, or `$scopedSlots`.
fn is_slots_identifier(name: &str) -> bool {
    matches!(name, "slots" | "$slots" | "$scopedSlots")
}

/// A `this.`-prefixed slots property: `$slots` or `$scopedSlots`.
fn is_dollar_slots(name: &str) -> bool {
    matches!(name, "$slots" | "$scopedSlots")
}

/// Whether the call is `useSlots(...)` (the Composition API slots accessor).
fn is_use_slots_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == "useSlots"
    )
}

#[cfg(test)]
mod tests;
