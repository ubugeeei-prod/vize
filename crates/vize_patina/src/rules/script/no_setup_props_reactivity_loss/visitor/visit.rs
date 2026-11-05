use super::{Kind, ObjectFrame, Visitor};
use oxc_ast::ast::{Argument, ExportDefaultDeclarationKind, ObjectPropertyKind, Program};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{
    walk_arrow_function_expression, walk_binding_pattern, walk_block_statement,
    walk_export_default_declaration, walk_function, walk_function_body, walk_new_expression,
    walk_program,
};
use oxc_syntax::scope::ScopeFlags;
use vize_l0::{CompactString, FxHashMap};

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_program(&mut self, it: &Program<'a>) {
        self.frames.push(FxHashMap::default());
        walk_program(self, it);
        self.frames.pop();
    }

    fn visit_import_declaration(&mut self, it: &oxc_ast::ast::ImportDeclaration<'a>) {
        self.record_import(it);
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

    fn visit_object_expression(&mut self, it: &oxc_ast::ast::ObjectExpression<'a>) {
        let is_vue = self.force_vue;
        self.force_vue = false;
        self.objects.push(ObjectFrame {
            is_vue,
            depth: self.fn_depth,
            key: None,
        });
        for property in &it.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                if let ObjectPropertyKind::SpreadProperty(spread) = property {
                    self.visit_expression(&spread.argument);
                }
                continue;
            };
            self.visit_property_key(&property.key);
            if let Some(frame) = self.objects.last_mut() {
                frame.key = property
                    .key
                    .static_name()
                    .map(|name| CompactString::from(name.as_ref()));
            }
            self.visit_expression(&property.value);
            if let Some(frame) = self.objects.last_mut() {
                frame.key = None;
            }
        }
        self.objects.pop();
    }

    fn visit_call_expression(&mut self, it: &oxc_ast::ast::CallExpression<'a>) {
        if let Some(scope) = self.call_access(it) {
            self.report_get(it.span, scope);
        }
        self.mark_vue_argument(it);
        self.visit_expression(&it.callee);
        for argument in &it.arguments {
            if let Some(expression) = argument.as_expression() {
                self.visit_expression(expression);
            }
        }
    }

    fn visit_new_expression(&mut self, it: &oxc_ast::ast::NewExpression<'a>) {
        if matches!(Self::peel(&it.callee), oxc_ast::ast::Expression::Identifier(id) if id.name == "Vue")
            && matches!(it.arguments.first(), Some(Argument::ObjectExpression(_)))
        {
            self.force_vue = true;
        }
        walk_new_expression(self, it);
    }

    fn visit_function(&mut self, it: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
        let setup = self.in_setup_property();
        self.enter_function(&it.params, setup);
        if let Some(id) = &it.id {
            self.insert(id.name.as_str(), Kind::Other);
        }
        walk_function(self, it, flags);
        self.exit_function();
    }

    fn visit_arrow_function_expression(&mut self, it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
        let setup = self.in_setup_property();
        self.enter_function(&it.params, setup);
        walk_arrow_function_expression(self, it);
        self.exit_function();
    }

    fn visit_function_body(&mut self, it: &oxc_ast::ast::FunctionBody<'a>) {
        self.frames.push(FxHashMap::default());
        walk_function_body(self, it);
        self.frames.pop();
    }

    fn visit_block_statement(&mut self, it: &oxc_ast::ast::BlockStatement<'a>) {
        self.scope_id += 1;
        self.frames.push(FxHashMap::default());
        walk_block_statement(self, it);
        self.frames.pop();
        self.scope_id -= 1;
    }

    fn visit_variable_declarator(&mut self, it: &oxc_ast::ast::VariableDeclarator<'a>) {
        if let Some(init) = &it.init {
            self.verify_value(Self::binding_pattern_span(&it.id), init);
        }
        if it
            .init
            .as_ref()
            .is_some_and(|init| self.is_define_props(init))
        {
            self.bind_define_props(&it.id);
        } else {
            self.bind_other(&it.id);
        }
        if let Some(init) = &it.init {
            self.visit_expression(init);
        }
        walk_binding_pattern(self, &it.id);
    }

    fn visit_assignment_expression(&mut self, it: &oxc_ast::ast::AssignmentExpression<'a>) {
        self.verify_value(Self::pattern_span(&it.left), &it.right);
        self.visit_expression(&it.right);
        self.visit_assignment_target(&it.left);
    }
}
