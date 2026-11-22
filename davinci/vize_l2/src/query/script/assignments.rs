//! Per-initializer judgments for explicit primitive annotations. Every skipped
//! statement or declarator has an original span. No judgment for a missing
//! initializer, a type alias, a recovered parse or an unmodeled expression.
use super::{DeclarationSyntaxError, ScriptDeclarations};
use alloc::vec::Vec;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, Statement, TSType, VariableDeclaration,
    VariableDeclarationKind, VariableDeclarator,
};
use oxc_span::GetSpan;
use vize_l0::{SourceBlock, Span};
use vize_l1::embed::authored_program::AuthoredProgram;

mod relation;
pub use relation::{Primitive, Verdict};
mod verify;
pub use verify::{AssignmentError, CheckedAssignments};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptRefusal {
    Grammar,
    Parse(DeclarationSyntaxError),
    DiagnosticDirective,
}

/// The concrete initializer and declared type of one original binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assignment {
    pub binding: Span,
    pub kind: VariableDeclarationKind,
    pub exported: bool,
    pub annotation: Span,
    pub initializer: Span,
    pub expected: Primitive,
    pub actual: Primitive,
    pub verdict: Verdict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Assignment(Assignment),
    Outside(Span),
}

pub struct LiteralAssignments<'p, 'a> {
    original: &'p AuthoredProgram<'a>,
}

impl<'p, 'a> LiteralAssignments<'p, 'a> {
    pub fn new(original: &'p AuthoredProgram<'a>) -> Result<Self, ScriptRefusal> {
        let ty = original.source_type();
        if !ty.is_typescript() || ty.is_typescript_definition() {
            return Err(ScriptRefusal::Grammar);
        }
        ScriptDeclarations::new(original).map_err(ScriptRefusal::Parse)?;
        let source = original.block().source();
        if original.parsed().program.comments.iter().any(|c| {
            source
                .get(c.span.start as usize..c.span.end as usize)
                .is_some_and(|s| {
                    s.as_bytes()
                        .windows(4)
                        .any(|part| part.eq_ignore_ascii_case(b"@ts-"))
                })
        }) {
            return Err(ScriptRefusal::DiagnosticDirective);
        }
        Ok(Self { original })
    }

    pub fn original(&self) -> &'p AuthoredProgram<'a> {
        self.original
    }

    /// Stream local outcomes. A compiler or linter can consume the name and
    /// both original ranges in the same pass.
    pub fn visit(&self, mut report: impl FnMut(Row)) {
        let block = self.original.block();
        self.inputs(|input| match input {
            Input::Variable(decl, exported) => {
                let row = observe(block, decl, exported)
                    .map_or_else(|| Row::Outside(absolute(block, decl.span)), Row::Assignment);
                report(row);
            }
            Input::Outside(span) => report(Row::Outside(absolute(block, span))),
        });
    }

    pub fn prove(&self) -> Vec<Row> {
        let mut rows = Vec::new();
        self.visit(|row| rows.push(row));
        rows
    }

    fn inputs(&self, mut visit: impl FnMut(Input<'p, 'a>)) {
        for statement in &self.original.parsed().program.body {
            let decl = match statement {
                Statement::VariableDeclaration(d) => Some(d.as_ref()),
                Statement::ExportNamedDeclaration(named) => match &named.declaration {
                    Some(Declaration::VariableDeclaration(d)) if !named.export_kind.is_type() => {
                        Some(d.as_ref())
                    }
                    _ => None,
                },
                Statement::EmptyStatement(_) => continue,
                _ => None,
            };
            if let Some(VariableDeclaration {
                declarations,
                declare: false,
                ..
            }) = decl
            {
                for variable in declarations {
                    visit(Input::Variable(
                        variable,
                        matches!(statement, Statement::ExportNamedDeclaration(_)),
                    ));
                }
            } else {
                visit(Input::Outside(statement.span()));
            }
        }
    }
}

enum Input<'p, 'a> {
    Variable(&'p VariableDeclarator<'a>, bool),
    Outside(oxc_span::Span),
}

fn absolute(block: SourceBlock<'_>, span: oxc_span::Span) -> Span {
    // The parse borrowed exactly this checked SourceBlock. Each AST range is
    // within it and the root was admitted into the shared 32-bit source space.
    Span::new(block.start() + span.start, block.start() + span.end)
}

fn observe(
    block: SourceBlock<'_>,
    decl: &VariableDeclarator<'_>,
    exported: bool,
) -> Option<Assignment> {
    if decl.definite
        || !matches!(
            decl.kind,
            VariableDeclarationKind::Const
                | VariableDeclarationKind::Let
                | VariableDeclarationKind::Var
        )
    {
        return None;
    }
    let BindingPattern::BindingIdentifier(id) = &decl.id else {
        return None;
    };
    let annotation = &decl.type_annotation.as_ref()?.type_annotation;
    let init = decl.init.as_ref()?;
    let expected = match annotation {
        TSType::TSBooleanKeyword(_) => Primitive::Boolean,
        TSType::TSNumberKeyword(_) => Primitive::Number,
        TSType::TSStringKeyword(_) => Primitive::String,
        _ => return None,
    };
    if absolute(block, annotation.span()).slice(block.root_source()) != expected.keyword() {
        return None;
    }
    let actual = match init {
        Expression::BooleanLiteral(_) => Primitive::Boolean,
        Expression::NumericLiteral(_) => Primitive::Number,
        Expression::StringLiteral(_) => Primitive::String,
        _ => return None,
    };
    Some(Assignment {
        kind: decl.kind,
        exported,
        binding: absolute(block, id.span),
        annotation: absolute(block, annotation.span()),
        initializer: absolute(block, init.span()),
        expected,
        actual,
        verdict: expected.accepts(actual),
    })
}

#[cfg(test)]
mod tests;
