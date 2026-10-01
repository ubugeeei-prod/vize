//! One retained JS/TS parse over checked embedded source.
//!
//! Private wrappers retain comments for expressions, handler bodies and slot
//! parameters. Consumers receive authored syntax, corrected coordinate views
//! and typed local holes. Program source is unwrapped. No product route calls
//! this provider yet; it neither selects file languages nor attaches identities.

use alloc::boxed::Box;
use oxc_ast::ast::{Expression, Program, Statement};
use oxc_diagnostics::Diagnostics;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span, String, expression_guard::expression_is_safe_to_parse};

use super::{Embed, EmbedSource, Grammar, Lang, Shape, SourceError};

mod admission;
mod coordinates;
mod handoff;
mod shapes;
mod views;
pub use admission::NATIVE_SYNTAX_UNIT_LIMIT;
use coordinates::Coordinates;
pub use handoff::RetainedExpression;
use shapes::Wrapper;
pub use shapes::{HandlerBodyView, SlotParamsView};
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
    InvalidWrappedShape,
    InvalidModuleContext,
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

    /// Unwrapped programs only; other shapes never expose wrapper Program/text.
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

    /// Authored directives/statements only, in an ordinary non-async arrow body.
    #[must_use]
    pub fn handler_body(&self) -> Option<HandlerBodyView<'_, 'a>> {
        if self.hole.is_some() || self.grammar.shape != Shape::HandlerBody {
            return None;
        }
        shapes::handler_body(self.program.as_ref()?)
    }

    /// Authored formal parameters/rest only; the synthetic arrow stays private.
    #[must_use]
    pub fn slot_params(&self) -> Option<SlotParamsView<'_, 'a>> {
        if self.hole.is_some() || self.grammar.shape != Shape::SlotParams {
            return None;
        }
        shapes::slot_params(self.program.as_ref()?)
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

    /// Move an Expr root into the arena without reparsing or cloning its AST.
    /// Non-Expr artifacts are returned intact in a normally owned box. Local
    /// Expr holes keep all source and observation metadata while exposing no
    /// recovered expression.
    pub fn into_expression(
        self,
        allocator: &'a Allocator,
    ) -> Result<RetainedExpression<'a>, Box<Self>> {
        handoff::into_expression(self, allocator)
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
    wrapper: Wrapper,
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
    if wrapper.prefix.is_empty() {
        admit(source)?;
        return Ok(source);
    }
    let mut wrapped = String::with_capacity(length);
    wrapped.push_str(wrapper.prefix);
    wrapped.push_str(source);
    wrapped.push_str(wrapper.suffix);
    admit(wrapped.as_str())?;
    Ok(allocator.alloc_str(wrapped.as_str()))
}

/// Parse Program, Expr, HandlerBody or SlotParams exactly once into the shared
/// arena. Composite shapes, Flow, inputs above the conservative token budget
/// and inputs rejected by the shared OXC safety guard remain typed holes. Full
/// admission, JSX/TSX and file language resolution remain unfinished. Actual
/// syntax diagnostics are retained, not discarded on failure.
pub fn parse_once<'a>(allocator: &'a Allocator, embed: Embed<'a>) -> NativeSyntax<'a> {
    let wrapper = Wrapper::for_shape(embed.grammar.shape);
    let mut result = NativeSyntax {
        grammar: embed.grammar,
        coordinates: Coordinates {
            source: embed.source,
            prefix: wrapper.map_or(0, |wrapper| wrapper.prefix.len() as u32),
        },
        program: None,
        diagnostics: Diagnostics::default(),
        hole: None,
    };
    let Some(wrapper) = wrapper else {
        result.hole = Some(EmbedHole::UnsupportedShape);
        return result;
    };
    let extra = wrapper.prefix.len() + wrapper.suffix.len();
    let Some(length) = parser_length(embed.source.text().len(), extra) else {
        result.hole = Some(EmbedHole::SourceTooLarge);
        return result;
    };
    let input = match admitted_input(allocator, embed.source.text(), wrapper, length) {
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
    } else if embed.grammar.shape == Shape::Expr
        && expression(&parsed.program)
            .and_then(|expression| {
                use oxc_span::GetSpan;
                result.decoded_span(expression.span()).ok()
            })
            .is_none()
    {
        Some(EmbedHole::InvalidExpressionShape)
    } else if matches!(embed.grammar.shape, Shape::HandlerBody | Shape::SlotParams)
        && !shapes::valid_wrapper(&parsed.program, embed.grammar.shape, result.coordinates)
    {
        Some(EmbedHole::InvalidWrappedShape)
    } else if matches!(embed.grammar.shape, Shape::HandlerBody | Shape::SlotParams)
        && shapes::contains_module_declaration(&parsed.program)
    {
        Some(EmbedHole::InvalidModuleContext)
    } else {
        None
    };
    result.diagnostics = parsed.diagnostics;
    result.program = Some(parsed.program);
    result
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod shape_tests;
