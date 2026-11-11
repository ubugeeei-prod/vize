use super::{Kind, Visitor};
use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty, CallExpression,
    Expression, Function, MemberExpression, Program, Statement, VariableDeclarator,
};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{
    walk_arrow_function_expression, walk_binding_pattern, walk_block_statement,
    walk_call_expression, walk_computed_member_expression, walk_function, walk_function_body,
    walk_program, walk_static_member_expression,
};
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use vize_l0::FxHashMap;

impl<'a> Visit<'a> for Visitor<'_, 'a> {
    fn visit_program(&mut self, it: &Program<'a>) {
        self.frames.push(FxHashMap::default());
        if let Some(sibling) = self.sibling_program {
            for statement in &sibling.body {
                if let Statement::ImportDeclaration(import) = statement {
                    self.record_import(import);
                }
            }
        }
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

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        match it.callee.get_inner_expression() {
            Expression::ArrowFunctionExpression(arrow) if !arrow.r#async => {
                self.frames.push(FxHashMap::default());
                self.bind_params(&arrow.params);
                walk_arrow_function_expression(self, arrow);
                self.frames.pop();
                for argument in &it.arguments {
                    self.visit_argument(argument);
                }
                return;
            }
            Expression::FunctionExpression(function)
                if !function.r#async && !function.generator =>
            {
                self.frames.push(FxHashMap::default());
                if let Some(id) = &function.id {
                    self.insert(id.name.as_str(), Kind::Other);
                }
                self.bind_params(&function.params);
                walk_function(self, function, ScopeFlags::Function);
                self.frames.pop();
                for argument in &it.arguments {
                    self.visit_argument(argument);
                }
                return;
            }
            _ => {}
        }
        walk_call_expression(self, it);
    }

    fn visit_static_member_expression(&mut self, it: &oxc_ast::ast::StaticMemberExpression<'a>) {
        if it.property.name == "value" && self.same_scope_ref(&it.object) {
            self.report(it.object.span());
        }
        walk_static_member_expression(self, it);
    }

    fn visit_computed_member_expression(
        &mut self,
        it: &oxc_ast::ast::ComputedMemberExpression<'a>,
    ) {
        if it.static_property_name().as_deref() == Some("value") && self.same_scope_ref(&it.object)
        {
            self.report(it.object.span());
        }
        walk_computed_member_expression(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &oxc_ast::ast::AssignmentExpression<'a>) {
        self.write_target(&it.left);
        self.visit_expression(&it.right);
        if let AssignmentTarget::ObjectAssignmentTarget(object) = &it.left
            && object.properties.iter().any(copies_value)
            && self.same_scope_ref(&it.right)
        {
            self.report(object.span);
        }
        if let AssignmentTarget::AssignmentTargetIdentifier(identifier) = &it.left
            && let Some(kind) = self.assigned_kind(&it.right)
        {
            self.assign(identifier.name.as_str(), kind);
        }
    }

    fn visit_update_expression(&mut self, it: &oxc_ast::ast::UpdateExpression<'a>) {
        if let Some(member) = it.argument.as_member_expression() {
            self.write_member(member);
        } else {
            self.visit_simple_assignment_target(&it.argument);
        }
    }
}

impl<'a> Visitor<'_, 'a> {
    /// Visit a write target: the written member chain is not a read, while
    /// computed keys, call arguments and defaults inside it still are.
    fn write_target(&mut self, target: &AssignmentTarget<'a>) {
        if let Some(member) = target.as_member_expression() {
            self.write_member(member);
            return;
        }
        match target {
            AssignmentTarget::ArrayAssignmentTarget(array) => {
                for element in array.elements.iter().flatten() {
                    self.write_maybe_default(element);
                }
                if let Some(rest) = &array.rest {
                    self.write_target(&rest.target);
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(object) => {
                for property in &object.properties {
                    match property {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                            if let Some(init) = &property.init {
                                self.visit_expression(init);
                            }
                        }
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                            self.visit_property_key(&property.name);
                            self.write_maybe_default(&property.binding);
                        }
                    }
                }
                if let Some(rest) = &object.rest {
                    self.write_target(&rest.target);
                }
            }
            AssignmentTarget::TSAsExpression(expression) => {
                self.write_expression_target(&expression.expression)
            }
            AssignmentTarget::TSSatisfiesExpression(expression) => {
                self.write_expression_target(&expression.expression)
            }
            AssignmentTarget::TSNonNullExpression(expression) => {
                self.write_expression_target(&expression.expression)
            }
            AssignmentTarget::TSTypeAssertion(expression) => {
                self.write_expression_target(&expression.expression)
            }
            other => self.visit_assignment_target(other),
        }
    }

    /// Visit a write target hidden by a TypeScript expression wrapper.
    fn write_expression_target(&mut self, target: &Expression<'a>) {
        let target = target.without_parentheses();
        if let Some(member) = target.as_member_expression() {
            self.write_member(member);
            return;
        }
        match target {
            Expression::TSAsExpression(expression) => {
                self.write_expression_target(&expression.expression)
            }
            Expression::TSSatisfiesExpression(expression) => {
                self.write_expression_target(&expression.expression)
            }
            Expression::TSNonNullExpression(expression) => {
                self.write_expression_target(&expression.expression)
            }
            Expression::TSTypeAssertion(expression) => {
                self.write_expression_target(&expression.expression)
            }
            Expression::Identifier(_) => {}
            target => self.visit_expression(target),
        }
    }

    fn write_maybe_default(&mut self, target: &AssignmentTargetMaybeDefault<'a>) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
                self.write_target(&target.binding);
                self.visit_expression(&target.init);
            }
            other => {
                if let Some(target) = other.as_assignment_target() {
                    self.write_target(target);
                }
            }
        }
    }

    fn write_member(&mut self, member: &MemberExpression<'a>) {
        // ESTree has no parentheses; a type assertion ends the written chain.
        let object = member.object().without_parentheses();
        if let Some(object) = object.as_member_expression() {
            self.write_member(object);
        } else {
            self.visit_expression(object);
        }
        if let MemberExpression::ComputedMemberExpression(computed) = member {
            self.visit_expression(&computed.expression);
        }
    }
}

fn copies_value(property: &AssignmentTargetProperty<'_>) -> bool {
    match property {
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
            property.binding.name == "value"
        }
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
            property.name.static_name().as_deref() == Some("value")
        }
    }
}
