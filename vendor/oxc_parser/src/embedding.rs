//! Authenticated fixed language wrappers over the original source and arena.

use compact_str::CompactString;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Comment, Expression, FormalParameters, FunctionBody};
use oxc_diagnostics::Diagnostics;
use oxc_span::{SourceType, Span};

use crate::{ParseOptions, Parser, ParserReturn};

mod projection;
mod shapes;
pub use projection::{
    AdmittedExpression, AdmittedHandlerBody, AdmittedParameters, ExpressionObservation,
    HandlerBodyObservation, ParametersObservation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingGoal {
    Expr,
    HandlerBody,
    Parameters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingInputError {
    SourceTooLarge,
    UnsupportedProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingHole {
    Syntax,
    UnsupportedFlow,
    InvalidExpressionShape,
    InvalidWrappedShape,
    InvalidModuleContext,
    InvalidParameterContext,
}

/// A private fixed wrapper and its original allocation/source identity.
/// No caller AST, wrapper, options or lexer configuration can be substituted.
///
/// ```compile_fail,E0451
/// use oxc_parser::EmbeddingInput;
/// fn forge<'a>(input: EmbeddingInput<'a>) -> EmbeddingInput<'a> {
///     EmbeddingInput { content: "replacement", ..input }
/// }
/// ```
pub struct EmbeddingInput<'a> {
    allocator: &'a Allocator,
    content: &'a str,
    source_type: SourceType,
    goal: EmbeddingGoal,
    wrapper: CompactString,
    content_span: Span,
}

impl<'a> EmbeddingInput<'a> {
    /// Prepare ordinary temporary bytes, without promoting them into the arena.
    /// Current embedding contexts admit explicit modules, not inferred goals.
    pub fn prepare_in(
        allocator: &'a Allocator,
        content: &'a str,
        source_type: SourceType,
        goal: EmbeddingGoal,
    ) -> Result<Self, EmbeddingInputError> {
        if !source_type.is_module() || source_type.is_typescript_definition() {
            return Err(EmbeddingInputError::UnsupportedProfile);
        }
        let (prefix, suffix) = shapes::wrapper(goal);
        let length = content
            .len()
            .checked_add(prefix.len() + suffix.len())
            .filter(|length| *length <= crate::MAX_LEN)
            .ok_or(EmbeddingInputError::SourceTooLarge)?;
        let content_span = Span::new(prefix.len() as u32, (prefix.len() + content.len()) as u32);
        let mut wrapper = CompactString::with_capacity(length);
        wrapper.push_str(prefix);
        wrapper.push_str(content);
        wrapper.push_str(suffix);
        Ok(Self { allocator, content, source_type, goal, wrapper, content_span })
    }

    /// The exact temporary parser input, available to the caller's resource guard.
    #[must_use]
    pub fn parser_source(&self) -> &str {
        self.wrapper.as_str()
    }

    #[must_use]
    pub const fn parser_content_span(&self) -> Span {
        self.content_span
    }

    /// Promote into the original arena and call the stock parser exactly once.
    /// This has no custom configuration, options or replacement arena argument.
    ///
    /// ```compile_fail,E0061
    /// use oxc_allocator::Allocator;
    /// use oxc_parser::EmbeddingInput;
    /// fn substitute<'a>(input: EmbeddingInput<'a>, other: &'a Allocator) {
    ///     input.observe(other);
    /// }
    /// ```
    #[must_use]
    pub fn observe(self) -> EmbeddingObservation<'a> {
        let source = self.allocator.alloc_str(self.wrapper.as_str());
        let options = ParseOptions::default();
        let parsed =
            Parser::new(self.allocator, source, self.source_type).with_options(options).parse();
        let hole = if parsed.is_flow_language {
            Some(EmbeddingHole::UnsupportedFlow)
        } else if parsed.panicked || parsed.diagnostics.has_errors() {
            Some(EmbeddingHole::Syntax)
        } else {
            shapes::shape_hole(&parsed.program, self.goal, self.content_span).or_else(|| {
                (self.goal != EmbeddingGoal::Expr)
                    .then(|| shapes::context_hole(&parsed.program))
                    .flatten()
            })
        };
        EmbeddingObservation {
            allocator: self.allocator,
            content: self.content,
            source_type: self.source_type,
            goal: self.goal,
            content_span: self.content_span,
            parsed,
            options,
            hole,
        }
    }
}

/// The complete ordinary result, including all diagnostics and parser fields.
/// Shared selected syntax cannot mint a replacement admitted projection.
pub struct EmbeddingObservation<'a> {
    allocator: &'a Allocator,
    content: &'a str,
    source_type: SourceType,
    goal: EmbeddingGoal,
    content_span: Span,
    parsed: ParserReturn<'a>,
    options: ParseOptions,
    hole: Option<EmbeddingHole>,
}

impl core::fmt::Debug for EmbeddingObservation<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EmbeddingObservation")
            .field("goal", &self.goal)
            .field("source_type", &self.source_type)
            .field("hole", &self.hole)
            .finish_non_exhaustive()
    }
}

impl<'a> EmbeddingObservation<'a> {
    #[must_use]
    pub const fn content(&self) -> &'a str {
        self.content
    }
    #[must_use]
    pub const fn source_type(&self) -> SourceType {
        self.source_type
    }
    #[must_use]
    pub const fn options(&self) -> ParseOptions {
        self.options
    }
    #[must_use]
    pub const fn hole(&self) -> Option<EmbeddingHole> {
        self.hole
    }
    #[must_use]
    pub const fn parser_content_span(&self) -> Span {
        self.content_span
    }
    #[must_use]
    pub fn comments(&self) -> &[Comment] {
        self.parsed.program.comments.as_slice()
    }
    #[must_use]
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.parsed.diagnostics
    }
    #[must_use]
    pub fn panicked(&self) -> bool {
        self.parsed.panicked
    }
    #[must_use]
    pub fn is_flow_language(&self) -> bool {
        self.parsed.is_flow_language
    }

    #[must_use]
    pub fn expression(&self) -> Option<&Expression<'a>> {
        (self.hole.is_none() && self.goal == EmbeddingGoal::Expr)
            .then(|| shapes::expression(&self.parsed.program))
            .flatten()
    }
    #[must_use]
    pub fn handler_body(&self) -> Option<&FunctionBody<'a>> {
        (self.hole.is_none() && self.goal == EmbeddingGoal::HandlerBody)
            .then(|| shapes::arrow(&self.parsed.program).map(|arrow| &*arrow.body))
            .flatten()
    }
    #[must_use]
    pub fn parameters(&self) -> Option<&FormalParameters<'a>> {
        (self.hole.is_none() && self.goal == EmbeddingGoal::Parameters)
            .then(|| shapes::arrow(&self.parsed.program).map(|arrow| &*arrow.params))
            .flatten()
    }
}
