//! vue/no-this-in-before-route-enter
//!
//! `beforeRouteEnter` runs before the component instance exists, so `this` inside
//! that function is not the instance. An arrow nested in the hook keeps the same
//! `this`. A nested `function` has its own `this` and is left alone. Arrow
//! properties are not the hook function the rule tracks.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, Function,
    ObjectPropertyKind, Program,
};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{
    walk_call_expression, walk_export_default_declaration, walk_function, walk_new_expression,
    walk_program,
};
use oxc_syntax::scope::ScopeFlags;

pub(super) const RULE: &str = "vue/no-this-in-before-route-enter";
pub(super) const MESSAGE: &str = "'beforeRouteEnter' does NOT have access to `this` component instance. https://router.vuejs.org/guide/advanced/navigation-guards.html#in-component-guards.";

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: RULE,
    description: "Disallow `this` in `beforeRouteEnter`",
    default_severity: Severity::Warning,
};

/// Implements the `script/no-this-in-before-route-enter` lint rule.
pub struct NoThisInBeforeRouteEnter;

impl ScriptRule for NoThisInBeforeRouteEnter {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

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
        let mut visitor = Visitor {
            offset,
            result,
            force_vue: false,
            pending_guard: false,
            guards: Vec::new(),
        };
        visitor.visit_program(program);
    }
}

struct Visitor<'a> {
    offset: usize,
    result: &'a mut ScriptLintResult,
    force_vue: bool,
    pending_guard: bool,
    guards: Vec<bool>,
}

impl Visitor<'_> {
    fn peel<'b>(expr: &'b Expression<'b>) -> &'b Expression<'b> {
        match expr {
            Expression::ParenthesizedExpression(inner) => Self::peel(&inner.expression),
            Expression::TSAsExpression(inner) => Self::peel(&inner.expression),
            Expression::TSSatisfiesExpression(inner) => Self::peel(&inner.expression),
            Expression::TSNonNullExpression(inner) => Self::peel(&inner.expression),
            other => other,
        }
    }

    fn is_vue_factory(call: &CallExpression<'_>) -> bool {
        match Self::peel(&call.callee) {
            Expression::Identifier(id) => matches!(
                id.name.as_str(),
                "defineComponent" | "createApp" | "defineNuxtComponent" | "component"
            ),
            Expression::StaticMemberExpression(member) => {
                if !matches!(Self::peel(&member.object), Expression::Identifier(_)) {
                    return false;
                }
                match member.property.name.as_str() {
                    "component" | "mixin" => true,
                    "extend" => {
                        matches!(Self::peel(&member.object), Expression::Identifier(id) if id.name == "Vue")
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn mark_vue_argument(&mut self, call: &CallExpression<'_>) {
        if !Self::is_vue_factory(call) {
            return;
        }
        if let Some(Argument::ObjectExpression(_)) = call.arguments.last() {
            self.force_vue = true;
        }
    }

    fn report_this(&mut self, span: oxc_span::Span) {
        self.result.add_diagnostic(LintDiagnostic::warn(
            RULE,
            MESSAGE,
            self.offset as u32 + span.start,
            self.offset as u32 + span.end,
        ));
    }
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_program(&mut self, it: &Program<'a>) {
        walk_program(self, it);
    }

    fn visit_export_default_declaration(
        &mut self,
        it: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        if matches!(
            it.declaration,
            ExportDefaultDeclarationKind::ObjectExpression(_)
        ) {
            self.force_vue = true;
        }
        walk_export_default_declaration(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.mark_vue_argument(it);
        walk_call_expression(self, it);
    }

    fn visit_new_expression(&mut self, it: &oxc_ast::ast::NewExpression<'a>) {
        if matches!(Self::peel(&it.callee), Expression::Identifier(id) if id.name == "Vue")
            && matches!(it.arguments.first(), Some(Argument::ObjectExpression(_)))
        {
            self.force_vue = true;
        }
        walk_new_expression(self, it);
    }

    fn visit_object_expression(&mut self, it: &oxc_ast::ast::ObjectExpression<'a>) {
        let is_vue = self.force_vue;
        self.force_vue = false;
        for property in &it.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                if let ObjectPropertyKind::SpreadProperty(spread) = property {
                    self.visit_expression(&spread.argument);
                }
                continue;
            };
            self.visit_property_key(&property.key);
            if is_vue
                && property.key.static_name().as_deref() == Some("beforeRouteEnter")
                && matches!(property.value, Expression::FunctionExpression(_))
            {
                self.pending_guard = true;
            }
            self.visit_expression(&property.value);
            self.pending_guard = false;
        }
    }

    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        let guard = self.pending_guard;
        self.pending_guard = false;
        self.guards.push(guard);
        walk_function(self, it, flags);
        self.guards.pop();
    }

    fn visit_this_expression(&mut self, it: &oxc_ast::ast::ThisExpression) {
        if self.guards.last() == Some(&true) {
            self.report_this(it.span);
        }
    }
}

#[cfg(test)]
mod tests;
