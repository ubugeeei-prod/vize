use super::{Binding, Frame, merge_scope_bindings, record_binding_names};
use oxc_ast::ast::{
    ArrowFunctionExpression, Class, Expression, Function, Statement, VariableDeclaration,
    VariableDeclarationKind,
};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;

pub(in crate::rules::script::no_ref_as_operand) fn collect_function_scope(
    statements: &[Statement<'_>],
    outer: &[Frame],
) -> Frame {
    let mut frame = Frame::default();
    merge_function_scope(statements, &mut frame, outer);
    frame
}

pub(in crate::rules::script::no_ref_as_operand) fn merge_function_scope(
    statements: &[Statement<'_>],
    frame: &mut Frame,
    outer: &[Frame],
) {
    // `var` shadows the function/module binding even when written in a nested
    // block or loop. Nested functions and classes own separate var scopes.
    let mut visitor = HoistedVars(frame);
    for statement in statements {
        visitor.visit_statement(statement);
    }
    merge_scope_bindings(statements, frame, outer);
}

struct HoistedVars<'scope>(&'scope mut Frame);

impl<'a> Visit<'a> for HoistedVars<'_> {
    // Hoisted declarations live in statement/loop structure. Expressions only
    // contain declarations inside nested functions/classes, which own scopes.
    fn visit_expression(&mut self, _: &Expression<'a>) {}
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Var {
            for declarator in &declaration.declarations {
                record_binding_names(&declarator.id, Binding::Other, self.0);
            }
        }
    }

    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
    fn visit_class(&mut self, _: &Class<'a>) {}
}
