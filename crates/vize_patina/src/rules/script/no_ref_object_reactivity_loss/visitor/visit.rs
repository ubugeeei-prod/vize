use super::{Kind, Visitor};
use oxc_ast::ast::{Function, Program, Statement, VariableDeclarator};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{
    walk_arrow_function_expression, walk_binding_pattern, walk_block_statement,
    walk_computed_member_expression, walk_function, walk_function_body, walk_program,
    walk_static_member_expression,
};
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use vize_l0::FxHashMap;

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_program(&mut self, it: &Program<'a>) {
        self.frames.push(FxHashMap::default());
        for statement in &it.body {
            if let Statement::ImportDeclaration(import) = statement {
                self.record_import(import);
            }
        }
        walk_program(self, it);
        self.frames.pop();
    }

    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        self.fn_depth += 1;
        self.frames.push(FxHashMap::default());
        if let Some(id) = &it.id {
            self.insert(id.name.as_str(), Kind::Other);
        }
        self.bind_params(&it.params);
        walk_function(self, it, flags);
        self.frames.pop();
        self.fn_depth -= 1;
    }

    fn visit_function_body(&mut self, it: &oxc_ast::ast::FunctionBody<'a>) {
        self.frames.push(FxHashMap::default());
        walk_function_body(self, it);
        self.frames.pop();
    }

    fn visit_arrow_function_expression(&mut self, it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
        self.fn_depth += 1;
        self.frames.push(FxHashMap::default());
        self.bind_params(&it.params);
        walk_arrow_function_expression(self, it);
        self.frames.pop();
        self.fn_depth -= 1;
    }

    fn visit_block_statement(&mut self, it: &oxc_ast::ast::BlockStatement<'a>) {
        self.frames.push(FxHashMap::default());
        walk_block_statement(self, it);
        self.frames.pop();
    }

    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init {
            self.visit_expression(init);
        }
        self.record(&it.id, it.init.as_ref());
        walk_binding_pattern(self, &it.id);
    }

    fn visit_static_member_expression(&mut self, it: &oxc_ast::ast::StaticMemberExpression<'a>) {
        if !self.writing && it.property.name == "value" && self.same_scope_ref(&it.object) {
            self.report(it.object.span());
        }
        walk_static_member_expression(self, it);
    }

    fn visit_computed_member_expression(
        &mut self,
        it: &oxc_ast::ast::ComputedMemberExpression<'a>,
    ) {
        if !self.writing
            && it.static_property_name().as_deref() == Some("value")
            && self.same_scope_ref(&it.object)
        {
            self.report(it.object.span());
        }
        walk_computed_member_expression(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &oxc_ast::ast::AssignmentExpression<'a>) {
        let writing = self.writing;
        self.writing = true;
        self.visit_assignment_target(&it.left);
        self.writing = writing;
        self.visit_expression(&it.right);
    }

    fn visit_update_expression(&mut self, it: &oxc_ast::ast::UpdateExpression<'a>) {
        let writing = self.writing;
        self.writing = true;
        self.visit_simple_assignment_target(&it.argument);
        self.writing = writing;
    }
}
