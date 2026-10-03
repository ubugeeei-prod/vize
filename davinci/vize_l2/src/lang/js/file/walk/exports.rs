//! Source-linked export records; missing reference support stays explicit.

use super::{Context, Walk, imports::namespace};
use crate::file::{Export, Namespace};
use crate::lang::js::file::observer::FileObserver;
use crate::resolution::BindingId;
use oxc_ast::ast::{
    ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration, Expression,
};
use oxc_span::GetSpan;
use vize_l0::{Span, String};

impl<'a, O: FileObserver<'a>> Walk<'_, '_, 'a, O> {
    pub(super) fn push_export(
        &mut self,
        name: &str,
        local: Option<BindingId>,
        source: Option<String>,
        namespace: Namespace,
        span: Span,
    ) -> usize {
        let index = self.facts.exports.len();
        self.facts.exports.push(Export {
            unit: self.unit,
            name: String::from(name),
            local,
            source,
            namespace,
            span,
            local_reference: None,
        });
        index
    }

    pub(super) fn export_named(&mut self, export: &ExportNamedDeclaration<'a>) {
        if let Some(declaration) = &export.declaration {
            self.declaration(declaration, true);
        }
        if export.with_clause.is_some() {
            self.unsupported(export.span);
        }
        for specifier in &export.specifiers {
            let Some(span) = self.span(specifier.span) else {
                continue;
            };
            let namespace = if export.export_kind == oxc_ast::ast::ImportOrExportKind::Type {
                Namespace::Type
            } else {
                namespace(specifier.export_kind)
            };
            let index = self.push_export(
                specifier.exported.name().as_str(),
                None,
                export
                    .source
                    .as_ref()
                    .map(|value| String::from(value.value.as_str())),
                namespace,
                span,
            );
            if export.source.is_none() {
                let reference = self.export_reference(&specifier.local, namespace);
                if let Some(export) = self.facts.exports.get_mut(index) {
                    export.local_reference = reference;
                }
            }
        }
    }

    pub(super) fn export_default(&mut self, export: &ExportDefaultDeclaration<'a>) {
        self.ordinary_empty_default(export);
        let Some(span) = self.span(export.span) else {
            return;
        };
        self.push_export("default", None, None, Namespace::Value, span);
        if let Some(expression) = export.declaration.as_expression() {
            self.expression(expression);
        } else {
            self.unsupported(export.declaration.span());
        }
    }

    fn ordinary_empty_default(&mut self, export: &ExportDefaultDeclaration<'a>) {
        let valid = self.context == Context::Unit
            && export.declaration.as_expression().is_some_and(|expression| {
                matches!(expression, Expression::ObjectExpression(object) if object.properties.is_empty())
            });
        let receipt = self.input.references.admitted().sole_default_export();
        let valid = valid
            && receipt.as_ref().is_some_and(|receipt| {
                receipt.statement_span() == export.span
                    && receipt.declaration_span() == export.declaration.span()
            });
        if !valid {
            super::super::ordinary::reject(self.facts, self.unit);
            return;
        }
        let Some(receipt) = receipt else { return };
        let statement = self.span(receipt.statement_span());
        let export_keyword = receipt
            .statement_span()
            .start
            .checked_add(6)
            .and_then(|end| self.span(oxc_span::Span::new(receipt.statement_span().start, end)));
        let keyword = self.span(receipt.default_keyword_span());
        let object = self.span(receipt.declaration_span());
        if !matches!((statement, export_keyword, keyword, object),
            (Some(statement), Some(export_keyword), Some(keyword), Some(object))
            if statement.start == export_keyword.start && export_keyword.end <= keyword.start
                && keyword.end.checked_sub(keyword.start) == Some(7)
                && keyword.end <= object.start && object.start < object.end
                && object.end <= statement.end)
        {
            super::super::ordinary::reject(self.facts, self.unit);
        }
    }

    pub(super) fn export_all(&mut self, export: &ExportAllDeclaration<'a>) {
        let Some(span) = self.span(export.span) else {
            return;
        };
        self.push_export(
            export
                .exported
                .as_ref()
                .map_or("*", |value| value.name().as_str()),
            None,
            Some(String::from(export.source.value.as_str())),
            namespace(export.export_kind),
            span,
        );
        if export.with_clause.is_some() {
            self.unsupported(export.span);
        }
    }
}
