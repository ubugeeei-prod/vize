//! One original whole-body walk, sharing the expression traversal budget.

use super::Resolver;
use crate::resolution::handler::{HandlerDeclarationKind, sink::HandlerScopeSink};
use crate::resolution::source::HandlerReferenceSource;
use crate::resolution::{ResolutionError, ResolutionErrorKind};
use oxc_ast::ast::{
    BindingPattern, FunctionBody, Statement, VariableDeclaration, VariableDeclarationKind,
};
use oxc_span::GetSpan;

pub(in crate::resolution) fn handler_body<'a>(
    source: &HandlerReferenceSource<'_, 'a>,
    sink: &mut impl HandlerScopeSink<'a>,
) -> Result<(), ResolutionError> {
    let checkpoint = sink.checkpoint();
    let mut resolver = Resolver::new(source.coordinates(), sink);
    // The generated FunctionBody span is deliberately never projected.
    let result = resolver.original_handler(source.body());
    if result.is_err() {
        resolver.sink.rollback(checkpoint);
    }
    result
}

impl<'a, S: HandlerScopeSink<'a>> Resolver<'a, '_, S> {
    fn original_handler(&mut self, body: &FunctionBody<'a>) -> Result<(), ResolutionError> {
        self.sink.observe_body(
            body.directives.is_empty()
                && matches!(
                    body.statements.first(),
                    Some(Statement::VariableDeclaration(_))
                ),
        );
        for directive in &body.directives {
            self.visit(directive.span, 0)?;
        }
        for statement in &body.statements {
            self.handler_statement(statement, 0)?;
        }
        Ok(())
    }

    fn handler_statement(
        &mut self,
        statement: &Statement<'a>,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        let span = self.visit(statement.span(), depth)?;
        let next = depth + 1;
        match statement {
            Statement::EmptyStatement(_) => Ok(()),
            Statement::ExpressionStatement(value) => self.expression(&value.expression, next),
            Statement::ReturnStatement(value) => {
                self.sink.observe_return(span);
                if let Some(argument) = &value.argument {
                    self.expression(argument, next)?;
                }
                Ok(())
            }
            Statement::VariableDeclaration(value) => self.handler_variable(value, next),
            Statement::BlockStatement(value) => {
                self.sink
                    .enter_block(span)
                    .map_err(|kind| self.fail(value.span, kind))?;
                let result = value
                    .body
                    .iter()
                    .try_for_each(|statement| self.handler_statement(statement, next));
                self.sink.leave_block();
                result
            }
            Statement::IfStatement(value) => {
                self.expression(&value.test, next)?;
                self.handler_statement(&value.consequent, next)?;
                if let Some(alternate) = &value.alternate {
                    self.handler_statement(alternate, next)?;
                }
                Ok(())
            }
            other => Err(self.fail(other.span(), ResolutionErrorKind::UnsupportedSyntax)),
        }
    }

    fn handler_variable(
        &mut self,
        value: &VariableDeclaration<'a>,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        let kind = match value.kind {
            VariableDeclarationKind::Var => HandlerDeclarationKind::Var,
            VariableDeclarationKind::Let => HandlerDeclarationKind::Let,
            VariableDeclarationKind::Const => HandlerDeclarationKind::Const,
            _ => return Err(self.fail(value.span, ResolutionErrorKind::UnsupportedSyntax)),
        };
        if value.declare {
            return Err(self.fail(value.span, ResolutionErrorKind::UnsupportedSyntax));
        }
        for declaration in &value.declarations {
            self.visit(declaration.span, depth)?;
            let BindingPattern::BindingIdentifier(identifier) = &declaration.id else {
                return Err(self.fail(
                    declaration.id.span(),
                    ResolutionErrorKind::UnsupportedSyntax,
                ));
            };
            if declaration.type_annotation.is_some() || declaration.definite {
                return Err(self.fail(declaration.span, ResolutionErrorKind::UnsupportedSyntax));
            }
            let span = self.visit(identifier.span, depth)?;
            self.sink
                .declare(identifier.name.as_str(), span, kind)
                .map_err(|kind| self.fail(identifier.span, kind))?;
            if let Some(initializer) = &declaration.init {
                self.expression(initializer, depth + 1)?;
            }
        }
        Ok(())
    }
}
