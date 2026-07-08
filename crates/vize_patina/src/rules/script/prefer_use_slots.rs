//! script/prefer-use-slots
//!
//! Recommend using useSlots() over $slots or context.slots.
//!
//! In Composition API, useSlots() is the preferred way to access
//! slot content programmatically. It's more explicit and works in both
//! `<script setup>` and regular setup functions.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // In Options API style
//! export default {
//!   setup(props, { slots }) {
//!     return () => h('div', slots.default?.())
//!   }
//! }
//!
//! // Using context.slots
//! const vnode = context.slots.default?.()
//! ```
//!
//! ### Valid
//! ```ts
//! // Using useSlots()
//! const slots = useSlots()
//! return () => h('div', slots.default?.())
//! ```

use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    BindingPattern, Expression, FormalParameters, ObjectProperty, Program, PropertyKey,
    StaticMemberExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_object_property, walk_static_member_expression},
};
use oxc_span::Span;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-use-slots",
    description: "Recommend using useSlots() over context.slots",
    default_severity: Severity::Warning,
};

/// Prefer useSlots() rule
pub struct PreferUseSlots;

impl ScriptRule for PreferUseSlots {
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
        let mut visitor = PreferUseSlotsVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct PreferUseSlotsVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for PreferUseSlotsVisitor<'_> {
    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        if !it.computed
            && property_key_name(&it.key) == Some("setup")
            && let Some(params) = setup_parameters(&it.value)
            && let Some(span) = setup_context_property_span(params, "slots")
        {
            self.add_diagnostic(
                "Prefer useSlots() over destructuring slots from setup context",
                span,
            );
        }

        walk_object_property(self, it);
    }

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        if it.property.name.as_str() == "slots"
            && matches!(&it.object, Expression::Identifier(identifier) if identifier.name.as_str() == "context")
        {
            self.add_diagnostic("Prefer useSlots() over context.slots", it.span);
        }

        walk_static_member_expression(self, it);
    }
}

impl PreferUseSlotsVisitor<'_> {
    fn add_diagnostic(&mut self, message: &'static str, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result.add_diagnostic(
            LintDiagnostic::warn(META.name, message, start, end)
                .with_help("Use `const slots = useSlots()` instead"),
        );
    }
}

fn setup_parameters<'a>(value: &'a Expression<'a>) -> Option<&'a FormalParameters<'a>> {
    match value {
        Expression::FunctionExpression(function) => Some(&function.params),
        Expression::ArrowFunctionExpression(arrow) => Some(&arrow.params),
        _ => None,
    }
}

fn setup_context_property_span(params: &FormalParameters<'_>, name: &str) -> Option<Span> {
    let context = params.items.get(1)?;
    let BindingPattern::ObjectPattern(object) = &context.pattern else {
        return None;
    };

    for property in &object.properties {
        if !property.computed && property_key_name(&property.key) == Some(name) {
            return Some(property.span);
        }
    }
    None
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::PreferUseSlots;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(PreferUseSlots));
        linter
    }

    #[test]
    fn test_valid_use_slots() {
        let linter = create_linter();
        let result = linter.lint("const slots = useSlots()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_context_slots() {
        let linter = create_linter();
        let result = linter.lint("const slot = context.slots.default?.()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_destructure_slots() {
        let linter = create_linter();
        let result = linter.lint("export default { setup(props, { slots }) {} }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_arrow_setup_destructure_slots() {
        let linter = create_linter();
        let result = linter.lint("export default { setup: (props, { slots }) => {} }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_no_warning_for_string_or_unrelated_destructure() {
        let linter = create_linter();
        let result = linter.lint(
            r#"
const text = "context.slots"
const { slots } = context
export default { mounted() { console.log(context.slots) } }
"#,
            0,
        );
        assert_eq!(result.warning_count, 1);
    }
}
