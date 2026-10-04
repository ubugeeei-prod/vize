//! One statement/declaration traversal; incomplete subtrees are never guessed.

use super::ProgramInput;
use super::observer::{DeclaredEvent, FileObserver, StatementEvent};
use crate::file::build::{DeclarationSite, Facts};
use crate::file::{
    DeclarationKind, FileIssueKind, InitializerKind, Namespace, ScopeId, ScriptUnitId,
};
use core::num::NonZeroU32;
use oxc_ast::ast::{
    BindingIdentifier, BindingPattern, Declaration, Expression, Statement, VariableDeclaration,
    VariableDeclarationKind,
};
use oxc_span::GetSpan;
use vize_l0::Span;

mod annotations;
mod exports;
mod functions;
mod imports;
mod references;

pub(super) fn program<'a, O: FileObserver<'a>>(
    input: &ProgramInput<'_, 'a>,
    facts: &mut Facts<'a>,
    unit: ScriptUnitId,
    scope: ScopeId,
    observer: &mut O,
) {
    let first_reference = facts.references.len();
    let first_export = facts.exports.len();
    let mut walk = Walk {
        input,
        facts,
        unit,
        scope,
        observer,
        context: Context::Unit,
        root_statement: None,
    };
    for (index, statement) in input.references.program().body.iter().enumerate() {
        walk.root_statement = u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1))
            .and_then(NonZeroU32::new);
        if walk.root_statement.is_none() {
            walk.facts
                .issue(unit, input.block.span(), FileIssueKind::BindingLimit);
            break;
        }
        walk.statement(statement);
    }
    walk.facts.resolve_references(first_reference, first_export);
}

struct Walk<'f, 'p, 'a, O> {
    input: &'f ProgramInput<'p, 'a>,
    facts: &'f mut Facts<'a>,
    unit: ScriptUnitId,
    scope: ScopeId,
    observer: &'f mut O,
    context: Context,
    root_statement: Option<NonZeroU32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    Unit,
    Function,
}

impl<'a, O: FileObserver<'a>> Walk<'_, '_, 'a, O> {
    fn source_site(
        &mut self,
        literal: &oxc_ast::ast::StringLiteral<'a>,
    ) -> Option<crate::file::SourceSite> {
        self.span(literal.span)?;
        if self.context != Context::Unit {
            self.unsupported(literal.span);
            return None;
        }
        Some(crate::file::SourceSite {
            statement: self.root_statement?,
        })
    }

    fn span(&mut self, span: oxc_span::Span) -> Option<Span> {
        let Some(authored) = self
            .input
            .references
            .authored_span(Span::new(span.start, span.end))
        else {
            self.facts.issue(
                self.unit,
                self.input.block.span(),
                FileIssueKind::InvalidSpan,
            );
            return None;
        };
        Some(authored)
    }

    fn unsupported(&mut self, span: oxc_span::Span) {
        if let Some(span) = self.span(span) {
            self.facts
                .issue(self.unit, span, FileIssueKind::UnsupportedSyntax);
        }
    }

    fn statement(&mut self, statement: &Statement<'a>) {
        if self.context != Context::Unit
            || !matches!(
                statement,
                Statement::ExportDefaultDeclaration(_) | Statement::EmptyStatement(_)
            )
        {
            super::ordinary::reject(self.facts, self.unit);
        }
        if self.context != Context::Unit
            || !matches!(
                statement,
                Statement::VariableDeclaration(_) | Statement::EmptyStatement(_)
            )
        {
            super::setup::reject(self.facts, self.unit);
        }
        let Some(span) = self.span(statement.span()) else {
            return;
        };
        self.observer.statement(StatementEvent {
            unit: self.unit,
            scope: self.scope,
            span,
            statement,
        });
        if matches!(
            statement,
            Statement::ExportNamedDeclaration(_)
                | Statement::ExportDefaultDeclaration(_)
                | Statement::ExportAllDeclaration(_)
        ) && let Some(unit) = self
            .facts
            .units
            .iter_mut()
            .find(|unit| unit.id == self.unit)
        {
            unit.origin.has_export = true;
        }
        if self.context == Context::Function
            && !matches!(
                statement,
                Statement::VariableDeclaration(_)
                    | Statement::EmptyStatement(_)
                    | Statement::ExpressionStatement(_)
                    | Statement::ReturnStatement(_)
            )
        {
            self.unsupported(statement.span());
            return;
        }
        match statement {
            Statement::VariableDeclaration(value) => self.variable(value, false),
            Statement::ImportDeclaration(value) => self.import(value),
            Statement::ExportNamedDeclaration(value) => self.export_named(value),
            Statement::ExportDefaultDeclaration(value) => self.export_default(value),
            Statement::ExportAllDeclaration(value) => self.export_all(value),
            Statement::FunctionDeclaration(value) => self.function(value, false),
            Statement::ClassDeclaration(value) => {
                if let Some(id) = &value.id {
                    self.binding(id, DeclarationKind::Class, InitializerKind::Unknown, None);
                }
                self.unsupported(value.span);
            }
            Statement::EmptyStatement(_) => {}
            Statement::ExpressionStatement(value) => self.expression(&value.expression),
            Statement::ReturnStatement(value) if self.context == Context::Function => {
                if let Some(expression) = &value.argument {
                    self.expression(expression);
                }
            }
            other => self.unsupported(other.span()),
        }
    }

    fn declaration(&mut self, declaration: &Declaration<'a>, exported: bool) {
        match declaration {
            Declaration::VariableDeclaration(value) => self.variable(value, exported),
            Declaration::FunctionDeclaration(value) => self.function(value, exported),
            other => self.unsupported(other.span()),
        }
    }

    fn variable(&mut self, value: &VariableDeclaration<'a>, exported: bool) {
        if self.context != Context::Unit
            || exported
            || value.declare
            || !matches!(
                value.kind,
                VariableDeclarationKind::Const
                    | VariableDeclarationKind::Let
                    | VariableDeclarationKind::Var
            )
        {
            super::setup::reject(self.facts, self.unit);
        }
        let kind = match value.kind {
            VariableDeclarationKind::Const => DeclarationKind::Const,
            VariableDeclarationKind::Let => DeclarationKind::Let,
            VariableDeclarationKind::Var => DeclarationKind::Var,
            _ => {
                self.unsupported(value.span);
                return;
            }
        };
        if value.declare {
            self.unsupported(value.span);
            return;
        }
        for declaration in &value.declarations {
            let BindingPattern::BindingIdentifier(id) = &declaration.id else {
                super::setup::reject(self.facts, self.unit);
                self.unsupported(declaration.span);
                continue;
            };
            let initializer = match &declaration.init {
                Some(value) => initializer_kind(value),
                _ => InitializerKind::Unknown,
            };
            let type_supported = self.type_annotation(
                declaration,
                self.context == Context::Unit && !exported,
                initializer.is_primitive(),
            );
            if !initializer.is_primitive()
                || !type_supported
                || declaration.definite
                || matches!(
                    id.name.as_str(),
                    "eval"
                        | "arguments"
                        | "implements"
                        | "interface"
                        | "let"
                        | "package"
                        | "private"
                        | "protected"
                        | "public"
                        | "static"
                        | "yield"
                )
            {
                super::setup::reject(self.facts, self.unit);
            }
            let binding = self.binding(id, kind, initializer, declaration.init.as_ref());
            if exported && let Some(span) = self.span(id.span) {
                self.push_export(id.name.as_str(), binding, None, Namespace::Value, span);
            }
            if !type_supported {
                self.unsupported(declaration.span);
            }
            if let Some(expression) = &declaration.init {
                self.expression(expression);
            }
        }
    }

    fn binding(
        &mut self,
        id: &BindingIdentifier<'_>,
        kind: DeclarationKind,
        initializer: InitializerKind,
        expression: Option<&Expression<'a>>,
    ) -> Option<crate::resolution::BindingId> {
        let span = self.span(id.span)?;
        let binding = self.facts.declare(DeclarationSite {
            unit: self.unit,
            scope: self.scope,
            name: id.name.as_str(),
            span,
            namespace: Namespace::Value,
            kind,
            initializer,
            import_source: None,
            imported_name: None,
            direct_program: self.context == Context::Unit,
        })?;
        let initializer = match expression {
            Some(expression) => self.span(expression.span()).map(|span| (expression, span)),
            None => None,
        };
        if let Some(declaration) = self.facts.declarations.get(binding.index() as usize) {
            self.observer.declared(DeclaredEvent {
                declaration,
                initializer,
            });
        }
        Some(binding)
    }
}

fn initializer_kind(expression: &Expression<'_>) -> InitializerKind {
    match expression {
        Expression::StringLiteral(value) if value.lone_surrogates => {
            InitializerKind::PrimitiveStringWithLoneSurrogates
        }
        Expression::StringLiteral(value)
            if value
                .value
                .as_bytes()
                .iter()
                .any(|byte| matches!(*byte, b'\0' | b'\r')) =>
        {
            InitializerKind::PrimitiveStringWithNulOrCr
        }
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::StringLiteral(_) => InitializerKind::PrimitiveLiteral,
        _ => InitializerKind::Unknown,
    }
}
