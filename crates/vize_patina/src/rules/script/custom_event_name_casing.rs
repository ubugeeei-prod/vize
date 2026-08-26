//! script/custom-event-name-casing
//!
//! Enforce camelCase for emitted custom event names.
//!
//! eslint-plugin-vue recommends a consistent casing for the events a component
//! emits. For Vue 3 the default is **camelCase**, so an emit whose event name is
//! kebab-case (`'my-event'`) or PascalCase (`'MyEvent'`) is inconsistent with the
//! recommended casing and is reported.
//!
//! This walks the emit call sites and checks the string-literal event name (the
//! first argument). Only string-literal event names are checked; a dynamic name
//! (`emit(eventName)`) carries no literal to inspect and is skipped. The
//! `update:` prefix used by `v-model` (`'update:modelValue'`) is permitted.
//!
//! ## Script call sites
//!
//! A call to the captured `defineEmits` binding (`const emit = defineEmits(...)`
//! then `emit('my-event')`), or a member call whose property is `emit`/`$emit`
//! (`ctx.emit('my-event')`, `this.$emit('my-event')`).
//!
//! ## Template call sites
//!
//! A template dispatches the same events, through the built-in `$emit` helper
//! (`@click="$emit('foo-bar')"`) or through the captured binding, which is
//! template-visible as a top-level `<script setup>` binding. Recovering them
//! *creates* findings from template evidence, so they come from the template
//! AST and an oxc parse of each expression; see [`super::template_scan`] for the
//! over-match analysis and the shadowing rules.
//!
//! The template half additionally requires the SFC to have a single script
//! block. The rule is invoked once per block, and unlike the script half — whose
//! call sites live in the block being linted — a template `$emit` is visible to
//! every block, so two blocks would report it twice.
//!
//! Mirrors [`vue/custom-event-name-casing`](https://eslint.vuejs.org/rules/custom-event-name-casing.html)
//! with the Vue 3 default (`camelCase`).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const emit = defineEmits(['my-event'])
//! emit('my-event')         // kebab-case → report
//! ```
//!
//! ### Valid
//! ```ts
//! const emit = defineEmits(['myEvent'])
//! emit('myEvent')
//! ```

use super::template_scan::{DOLLAR_EMIT, for_each_template_call};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, Program, Statement, StringLiteral,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use vize_s0::CompactString;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/custom-event-name-casing",
    description: "Enforce camelCase for emitted custom event names",
    default_severity: Severity::Error,
};

/// Enforce camelCase for emitted custom event names.
pub struct CustomEventNameCasing;

impl ScriptRule for CustomEventNameCasing {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn uses_template_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        // Keep the parse-owning `check` path functional: without SFC context
        // only the script call sites are observable.
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        // The `<script setup>` emit function captured from `defineEmits(...)`, if
        // any. A bare `defineEmits([...])` without a binding cannot be tracked.
        let emit_binding = find_emit_binding(program);

        let mut visitor = CustomEventNameCasingVisitor {
            offset,
            result,
            emit_binding,
        };
        visitor.visit_program(program);

        check_template(emit_binding, sfc, result);
    }
}

/// The template half: `$emit('foo-bar')`, or a call of the captured binding.
fn check_template(
    emit_binding: Option<&str>,
    sfc: SfcScriptContext<'_>,
    result: &mut ScriptLintResult,
) {
    if !sfc.sole_script_block {
        return;
    }
    let Some((root, template_offset)) = sfc.template_ast() else {
        return;
    };
    for_each_template_call(root, |call| {
        if call.callee != DOLLAR_EMIT && Some(call.callee) != emit_binding {
            return;
        }
        let Some(event) = call.first_string_argument else {
            return;
        };
        if is_camel_case_event(event.value) {
            return;
        }
        report(
            event.value,
            template_offset + event.start,
            template_offset + event.end,
            result,
        );
    });
}

struct CustomEventNameCasingVisitor<'a, 'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    emit_binding: Option<&'a str>,
}

impl<'a> Visit<'a> for CustomEventNameCasingVisitor<'a, '_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.is_emit_call(it)
            && let Some(Argument::StringLiteral(literal)) = it.arguments.first()
        {
            self.check_event_name(literal);
        }
        walk_call_expression(self, it);
    }
}

impl<'a> CustomEventNameCasingVisitor<'a, '_> {
    /// Whether the call is an emit of a custom event: a call to the captured
    /// `defineEmits` binding (`emit(...)`) or a member call whose property is
    /// `emit`/`$emit` (`ctx.emit(...)`, `this.$emit(...)`).
    fn is_emit_call(&self, call: &CallExpression<'a>) -> bool {
        match &call.callee {
            Expression::Identifier(identifier) => {
                self.emit_binding == Some(identifier.name.as_str())
            }
            Expression::StaticMemberExpression(member) => {
                matches!(member.property.name.as_str(), "emit" | "$emit")
            }
            _ => false,
        }
    }

    fn check_event_name(&mut self, literal: &StringLiteral<'_>) {
        let value = literal.value.as_str();
        if is_camel_case_event(value) {
            return;
        }
        report(
            value,
            self.offset as u32 + literal.span.start,
            self.offset as u32 + literal.span.end,
            self.result,
        );
    }
}

fn report(value: &str, start: u32, end: u32, result: &mut ScriptLintResult) {
    let mut message = CompactString::with_capacity(value.len() + 40);
    message.push_str("Custom event name '");
    message.push_str(value);
    message.push_str("' is not camelCase.");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("expected camelCase", start, end)
        .with_help(
            "Vue 3 recommends camelCase for emitted event names; rename this event \
             to camelCase (e.g. `myEvent`).",
        );
    result.add_diagnostic(diagnostic);
}

/// Whether `value` is an acceptable camelCase event name. Each `:`-separated
/// segment (so the `v-model` `update:modelValue` form is allowed) must match
/// `^[a-z][a-zA-Z0-9]*$`: a lowercase first character followed by alphanumerics.
fn is_camel_case_event(value: &str) -> bool {
    !value.is_empty() && value.split(':').all(is_camel_case_segment)
}

/// Whether a single segment matches `^[a-z][a-zA-Z0-9]*$`.
fn is_camel_case_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

/// The identifier a top-level `const <id> = defineEmits(...)` binds the emit
/// function to, if present. An unassigned `defineEmits(...)` returns `None`.
fn find_emit_binding<'a>(program: &'a Program<'a>) -> Option<&'a str> {
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            if declarator
                .init
                .as_ref()
                .is_some_and(|init| is_define_emits_call(init))
            {
                return Some(id.name.as_str());
            }
        }
    }
    None
}

/// Whether the expression is a `defineEmits(...)` call, unwrapping the TS
/// `as`/`satisfies`/non-null and parenthesized wrappers.
fn is_define_emits_call(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::CallExpression(call) => matches!(
            &call.callee,
            Expression::Identifier(identifier) if identifier.name.as_str() == "defineEmits"
        ),
        Expression::ParenthesizedExpression(paren) => is_define_emits_call(&paren.expression),
        Expression::TSAsExpression(ts) => is_define_emits_call(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => is_define_emits_call(&ts.expression),
        Expression::TSNonNullExpression(ts) => is_define_emits_call(&ts.expression),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
