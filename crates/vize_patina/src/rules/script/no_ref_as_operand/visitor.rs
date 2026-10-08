use super::{
    RefOperandVisitor,
    scope::{
        Binding, Frame, collect_function_scope, collect_scope_bindings, prime_scope_bindings,
        record_binding_names, record_declaration,
    },
};
use oxc_ast::ast::{
    AssignmentExpression, BinaryExpression, BlockStatement, CatchClause, Class,
    ConditionalExpression, DoWhileStatement, ForInStatement, ForOfStatement, ForStatement,
    ForStatementLeft, Function, FunctionBody, IfStatement, LogicalExpression, Program, Statement,
    StaticBlock, SwitchStatement, TemplateLiteral, UnaryExpression, UpdateExpression,
    WhileStatement,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_assignment_expression, walk_binary_expression, walk_block_statement,
        walk_catch_clause, walk_class, walk_conditional_expression, walk_do_while_statement,
        walk_for_in_statement, walk_for_of_statement, walk_for_statement, walk_function,
        walk_function_body, walk_if_statement, walk_logical_expression, walk_program,
        walk_static_block, walk_template_literal, walk_unary_expression, walk_update_expression,
        walk_while_statement,
    },
};

use oxc_syntax::{operator::UnaryOperator, scope::ScopeFlags};

impl<'a> Visit<'a> for RefOperandVisitor<'_> {
    fn visit_program(&mut self, it: &Program<'a>) {
        self.scopes
            .push(collect_function_scope(&it.body, &self.scopes));
        walk_program(self, it);
        self.scopes.pop();
    }

    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        // Parameters shadow outer bindings of the same name (a param is never a
        // ref binding for our purposes), as do the function body's own locals.
        let mut frame = Frame::default();
        if let Some(id) = &it.id {
            frame.insert(id.name.as_str().into(), Binding::Other);
        }
        for param in &it.params.items {
            record_binding_names(&param.pattern, Binding::Other, &mut frame);
        }
        if let Some(rest) = &it.params.rest {
            record_binding_names(&rest.rest.argument, Binding::Other, &mut frame);
        }
        self.scopes.push(frame);
        walk_function(self, it, flags);
        self.scopes.pop();
    }

    fn visit_function_body(&mut self, it: &FunctionBody<'a>) {
        // Parameters run before the body and cannot see its local declarations.
        self.scopes
            .push(collect_function_scope(&it.statements, &self.scopes));
        walk_function_body(self, it);
        self.scopes.pop();
    }

    fn visit_static_block(&mut self, it: &StaticBlock<'a>) {
        self.scopes
            .push(collect_function_scope(&it.body, &self.scopes));
        walk_static_block(self, it);
        self.scopes.pop();
    }

    fn visit_switch_statement(&mut self, it: &SwitchStatement<'a>) {
        self.visit_expression(&it.discriminant);
        let mut frame = Frame::default();
        for case in &it.cases {
            prime_scope_bindings(&case.consequent, &mut frame);
        }
        for case in &it.cases {
            for statement in &case.consequent {
                if let Statement::VariableDeclaration(declaration) = statement {
                    record_declaration(declaration, &mut frame, &self.scopes);
                }
            }
        }
        self.scopes.push(frame);
        for case in &it.cases {
            self.visit_switch_case(case);
        }
        self.scopes.pop();
    }

    fn visit_class(&mut self, it: &Class<'a>) {
        let mut frame = Frame::default();
        if let Some(id) = &it.id {
            frame.insert(id.name.as_str().into(), Binding::Other);
        }
        self.scopes.push(frame);
        walk_class(self, it);
        self.scopes.pop();
    }

    fn visit_arrow_function_expression(&mut self, it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
        let mut frame = Frame::default();
        for param in &it.params.items {
            record_binding_names(&param.pattern, Binding::Other, &mut frame);
        }
        if let Some(rest) = &it.params.rest {
            record_binding_names(&rest.rest.argument, Binding::Other, &mut frame);
        }
        self.scopes.push(frame);
        oxc_ast_visit::walk::walk_arrow_function_expression(self, it);
        self.scopes.pop();
    }

    fn visit_block_statement(&mut self, it: &BlockStatement<'a>) {
        self.scopes
            .push(collect_scope_bindings(&it.body, &self.scopes));
        walk_block_statement(self, it);
        self.scopes.pop();
    }

    fn visit_catch_clause(&mut self, it: &CatchClause<'a>) {
        let mut frame = Frame::default();
        if let Some(param) = &it.param {
            record_binding_names(&param.pattern, Binding::Other, &mut frame);
        }
        self.scopes.push(frame);
        walk_catch_clause(self, it);
        self.scopes.pop();
    }

    fn visit_for_statement(&mut self, it: &ForStatement<'a>) {
        // A `for (let x = ...; ...)` header introduces a scope around the test,
        // update, and body. Collect any bindings declared in the init.
        let mut frame = Frame::default();
        if let Some(oxc_ast::ast::ForStatementInit::VariableDeclaration(declaration)) = &it.init {
            record_declaration(declaration, &mut frame, &self.scopes);
        }
        self.scopes.push(frame);
        if let Some(test) = &it.test {
            self.check_operand(test);
        }
        walk_for_statement(self, it);
        self.scopes.pop();
    }

    fn visit_for_in_statement(&mut self, it: &ForInStatement<'a>) {
        let mut frame = Frame::default();
        if let ForStatementLeft::VariableDeclaration(declaration) = &it.left {
            record_declaration(declaration, &mut frame, &self.scopes);
        }
        self.scopes.push(frame);
        walk_for_in_statement(self, it);
        self.scopes.pop();
    }

    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        let mut frame = Frame::default();
        if let ForStatementLeft::VariableDeclaration(declaration) = &it.left {
            record_declaration(declaration, &mut frame, &self.scopes);
        }
        self.scopes.push(frame);
        walk_for_of_statement(self, it);
        self.scopes.pop();
    }

    // --- Operator contexts ---

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        // `count++`, `--count`: the target is read and written as a value.
        if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = &it.argument
            && self.is_ref(&id.name)
        {
            self.report(id.span, &id.name);
        }
        walk_update_expression(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        // A compound assignment (`count += 1`, `count ??= x`) reads the target
        // as a value first. A plain `count = x` only rebinds and is left alone.
        if !it.operator.is_assign()
            && let oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(id) = &it.left
            && self.is_ref(&id.name)
        {
            self.report(id.span, &id.name);
        }
        walk_assignment_expression(self, it);
    }

    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        // `!count`, `-count`, `+count`, `~count` read the value. `typeof`,
        // `void`, and `delete` inspect the binding itself and are not flagged.
        if !matches!(
            it.operator,
            UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete
        ) {
            self.check_operand(&it.argument);
        }
        walk_unary_expression(self, it);
    }

    fn visit_binary_expression(&mut self, it: &BinaryExpression<'a>) {
        self.check_operand(&it.left);
        self.check_operand(&it.right);
        walk_binary_expression(self, it);
    }

    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        self.check_operand(&it.left);
        self.check_operand(&it.right);
        walk_logical_expression(self, it);
    }

    fn visit_conditional_expression(&mut self, it: &ConditionalExpression<'a>) {
        self.check_operand(&it.test);
        walk_conditional_expression(self, it);
    }

    fn visit_template_literal(&mut self, it: &TemplateLiteral<'a>) {
        for expression in &it.expressions {
            self.check_operand(expression);
        }
        walk_template_literal(self, it);
    }

    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        self.check_operand(&it.test);
        walk_if_statement(self, it);
    }

    fn visit_while_statement(&mut self, it: &WhileStatement<'a>) {
        self.check_operand(&it.test);
        walk_while_statement(self, it);
    }

    fn visit_do_while_statement(&mut self, it: &DoWhileStatement<'a>) {
        self.check_operand(&it.test);
        walk_do_while_statement(self, it);
    }
}
