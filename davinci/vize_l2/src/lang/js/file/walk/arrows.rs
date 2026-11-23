//! Lexical scopes of original, synchronous arrow initializers in module declarations.
use super::{Context, FileObserver, Walk, annotations};
use crate::file::{DeclarationKind, InitializerKind};
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, FormalParameterKind, Statement, TSTypeAnnotation,
};
use oxc_span::GetSpan;

impl<'a, O: FileObserver<'a>> Walk<'_, '_, 'a, O> {
    pub(super) fn arrow_initializer(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        let profile = self.input.source_type();
        if !profile.is_module()
            || profile.is_typescript_definition()
            || profile.is_jsx()
            || arrow.r#async
            || arrow.type_parameters.is_some()
            || arrow.params.kind != FormalParameterKind::ArrowFormalParameters
            || arrow.params.rest.is_some()
            || !arrow.body.directives.is_empty()
            || !self.arrow_annotation(arrow.return_type.as_deref())
        {
            self.unsupported(arrow.span);
            return;
        }
        let Some(span) = self.span(arrow.span) else {
            return;
        };
        let Some(scope) = self.facts.child_scope(self.unit, self.scope, span) else {
            return;
        };
        let mut child = Walk {
            input: self.input,
            facts: self.facts,
            unit: self.unit,
            scope,
            observer: self.observer,
            context: Context::Function,
            root_statement: None,
        };
        for parameter in &arrow.params.items {
            let BindingPattern::BindingIdentifier(id) = &parameter.pattern else {
                child.unsupported(parameter.span);
                continue;
            };
            if parameter.optional
                || parameter.initializer.is_some()
                || !parameter.decorators.is_empty()
                || parameter.accessibility.is_some()
                || parameter.readonly
                || parameter.r#override
                || !child.arrow_annotation(parameter.type_annotation.as_deref())
            {
                child.unsupported(parameter.span);
                continue;
            }
            child.binding(
                id,
                DeclarationKind::Parameter,
                InitializerKind::Unknown,
                None,
            );
        }
        if arrow.expression {
            // OXC stores a concise body in an expression-statement wrapper.
            // Only the enclosed expression is an authored operand.
            let [Statement::ExpressionStatement(body)] = arrow.body.statements.as_slice() else {
                child.unsupported(arrow.body.span);
                return;
            };
            child.expression(&body.expression);
        } else {
            for statement in &arrow.body.statements {
                child.statement(statement);
            }
        }
    }

    fn arrow_annotation(&mut self, annotation: Option<&TSTypeAnnotation<'_>>) -> bool {
        annotation.is_none_or(|ty| {
            self.input.source_type().is_typescript()
                && annotations::is_primitive_type(&ty.type_annotation)
                && self.span(ty.span).is_some()
                && self.span(ty.type_annotation.span()).is_some()
        })
    }
}
