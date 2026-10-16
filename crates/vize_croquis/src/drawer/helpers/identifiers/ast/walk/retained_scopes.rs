//! Lexical scopes for the demand-only walk; all collectors consume its budget.

use super::{
    BindingPattern, CompactString, FormalParameters, FunctionBody, RetainedIdentifierRef,
    Statement, TokenWalk,
};

impl TokenWalk {
    pub(super) fn parameters(
        &mut self,
        parameters: &FormalParameters<'_>,
    ) -> Option<Vec<RetainedIdentifierRef>> {
        self.within(|walk| {
            if parameters.kind == oxc_ast::ast::FormalParameterKind::Signature {
                return None;
            }
            let mut bindings = Vec::new();
            for parameter in &parameters.items {
                walk.tick()?;
                if !parameter.decorators.is_empty() {
                    return None;
                }
                walk.collect_bindings(&parameter.pattern, &mut bindings)?;
            }
            if let Some(rest) = &parameters.rest {
                walk.tick()?;
                walk.collect_bindings(&rest.rest.argument, &mut bindings)?;
            }
            // Every parameter is local to all default initializers, including
            // references that would throw because of a later parameter's TDZ.
            walk.locals
                .extend(bindings.iter().map(|binding| binding.name.clone()));
            for parameter in &parameters.items {
                walk.binding_expressions(&parameter.pattern)?;
                if let Some(initializer) = &parameter.initializer {
                    walk.expression(initializer)?;
                }
            }
            if let Some(rest) = &parameters.rest {
                walk.binding_expressions(&rest.rest.argument)?;
            }
            Some(bindings)
        })
    }

    pub(super) fn body(&mut self, body: &FunctionBody<'_>) -> Option<()> {
        self.within(|walk| {
            for _ in &body.directives {
                walk.tick()?;
            }
            for statement in &body.statements {
                walk.collect_vars(statement)?;
            }
            walk.block(&body.statements)
        })
    }

    fn block(&mut self, statements: &[Statement<'_>]) -> Option<()> {
        self.within(|walk| {
            let outer = walk.locals.len();
            for statement in statements {
                walk.tick()?;
                if let Statement::VariableDeclaration(declaration) = statement
                    && !declaration.kind.is_var()
                {
                    for declarator in &declaration.declarations {
                        walk.tick()?;
                        walk.bind_local(&declarator.id)?;
                    }
                }
            }
            for statement in statements {
                walk.statement(statement)?;
            }
            walk.locals.truncate(outer);
            Some(())
        })
    }

    fn collect_vars(&mut self, statement: &Statement<'_>) -> Option<()> {
        self.within(|walk| {
            match statement {
                Statement::VariableDeclaration(declaration) if declaration.kind.is_var() => {
                    for declarator in &declaration.declarations {
                        walk.tick()?;
                        walk.bind_local(&declarator.id)?;
                    }
                }
                Statement::BlockStatement(block) => {
                    for statement in &block.body {
                        walk.collect_vars(statement)?;
                    }
                }
                Statement::IfStatement(branch) => {
                    walk.collect_vars(&branch.consequent)?;
                    if let Some(alternate) = &branch.alternate {
                        walk.collect_vars(alternate)?;
                    }
                }
                // Unsupported statements are refused by the statement walk;
                // never recursively enter their potentially unbounded subtrees.
                _ => {}
            }
            Some(())
        })
    }

    fn statement(&mut self, statement: &Statement<'_>) -> Option<()> {
        self.within(|walk| match statement {
            Statement::ExpressionStatement(statement) => walk.expression(&statement.expression),
            Statement::ReturnStatement(statement) => {
                if let Some(argument) = &statement.argument {
                    walk.expression(argument)?;
                }
                Some(())
            }
            Statement::ThrowStatement(statement) => walk.expression(&statement.argument),
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    walk.tick()?;
                    walk.binding_expressions(&declarator.id)?;
                    if let Some(initializer) = &declarator.init {
                        walk.expression(initializer)?;
                    }
                }
                Some(())
            }
            Statement::IfStatement(branch) => {
                walk.expression(&branch.test)?;
                walk.statement(&branch.consequent)?;
                if let Some(alternate) = &branch.alternate {
                    walk.statement(alternate)?;
                }
                Some(())
            }
            Statement::BlockStatement(block) => walk.block(&block.body),
            Statement::EmptyStatement(_) => Some(()),
            _ => None,
        })
    }

    fn bind_local(&mut self, pattern: &BindingPattern<'_>) -> Option<()> {
        let mut bindings = Vec::new();
        self.collect_bindings(pattern, &mut bindings)?;
        self.locals
            .extend(bindings.into_iter().map(|binding| binding.name));
        Some(())
    }

    pub(super) fn collect_bindings(
        &mut self,
        pattern: &BindingPattern<'_>,
        bindings: &mut Vec<RetainedIdentifierRef>,
    ) -> Option<()> {
        self.within(|walk| {
            match pattern {
                BindingPattern::BindingIdentifier(id) => bindings.push(RetainedIdentifierRef {
                    name: CompactString::new(id.name.as_str()),
                    span: id.span,
                }),
                BindingPattern::ObjectPattern(object) => {
                    for property in &object.properties {
                        walk.tick()?;
                        walk.collect_bindings(&property.value, bindings)?;
                    }
                    if let Some(rest) = &object.rest {
                        walk.collect_bindings(&rest.argument, bindings)?;
                    }
                }
                BindingPattern::ArrayPattern(array) => {
                    for element in &array.elements {
                        walk.tick()?;
                        if let Some(element) = element {
                            walk.collect_bindings(element, bindings)?;
                        }
                    }
                    if let Some(rest) = &array.rest {
                        walk.collect_bindings(&rest.argument, bindings)?;
                    }
                }
                BindingPattern::AssignmentPattern(assignment) => {
                    walk.collect_bindings(&assignment.left, bindings)?;
                }
            }
            Some(())
        })
    }

    fn binding_expressions(&mut self, pattern: &BindingPattern<'_>) -> Option<()> {
        self.within(|walk| {
            match pattern {
                BindingPattern::BindingIdentifier(_) => {}
                BindingPattern::ObjectPattern(object) => {
                    for property in &object.properties {
                        walk.tick()?;
                        if property.computed {
                            walk.expression(property.key.as_expression()?)?;
                        }
                        walk.binding_expressions(&property.value)?;
                    }
                    if let Some(rest) = &object.rest {
                        walk.binding_expressions(&rest.argument)?;
                    }
                }
                BindingPattern::ArrayPattern(array) => {
                    for element in &array.elements {
                        walk.tick()?;
                        if let Some(element) = element {
                            walk.binding_expressions(element)?;
                        }
                    }
                    if let Some(rest) = &array.rest {
                        walk.binding_expressions(&rest.argument)?;
                    }
                }
                BindingPattern::AssignmentPattern(assignment) => {
                    walk.binding_expressions(&assignment.left)?;
                    walk.expression(&assignment.right)?;
                }
            }
            Some(())
        })
    }
}
