//! Crate-private Program source admission for the owning statement walker.

use oxc_ast::ast::{Expression, ModuleExportName, Program};
use oxc_parser::AdmittedProgram;
use oxc_span::{GetSpan, SourceType};
use vize_l0::Span;

use super::{ResolutionError, ResolutionErrorKind, sink::ReferenceSink, walk};
use crate::expr::JsExpr;

#[derive(Clone, Copy)]
pub(super) enum ReferenceSource<'a> {
    Expression(JsExpr<'a>),
    Program(&'a str),
    ProgramJsx(&'a str),
    ForCollection { source: &'a str, prefix: u32 },
}

impl ReferenceSource<'_> {
    pub(super) fn span(self, span: oxc_span::Span) -> Option<Span> {
        match self {
            Self::Expression(expression) => expression.ast_span_to_source(span),
            Self::Program(source) | Self::ProgramJsx(source) => {
                source.get(span.start as usize..span.end as usize)?;
                Some(Span::new(span.start, span.end))
            }
            Self::ForCollection { source, prefix } => {
                let start = span.start.checked_sub(prefix)?;
                let end = span.end.checked_sub(prefix)?;
                source.get(start as usize..end as usize)?;
                Some(Span::new(start, end))
            }
        }
    }
    pub(super) fn length(self) -> u32 {
        match self {
            Self::Expression(expression) => expression.source.len() as u32,
            Self::Program(source)
            | Self::ProgramJsx(source)
            | Self::ForCollection { source, .. } => source.len() as u32,
        }
    }

    pub(super) const fn allows_jsx(self) -> bool {
        matches!(self, Self::ProgramJsx(_))
    }
}

mod for_head;
pub(crate) use for_head::ForReferenceSource;

/// Internal capability over the actual whole Program, never an expression window.
/// The genuine no-hole parser observation stays borrowed through the sole walk.
pub(crate) struct ProgramReferenceSource<'p, 'a> {
    admitted: AdmittedProgram<'p, 'a>,
    content: Span,
}

impl<'p, 'a> ProgramReferenceSource<'p, 'a> {
    pub(crate) fn checked(
        admitted: AdmittedProgram<'p, 'a>,
        file: &'a str,
        content: Span,
        source_type: SourceType,
    ) -> Result<Self, ResolutionError> {
        let program = admitted.program();
        let fail = || ResolutionError {
            span: Span::new(0, 0),
            kind: ResolutionErrorKind::InvalidSpan,
        };
        u32::try_from(file.len()).map_err(|_| fail())?;
        let raw = file
            .get(content.start as usize..content.end as usize)
            .ok_or_else(fail)?;
        let length = u32::try_from(raw.len()).map_err(|_| fail())?;
        if program.source_type != source_type
            || program.span != oxc_span::Span::new(0, length)
            || !core::ptr::eq(raw, program.source_text)
        {
            return Err(fail());
        }
        Ok(Self { admitted, content })
    }

    #[must_use]
    pub(crate) fn program(&self) -> &'p Program<'a> {
        self.admitted.program()
    }

    pub(crate) const fn admitted(&self) -> &AdmittedProgram<'p, 'a> {
        &self.admitted
    }

    pub(crate) fn expression(
        &self,
        expression: &Expression<'a>,
        sink: &mut impl ReferenceSink<'a>,
    ) -> Result<(), ResolutionError> {
        let source = if self.program().source_type.is_jsx() {
            ReferenceSource::ProgramJsx(self.program().source_text)
        } else {
            ReferenceSource::Program(self.program().source_text)
        };
        walk::retained(source, expression, sink)
    }

    /// Only an actual local export leaf; imported/re-exported names are policy
    /// of the statement walker and must not be passed here as local uses.
    pub(crate) fn export_local(
        &self,
        local: &ModuleExportName<'a>,
        sink: &mut impl ReferenceSink<'a>,
    ) -> Result<(), ResolutionError> {
        let (span, name) = match local {
            ModuleExportName::IdentifierName(identifier) => {
                (identifier.span, identifier.name.as_str())
            }
            ModuleExportName::IdentifierReference(identifier) => {
                (identifier.span, identifier.name.as_str())
            }
            other => {
                return Err(ResolutionError {
                    span: ReferenceSource::Program(self.program().source_text)
                        .span(other.span())
                        .unwrap_or(Span::new(0, 0)),
                    kind: ResolutionErrorKind::UnsupportedSyntax,
                });
            }
        };
        walk::export_local(
            ReferenceSource::Program(self.program().source_text),
            span,
            name,
            sink,
        )
    }

    #[must_use]
    pub(crate) fn authored_span(&self, span: Span) -> Option<Span> {
        self.program()
            .source_text
            .get(span.start as usize..span.end as usize)?;
        Some(Span::new(
            self.content.start.checked_add(span.start)?,
            self.content.start.checked_add(span.end)?,
        ))
    }
}

#[cfg(test)]
mod calls;
#[cfg(test)]
mod jsx;
#[cfg(test)]
mod tests;
