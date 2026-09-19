//! Lexical declaration ownership for retained JavaScript expression rewriters.
//!
//! Resolve names independently of source order without cloning or reparsing the
//! AST. Each emitter owns its rewrite policy; this walk owns JavaScript scopes.

use oxc_ast::ast;
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;

/// Scope operations shared by expression-prefix visitors in every compiler lane.
pub trait ExpressionScope<'a>: Visit<'a> + Sized {
    fn push_scope(&mut self);
    fn pop_scope(&mut self);
    fn add_local(&mut self, name: &str);

    fn collect_binding_pattern(&mut self, pattern: &ast::BindingPattern<'a>) {
        // Always continue: OXC visits nested/default/rest bindings without an
        // intermediate identifier list and without entering default expressions.
        pattern.all_binding_identifiers(&mut |identifier| {
            self.add_local(identifier.name.as_str());
            true
        });
    }

    fn declare_lexical_bindings(&mut self, statements: &[ast::Statement<'a>]) {
        for statement in statements {
            match statement {
                ast::Statement::VariableDeclaration(declaration) => {
                    self.declare_header(declaration)
                }
                ast::Statement::FunctionDeclaration(function) => {
                    if let Some(id) = &function.id {
                        self.add_local(id.name.as_str());
                    }
                }
                ast::Statement::ClassDeclaration(class) => {
                    if let Some(id) = &class.id {
                        self.add_local(id.name.as_str());
                    }
                }
                _ => {}
            }
        }
    }

    fn declare_header(&mut self, declaration: &ast::VariableDeclaration<'a>) {
        if declaration.kind != ast::VariableDeclarationKind::Var {
            for item in &declaration.declarations {
                self.collect_binding_pattern(&item.id);
            }
        }
    }

    fn scoped_program(&mut self, program: &ast::Program<'a>) {
        self.push_scope();
        crate::function_vars::for_each_var_in_statements(&program.body, |pattern| {
            self.collect_binding_pattern(pattern)
        });
        self.declare_lexical_bindings(&program.body);
        walk::walk_program(self, program);
        self.pop_scope();
    }

    fn scoped_arrow(&mut self, arrow: &ast::ArrowFunctionExpression<'a>) {
        self.push_scope();
        for pattern in arrow.params.iter_bindings() {
            self.collect_binding_pattern(pattern);
        }
        walk::walk_arrow_function_expression(self, arrow);
        self.pop_scope();
    }

    fn scoped_function(&mut self, function: &ast::Function<'a>, flags: ScopeFlags) {
        self.push_scope();
        if let Some(id) = &function.id {
            self.add_local(id.name.as_str());
        }
        for pattern in function.params.iter_bindings() {
            self.collect_binding_pattern(pattern);
        }
        walk::walk_function(self, function, flags);
        self.pop_scope();
    }

    fn scoped_function_body(&mut self, body: &ast::FunctionBody<'a>) {
        // Body declarations must not resolve a reference in a parameter default.
        self.push_scope();
        crate::for_each_function_var(body, |pattern| self.collect_binding_pattern(pattern));
        self.declare_lexical_bindings(&body.statements);
        walk::walk_function_body(self, body);
        self.pop_scope();
    }

    fn scoped_block(&mut self, block: &ast::BlockStatement<'a>) {
        self.push_scope();
        self.declare_lexical_bindings(&block.body);
        walk::walk_block_statement(self, block);
        self.pop_scope();
    }

    fn scoped_catch(&mut self, clause: &ast::CatchClause<'a>) {
        self.push_scope();
        if let Some(parameter) = &clause.param {
            self.collect_binding_pattern(&parameter.pattern);
        }
        walk::walk_catch_clause(self, clause);
        self.pop_scope();
    }

    fn scoped_for(&mut self, statement: &ast::ForStatement<'a>) {
        self.push_scope();
        if let Some(ast::ForStatementInit::VariableDeclaration(declaration)) = &statement.init {
            self.declare_header(declaration);
        }
        walk::walk_for_statement(self, statement);
        self.pop_scope();
    }

    fn scoped_for_in(&mut self, statement: &ast::ForInStatement<'a>) {
        self.push_scope();
        if let ast::ForStatementLeft::VariableDeclaration(declaration) = &statement.left {
            self.declare_header(declaration);
        }
        walk::walk_for_in_statement(self, statement);
        self.pop_scope();
    }

    fn scoped_for_of(&mut self, statement: &ast::ForOfStatement<'a>) {
        self.push_scope();
        if let ast::ForStatementLeft::VariableDeclaration(declaration) = &statement.left {
            self.declare_header(declaration);
        }
        walk::walk_for_of_statement(self, statement);
        self.pop_scope();
    }

    fn scoped_switch(&mut self, statement: &ast::SwitchStatement<'a>) {
        // The discriminant is evaluated before entering the shared case scope.
        self.visit_expression(&statement.discriminant);
        self.push_scope();
        for case in &statement.cases {
            self.declare_lexical_bindings(&case.consequent);
        }
        for case in &statement.cases {
            self.visit_switch_case(case);
        }
        self.pop_scope();
    }

    fn scoped_class(&mut self, class: &ast::Class<'a>) {
        // Preserve OXC's child order and callbacks, entering the name scope
        // after class decorators. Calling walk_class here would visit them twice.
        let kind = oxc_ast::AstKind::Class(self.alloc(class));
        self.enter_node(kind);
        self.visit_span(&class.span);
        self.visit_decorators(&class.decorators);
        if let Some(id) = &class.id {
            self.visit_binding_identifier(id);
        }
        self.push_scope();
        if let Some(id) = &class.id {
            self.add_local(id.name.as_str());
        }
        self.enter_scope(ScopeFlags::StrictMode, &class.scope_id);
        if let Some(parameters) = &class.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(super_class) = &class.super_class {
            self.visit_expression(super_class);
        }
        if let Some(arguments) = &class.super_type_arguments {
            self.visit_ts_type_parameter_instantiation(arguments);
        }
        self.visit_ts_class_implements_list(&class.implements);
        self.visit_class_body(&class.body);
        self.leave_scope();
        self.pop_scope();
        self.leave_node(kind);
    }

    fn scoped_static_block(&mut self, block: &ast::StaticBlock<'a>) {
        self.push_scope();
        crate::function_vars::for_each_var_in_statements(&block.body, |pattern| {
            self.collect_binding_pattern(pattern)
        });
        self.declare_lexical_bindings(&block.body);
        walk::walk_static_block(self, block);
        self.pop_scope();
    }
}
