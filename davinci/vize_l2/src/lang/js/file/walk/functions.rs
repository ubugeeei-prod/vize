//! Genuine function scopes/parameters in the original statement traversal.

use super::{Context, Walk};
use crate::file::{DeclarationKind, InitializerKind, Namespace};
use crate::lang::js::file::observer::FileObserver;
use oxc_ast::ast::{BindingPattern, FormalParameterKind, Function, FunctionType};
use oxc_span::GetSpan;

impl<'a, O: FileObserver<'a>> Walk<'_, '_, 'a, O> {
    pub(super) fn function(&mut self, function: &Function<'a>, exported: bool) {
        let Some(id) = &function.id else {
            self.unsupported(function.span);
            return;
        };
        let binding = self.binding(
            id,
            DeclarationKind::Function,
            InitializerKind::Function,
            None,
        );
        if exported && let Some(span) = self.span(id.span) {
            self.push_export(id.name.as_str(), binding, None, Namespace::Value, span);
        }
        let Some(body) = &function.body else {
            self.unsupported(function.span);
            return;
        };
        if self.context != Context::Unit
            || function.r#type != FunctionType::FunctionDeclaration
            || function.r#async
            || function.generator
            || function.declare
            || function.type_parameters.is_some()
            || function.this_param.is_some()
            || function.return_type.as_ref().is_some_and(|annotation| {
                // Original keyword types have no type/value namespace reads.
                // They grant neither a setup annotation nor runtime erasure.
                let profile = self.input.source_type();
                !profile.is_typescript()
                    || !profile.is_module()
                    || profile.is_typescript_definition()
                    || !super::annotations::is_primitive_type(&annotation.type_annotation)
                    || self.span(annotation.span).is_none()
                    || self.span(annotation.type_annotation.span()).is_none()
            })
            || function.params.kind != FormalParameterKind::FormalParameter
            || function.params.rest.is_some()
            || !body.directives.is_empty()
        {
            self.unsupported(function.span);
            return;
        }
        let Some(span) = self.span(function.span) else {
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
        // Original parameter enumeration creates declarations, never a name list.
        for parameter in &function.params.items {
            let BindingPattern::BindingIdentifier(id) = &parameter.pattern else {
                child.unsupported(parameter.span);
                continue;
            };
            // Only reference-free original keyword types need no namespace
            // walk. This grants neither a setup annotation nor runtime erasure.
            let annotation_supported =
                parameter.type_annotation.as_ref().is_none_or(|annotation| {
                    let profile = child.input.source_type();
                    (!exported || !parameter.optional)
                        && profile.is_typescript()
                        && profile.is_module()
                        && !profile.is_typescript_definition()
                        && super::annotations::is_primitive_type(&annotation.type_annotation)
                        && child.span(annotation.span).is_some()
                        && child.span(annotation.type_annotation.span()).is_some()
                });
            if !parameter.decorators.is_empty()
                || !annotation_supported
                || parameter.initializer.is_some()
                || (parameter.optional
                    && (parameter.type_annotation.is_none()
                        || child.span(parameter.span).is_none()))
                || parameter.accessibility.is_some()
                || parameter.readonly
                || parameter.r#override
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
        for statement in &body.statements {
            child.statement(statement);
        }
    }
}
