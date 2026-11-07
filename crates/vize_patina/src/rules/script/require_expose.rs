//! vue/require-expose
//!
//! An options-API component object must declare `expose`, call setup's `expose`
//! (or `context.expose`), or return a render function from `setup`. Mixins are
//! left alone. This script pass cannot see a sibling `<script setup>`, so a
//! bare `export default {}` still warns. `defineExpose` is not this rule.

mod setup;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    CallExpression, ExportDefaultDeclaration, Expression, NewExpression, ObjectExpression,
    ObjectPropertyKind, Program,
};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{walk_export_default_declaration, walk_object_expression, walk_program};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::Span;

pub(super) const RULE: &str = "vue/require-expose";
pub(super) const MESSAGE: &str = "The public properties of the component must be explicitly declared using `expose`. If the component does not have public properties, declare it empty.";

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: RULE,
    description: "Require declare public properties using `expose`",
    default_severity: Severity::Warning,
};

pub struct RequireExpose;

impl ScriptRule for RequireExpose {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn runs_on_script_setup(&self) -> bool {
        // `eslint-plugin-vue` deliberately does not treat local
        // `defineComponent({...})` calls inside `<script setup>` as component
        // definitions for this rule. The SFC itself uses `defineExpose`, which
        // is a different API and is outside `vue/require-expose`.
        false
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let semantic = SemanticBuilder::new()
            .with_build_nodes(true)
            .build(program)
            .semantic;
        let mut visitor = Visitor {
            offset,
            result,
            semantic: &semantic,
            pending: None,
        };
        visitor.visit_program(program);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Component,
    Mixin,
}

struct Visitor<'r, 'a> {
    offset: usize,
    result: &'r mut ScriptLintResult,
    semantic: &'r Semantic<'a>,
    pending: Option<Kind>,
}

impl<'r, 'a> Visitor<'r, 'a> {
    fn report(&mut self, span: Span) {
        self.result.add_diagnostic(LintDiagnostic::warn(
            RULE,
            MESSAGE,
            self.offset as u32 + span.start,
            self.offset as u32 + span.end,
        ));
    }

    fn consider(&mut self, object: &ObjectExpression<'a>) {
        let mut exposed = false;
        let mut exempt = false;
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            let name = property.key.static_name();
            if !property.computed && name.as_deref() == Some("expose") {
                exposed = true;
            }
            if name.as_deref() == Some("setup") {
                exempt |= setup::is_exempt(&property.value, self.semantic);
            }
        }
        if !exposed && !exempt {
            self.report(object.span);
        }
    }
}

impl<'a> Visit<'a> for Visitor<'_, 'a> {
    fn visit_program(&mut self, it: &Program<'a>) {
        walk_program(self, it);
    }

    fn visit_export_default_declaration(&mut self, it: &ExportDefaultDeclaration<'a>) {
        let marks_object = it.declaration.as_expression().is_some_and(|expression| {
            matches!(
                setup::skip_wrappers(expression),
                Expression::ObjectExpression(_)
            )
        });
        if marks_object {
            self.pending = Some(Kind::Component);
        }
        walk_export_default_declaration(self, it);
        self.pending = None;
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.visit_expression(&it.callee);
        let kind = it
            .arguments
            .last()
            .filter(|argument| setup::is_object_argument(argument))
            .and_then(|_| setup::classify(self.semantic, &it.callee));
        let last = it.arguments.len().saturating_sub(1);
        for (index, argument) in it.arguments.iter().enumerate() {
            if index == last {
                self.pending = kind;
            }
            self.visit_argument(argument);
            if index == last {
                self.pending = None;
            }
        }
    }

    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        self.visit_expression(&it.callee);
        let vue_object = matches!(
            it.callee.without_parentheses(),
            Expression::Identifier(identifier) if identifier.name == "Vue"
        ) && it.arguments.first().is_some_and(setup::is_object_argument);
        for (index, argument) in it.arguments.iter().enumerate() {
            if index == 0 && vue_object {
                self.pending = Some(Kind::Component);
            }
            self.visit_argument(argument);
            if index == 0 {
                self.pending = None;
            }
        }
    }

    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        let pending = self.pending.take();
        if pending == Some(Kind::Component) {
            self.consider(it);
        }
        walk_object_expression(self, it);
    }
}

#[cfg(test)]
mod tests;
