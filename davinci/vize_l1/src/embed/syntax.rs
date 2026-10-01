//! One retained JS/TS parse over checked embedded source.
//!
//! Expr uses one private parenthesized Program wrapper to retain OXC comments.
//! Consumers receive the original expression, corrected coordinate views and
//! typed local holes. Program source is unwrapped. No product route calls this
//! provider yet; it neither selects file languages nor attaches node identities.

use oxc_ast::ast::{Expression, Program, Statement};
use oxc_diagnostics::Diagnostics;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span, String, expression_guard::expression_is_safe_to_parse};

use super::{Embed, EmbedSource, Grammar, Lang, Shape, SourceError};

mod admission;
mod coordinates;
mod views;
pub use admission::NATIVE_SYNTAX_UNIT_LIMIT;
use coordinates::Coordinates;
pub use views::{CommentView, DiagnosticLabel, DiagnosticView};

/// A malformed or unadmitted embed stays local; its source is always retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedHole {
    SafetyAdmission,
    TokenBudget,
    Syntax,
    UnsupportedShape,
    UnsupportedFlow,
    SourceTooLarge,
    InvalidExpressionShape,
}

/// Actual OXC syntax, retained comments/diagnostics and one optional typed hole.
///
/// Recovered trees remain private when a hole is present. Comment and diagnostic
/// views remain available, including their corrected decoded/authored spans.
/// AST spans must pass through `decoded_span` / `authored_span`; these are syntax
/// artifacts rather than semantic validation or resolved identifiers.
pub struct NativeSyntax<'a> {
    grammar: Grammar,
    coordinates: Coordinates<'a>,
    program: Option<Program<'a>>,
    diagnostics: Diagnostics,
    hole: Option<EmbedHole>,
}

impl core::fmt::Debug for NativeSyntax<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeSyntax")
            .field("grammar", &self.grammar)
            .field("source", &self.source())
            .field("hole", &self.hole)
            .field("diagnostic_count", &self.diagnostics.len())
            .finish_non_exhaustive()
    }
}

impl<'a> NativeSyntax<'a> {
    #[must_use]
    pub const fn grammar(&self) -> Grammar {
        self.grammar
    }

    #[must_use]
    pub const fn source(&self) -> EmbedSource<'a> {
        self.coordinates.source
    }

    #[must_use]
    pub const fn hole(&self) -> Option<EmbedHole> {
        self.hole
    }

    /// Unwrapped programs only; Expr never exposes the wrapper Program/text.
    #[must_use]
    pub fn program(&self) -> Option<&Program<'a>> {
        (self.hole.is_none() && self.grammar.shape == Shape::Program)
            .then_some(self.program.as_ref())
            .flatten()
    }

    /// Remove only the generated outer parentheses, preserving authored ones.
    #[must_use]
    pub fn expression(&self) -> Option<&Expression<'a>> {
        if self.hole.is_some() || self.grammar.shape != Shape::Expr {
            return None;
        }
        expression(self.program.as_ref()?)
    }

    pub fn comments(&self) -> impl Iterator<Item = CommentView<'_, 'a>> {
        self.program
            .iter()
            .flat_map(|program| program.comments.iter())
            .map(|comment| CommentView {
                comment,
                coordinates: self.coordinates,
            })
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = DiagnosticView<'_, 'a>> {
        self.diagnostics.iter().map(|diagnostic| DiagnosticView {
            diagnostic,
            coordinates: self.coordinates,
        })
    }

    /// Exact wrapper correction. Wrapper-only bytes never select source bytes.
    pub fn decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.coordinates.decoded_span(span)
    }

    /// Exact projection suitable for edits; partial entities remain errors.
    pub fn authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.source().authored_span(self.decoded_span(span)?)
    }
}

fn expression<'p, 'a>(program: &'p Program<'a>) -> Option<&'p Expression<'a>> {
    let [Statement::ExpressionStatement(statement)] = program.body.as_slice() else {
        return None;
    };
    let Expression::ParenthesizedExpression(wrapper) = &statement.expression else {
        return None;
    };
    Some(&wrapper.expression)
}

fn parser_length(length: usize, extra: usize) -> Option<usize> {
    let length = length.checked_add(extra)?;
    (length <= u32::MAX as usize && length <= isize::MAX as usize).then_some(length)
}

fn admitted_input<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    prefix: u32,
    length: usize,
) -> Result<&'a str, EmbedHole> {
    let admit = |text: &str| {
        if !admission::allows_small_input(text) {
            return Err(EmbedHole::TokenBudget);
        }
        if !expression_is_safe_to_parse(text) {
            return Err(EmbedHole::SafetyAdmission);
        }
        Ok(())
    };
    if prefix == 0 {
        admit(source)?;
        return Ok(source);
    }
    let mut wrapped = String::with_capacity(length);
    wrapped.push_str("(\n");
    wrapped.push_str(source);
    wrapped.push_str("\n)");
    admit(wrapped.as_str())?;
    Ok(allocator.alloc_str(wrapped.as_str()))
}

/// Parse Expr or Program exactly once into the shared arena. Other shapes,
/// Flow, inputs above the conservative token budget and inputs rejected by the
/// shared OXC safety guard remain typed holes. Full Program admission is pending.
/// JSX/TSX admission, composite shapes and per-file language resolution are
/// still unfinished. Syntax diagnostics are retained, not discarded on failure.
pub fn parse_once<'a>(allocator: &'a Allocator, embed: Embed<'a>) -> NativeSyntax<'a> {
    let prefix = if embed.grammar.shape == Shape::Expr {
        2
    } else {
        0
    };
    let mut result = NativeSyntax {
        grammar: embed.grammar,
        coordinates: Coordinates {
            source: embed.source,
            prefix,
        },
        program: None,
        diagnostics: Diagnostics::default(),
        hole: None,
    };
    if !matches!(embed.grammar.shape, Shape::Expr | Shape::Program) {
        result.hole = Some(EmbedHole::UnsupportedShape);
        return result;
    }
    let extra = if prefix == 0 { 0 } else { 4 };
    let Some(length) = parser_length(embed.source.text().len(), extra) else {
        result.hole = Some(EmbedHole::SourceTooLarge);
        return result;
    };
    let input = match admitted_input(allocator, embed.source.text(), prefix, length) {
        Ok(input) => input,
        Err(hole) => {
            result.hole = Some(hole);
            return result;
        }
    };
    let source_type = match embed.grammar.lang {
        Lang::Js => SourceType::mjs(),
        Lang::Ts => SourceType::ts().with_module(true),
    };
    let parsed = Parser::new(allocator.as_oxc(), input, source_type).parse();
    result.hole = if parsed.is_flow_language {
        Some(EmbedHole::UnsupportedFlow)
    } else if parsed.panicked || parsed.diagnostics.has_errors() {
        Some(EmbedHole::Syntax)
    } else if prefix != 0
        && expression(&parsed.program)
            .and_then(|expression| {
                use oxc_span::GetSpan;
                result.decoded_span(expression.span()).ok()
            })
            .is_none()
    {
        Some(EmbedHole::InvalidExpressionShape)
    } else {
        None
    };
    result.diagnostics = parsed.diagnostics;
    result.program = Some(parsed.program);
    result
}

#[cfg(test)]
mod tests;
