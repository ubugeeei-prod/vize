//! Walk an instance entry body; arrows inherit, ordinary functions rebind `this`.

use oxc_ast::ast::{
    AssignmentExpression, BindingIdentifier, CallExpression, Class, Function, Program,
    UnaryExpression, UpdateExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_assignment_expression, walk_call_expression, walk_unary_expression,
        walk_update_expression,
    },
};
use oxc_span::Span;
use oxc_syntax::{operator::UnaryOperator, scope::ScopeFlags};
use vize_l0::{FxHashSet, String, cstr};

use super::super::{META, script_mutations};
use super::targets;
use crate::{diagnostic::LintDiagnostic, rules::script::ScriptLintResult};

pub(super) fn has_object_binding(program: &Program<'_>) -> bool {
    struct Bindings(bool);
    impl<'a> Visit<'a> for Bindings {
        fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
            self.0 |= identifier.name.as_str() == "Object";
        }
    }
    let mut bindings = Bindings(false);
    bindings.visit_program(program);
    bindings.0
}

pub(super) fn check(
    function: &Function<'_>,
    props: &FxHashSet<String>,
    shallow_only: bool,
    allow_object_assign: bool,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let mut collector = Collector {
        props,
        shallow_only,
        allow_object_assign,
        offset: offset as u32,
        result,
    };
    collector.visit_formal_parameters(&function.params);
    if let Some(body) = &function.body {
        collector.visit_function_body(body);
    }
}

struct Collector<'r> {
    props: &'r FxHashSet<String>,
    shallow_only: bool,
    allow_object_assign: bool,
    offset: u32,
    result: &'r mut ScriptLintResult,
}

impl Collector<'_> {
    fn record(&mut self, target: Option<(&str, bool)>, span: Span, mutating_call: bool) {
        let Some((name, deep)) = target else { return };
        if self.shallow_only && (deep || mutating_call) {
            return;
        }
        self.result.add_diagnostic(
            LintDiagnostic::error(
                META.name,
                cstr!("Unexpected mutation of prop '{name}' in <script>"),
                self.offset + span.start,
                self.offset + span.end,
            )
            .with_help("Use a local ref or emit an event instead of mutating props directly"),
        );
    }
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}

    fn visit_class(&mut self, class: &Class<'a>) {
        // Heritage is evaluated in the outer scope; fields/methods own class `this`.
        if let Some(heritage) = &class.super_class {
            self.visit_expression(heritage);
        }
    }

    fn visit_assignment_expression(&mut self, expression: &AssignmentExpression<'a>) {
        if let Some(target) = expression.left.as_simple_assignment_target() {
            self.record(
                targets::assignment(target, self.props),
                expression.span,
                false,
            );
        }
        walk_assignment_expression(self, expression);
    }

    fn visit_update_expression(&mut self, expression: &UpdateExpression<'a>) {
        self.record(
            targets::assignment(&expression.argument, self.props),
            expression.span,
            false,
        );
        walk_update_expression(self, expression);
    }

    fn visit_unary_expression(&mut self, expression: &UnaryExpression<'a>) {
        if expression.operator == UnaryOperator::Delete {
            self.record(
                targets::expression(&expression.argument, self.props),
                expression.span,
                false,
            );
        }
        walk_unary_expression(self, expression);
    }

    fn visit_call_expression(&mut self, expression: &CallExpression<'a>) {
        if (self.allow_object_assign || !targets::object_assign(&expression.callee))
            && let Some(target) = script_mutations::mutating_call_target(expression)
        {
            self.record(
                targets::expression(target, self.props),
                expression.span,
                true,
            );
        }
        walk_call_expression(self, expression);
    }
}
