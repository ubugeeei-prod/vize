//! One retained JS/TS parse over checked embedded source.
//!
//! Private wrappers retain comments for expressions, handler bodies and slot
//! parameters. Consumers receive authored syntax, corrected coordinate views
//! and typed local holes. Program source is unwrapped. No product route calls
//! this provider yet; it neither selects file languages nor attaches identities.

use alloc::boxed::Box;
use oxc_ast::ast::{Expression, Program};
use oxc_diagnostics::Diagnostics;
use oxc_parser::{AdmittedProgram, EmbeddingObservation, ProgramObservation};
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

use super::{Embed, EmbedSource, Grammar, Lang, Shape, SourceError};

mod admission;
mod borrowed;
mod coordinates;
mod for_head;
mod handler;
mod handoff;
mod params;
mod program;
mod shapes;
mod views;
mod wrapped;
pub use admission::NATIVE_SYNTAX_UNIT_LIMIT;
pub use borrowed::NativeExpressionView;
use coordinates::Coordinates;
pub use for_head::{
    AdmittedDenseForHead, DenseForHeadView, ForHeadHole, ForHeadPart, ForKeyword, NativeForHead,
    NativeForInput, NativeForInputError, NativeForRefusal, RejectedNativeForInput,
    parse_vue_for_head_once,
};
pub use handler::RetainedHandlerBody;
pub use handoff::RetainedExpression;
pub use params::RetainedSlotParams;
pub use program::{ProgramGoal, ProgramOptions, parse_program_once};
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
    InvalidParameterContext,
}

/// Actual OXC syntax, retained comments/diagnostics and one optional typed hole.
///
/// Recovered trees remain private when a hole is present. Comment and diagnostic
/// views remain available, including their corrected decoded/authored spans.
/// AST spans must pass through `decoded_span` / `authored_span`; these are syntax
/// artifacts rather than semantic validation or resolved identifiers.
pub struct NativeSyntax<'a> {
    grammar: Grammar,
    source_type: SourceType,
    coordinates: Coordinates<'a>,
    observation: Option<ProgramObservation<'a>>,
    embedding: Option<EmbeddingObservation<'a>>,
    diagnostics: Diagnostics,
    hole: Option<EmbedHole>,
}

impl core::fmt::Debug for NativeSyntax<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeSyntax")
            .field("grammar", &self.grammar)
            .field("source_type", &self.source_type)
            .field("source", &self.source())
            .field("hole", &self.hole)
            .field("diagnostic_count", &self.diagnostics().count())
            .finish_non_exhaustive()
    }
}

impl<'a> NativeSyntax<'a> {
    #[must_use]
    pub const fn grammar(&self) -> Grammar {
        self.grammar
    }

    /// The explicit compiler parse profile, retained even on a local hole.
    /// This is never an OXC formatter-profile AST or an inferred module goal.
    #[must_use]
    pub const fn source_type(&self) -> SourceType {
        self.source_type
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
        self.admitted_program().map(|admitted| admitted.program())
    }

    /// Original parser-owned admission; callers cannot supply replacement status.
    #[must_use]
    pub fn admitted_program(&self) -> Option<AdmittedProgram<'_, 'a>> {
        if self.hole.is_some() || self.grammar.shape != Shape::Program {
            return None;
        }
        self.observation.as_ref()?.admitted()
    }

    /// Remove only the generated outer parentheses, preserving authored ones.
    #[must_use]
    pub fn expression(&self) -> Option<&Expression<'a>> {
        if self.hole.is_some() || self.grammar.shape != Shape::Expr {
            return None;
        }
        self.embedding.as_ref()?.expression()
    }

    /// Authored directives/statements only, in an ordinary non-async arrow body.
    #[must_use]
    pub fn handler_body(&self) -> Option<HandlerBodyView<'_, 'a>> {
        if self.hole.is_some() || self.grammar.shape != Shape::HandlerBody {
            return None;
        }
        self.embedding
            .as_ref()?
            .handler_body()
            .map(HandlerBodyView::from_body)
    }

    /// Authored formal parameters/rest only; the synthetic arrow stays private.
    #[must_use]
    pub fn slot_params(&self) -> Option<SlotParamsView<'_, 'a>> {
        if self.hole.is_some() || self.grammar.shape != Shape::SlotParams {
            return None;
        }
        self.embedding
            .as_ref()?
            .parameters()
            .map(SlotParamsView::from_parameters)
    }

    pub fn comments(&self) -> impl Iterator<Item = CommentView<'_, 'a>> {
        self.embedding
            .iter()
            .flat_map(|observation| observation.comments().iter())
            .chain(
                self.observation
                    .iter()
                    .flat_map(|observation| observation.comments().iter()),
            )
            .map(|comment| CommentView {
                comment,
                coordinates: self.coordinates,
            })
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = DiagnosticView<'_, 'a>> {
        self.diagnostics
            .iter()
            .chain(
                self.observation
                    .iter()
                    .flat_map(|observation| observation.diagnostics().iter()),
            )
            .chain(
                self.embedding
                    .iter()
                    .flat_map(|observation| observation.diagnostics().iter()),
            )
            .map(|diagnostic| DiagnosticView {
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
    pub fn into_expression(self) -> Result<RetainedExpression<'a>, Box<Self>> {
        handoff::into_expression(self)
    }

    /// Consume the original non-async HandlerBody parser owner without cloning
    /// or reparsing its AST. Every local hole keeps its complete observations;
    /// another grammar shape returns the original owner intact.
    pub fn into_handler_body(self) -> Result<RetainedHandlerBody<'a>, Box<Self>> {
        handler::into_handler_body(self)
    }

    /// Consume only the original parser-owned arena and complete observations.
    /// A non-parameter shape returns its original owned artifact intact.
    pub fn into_slot_params(self) -> Result<RetainedSlotParams<'a>, Box<Self>> {
        params::into_slot_params(self)
    }
}

fn parser_length(length: usize, extra: usize) -> Option<usize> {
    let length = length.checked_add(extra)?;
    (length <= u32::MAX as usize && length <= isize::MAX as usize).then_some(length)
}

/// Parse Program, Expr, HandlerBody or SlotParams exactly once into the shared
/// arena. Program uses the ordinary parser with explicit JS/TS Module options.
/// Wrapped shapes keep their conservative admission and OXC safety guard.
/// Composite shapes, Flow and file language resolution remain unfinished. Actual
/// syntax diagnostics are retained, not discarded on failure.
pub fn parse_once<'a>(allocator: &'a Allocator, embed: Embed<'a>) -> NativeSyntax<'a> {
    if embed.grammar.shape == Shape::Program {
        return parse_program_once(
            allocator,
            embed.source,
            ProgramOptions::module(embed.grammar.lang),
        );
    }
    wrapped::parse(allocator, embed)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod shape_tests;

#[cfg(test)]
mod embedding_tests;

#[cfg(test)]
mod parameter_context_tests;
