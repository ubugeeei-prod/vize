//! script/require-explicit-emits
//!
//! Require every emitted event to be declared in `defineEmits` / the Options API
//! `emits` option. An undeclared emit is invisible to tooling and (since
//! Vue 3.3) falls through as a native DOM listener on the root element, which is
//! rarely intended. Declaring emits documents the component's event surface.
//!
//! Port of
//! [`vue/require-explicit-emits`](https://eslint.vuejs.org/rules/require-explicit-emits.html).
//!
//! ## Sound subset
//!
//! Upstream also scans the template (`@evt` / `v-on:evt`), which a `script/*`
//! rule cannot observe. This implementation only decides what is sound from the
//! script alone:
//!
//! * It reports only when a declaration **exists** and is **fully known**. With
//!   no declaration, nothing is reported (the emits may be declared elsewhere, or
//!   intentionally not at all) — flagging would be a false positive.
//! * A declaration that cannot be fully enumerated — an array/object spread
//!   (`defineEmits([...names])`), a computed object key, or a bare type reference
//!   (`defineEmits<Emits>()`) — is treated as unknown, so nothing is reported.
//! * Only string-literal emit names are checked; a dynamic name (`emit(name)`)
//!   is skipped.
//!
//! Emit call sites are the captured `defineEmits` binding (`emit('change')`) and
//! `this.$emit('change')`. The declaration-resolution logic lives in the
//! [`declared`] submodule.
//!
//! ```ts
//! const emit = defineEmits(['change'])
//! emit('change') // ok
//! emit('input')  // reported: 'input' is not declared
//! ```

mod declared;

use oxc_ast::ast::{Argument, CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;

use vize_carton::{CompactString, FxHashSet};

use self::declared::{Declared, resolve_declared_emits};
use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-explicit-emits",
    description: "Require emitted events to be declared in defineEmits or the emits option",
    default_severity: Severity::Warning,
};

/// Require every emitted event to be declared.
pub struct RequireExplicitEmits;

impl ScriptRule for RequireExplicitEmits {
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
        // Bail unless a declaration exists AND is fully known; a missing or
        // partially-unknown declaration would make any report a false positive.
        let Declared::Known {
            names: declared,
            binding,
        } = resolve_declared_emits(program)
        else {
            return;
        };

        let mut visitor = EmitCallVisitor {
            declared: &declared,
            binding,
            offset,
            result,
        };
        visitor.visit_program(program);
    }
}

/// Walks the program reporting every string-literal emit whose name is absent
/// from the declared set. Emit call sites are the captured `defineEmits` binding
/// and `this.$emit(...)`.
struct EmitCallVisitor<'a, 'result> {
    declared: &'a FxHashSet<CompactString>,
    binding: Option<&'a str>,
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for EmitCallVisitor<'_, '_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(event) = emitted_event(it, self.binding)
            && !self.declared.contains(event.value)
        {
            self.report(event.value, event.span);
        }
        walk_call_expression(self, it);
    }
}

impl EmitCallVisitor<'_, '_> {
    fn report(&mut self, name: &str, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;

        let mut message = CompactString::with_capacity(name.len() + 48);
        message.push_str("The emitted event '");
        message.push_str(name);
        message.push_str("' is not declared in defineEmits or the emits option.");

        let diagnostic = LintDiagnostic::warn(META.name, message, start, end)
            .with_label("undeclared emitted event", start, end)
            .with_help(
                "Add this event to the defineEmits declaration (or the Options API `emits` \
                 option) so the component's event surface is explicit.",
            );
        self.result.add_diagnostic(diagnostic);
    }
}

/// The string-literal event name of an emit call, if `call` is one we track.
struct EmittedEvent<'a> {
    value: &'a str,
    span: Span,
}

/// The emitted event of `call` when it is one we track: the captured
/// `defineEmits` binding (`emit('x')`) or `this.$emit('x')`. Only a
/// string-literal first argument yields an event.
fn emitted_event<'a>(
    call: &'a CallExpression<'a>,
    binding: Option<&str>,
) -> Option<EmittedEvent<'a>> {
    if !is_emit_callee(&call.callee, binding) {
        return None;
    }
    match call.arguments.first()? {
        Argument::StringLiteral(literal) => Some(EmittedEvent {
            value: literal.value.as_str(),
            span: literal.span,
        }),
        _ => None,
    }
}

/// Whether `callee` is an emit dispatcher: the captured `defineEmits` binding
/// (when one exists), or a `*.$emit` member access (`this.$emit`, ...).
fn is_emit_callee(callee: &Expression<'_>, binding: Option<&str>) -> bool {
    match callee {
        Expression::Identifier(identifier) => {
            binding.is_some_and(|binding| binding == identifier.name.as_str())
        }
        expression if expression.is_member_expression() => expression
            .as_member_expression()
            .and_then(|member| member.static_property_name())
            .is_some_and(|property| property == "$emit"),
        // Look through wrappers so `(emit)('x')` / `(this.$emit)('x')` count.
        Expression::ParenthesizedExpression(paren) => is_emit_callee(&paren.expression, binding),
        Expression::TSAsExpression(ts) => is_emit_callee(&ts.expression, binding),
        Expression::TSSatisfiesExpression(ts) => is_emit_callee(&ts.expression, binding),
        Expression::TSNonNullExpression(ts) => is_emit_callee(&ts.expression, binding),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
