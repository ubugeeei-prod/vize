//! Move an authenticated selected header into its original arena, once.

use oxc_ast::ast::{Expression, FormalParameters, FunctionBody, Statement};
use oxc_diagnostics::Diagnostics;
use oxc_span::{GetSpan, SourceType, Span};

use super::{EmbeddingGoal, EmbeddingHole, EmbeddingObservation};
use crate::ParseOptions;

macro_rules! projection {
    ($owner:ident, $proof:ident, $root:ty, $accessor:ident) => {
        /// An ordinarily owned complete parser observation and selected arena root.
        pub struct $owner<'a> {
            observation: EmbeddingObservation<'a>,
            root: Option<&'a $root>,
        }
        impl core::fmt::Debug for $owner<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple(stringify!($owner)).field(&self.observation).finish()
            }
        }
        /// A short private-origin borrow; raw ASTs and numeric windows cannot mint it.
        pub struct $proof<'p, 'a> {
            owner: &'p $owner<'a>,
            root: &'a $root,
        }
        impl<'a> $owner<'a> {
            #[must_use]
            pub fn admitted(&self) -> Option<$proof<'_, 'a>> {
                self.root
                    .filter(|_| self.observation.hole.is_none())
                    .map(|root| $proof { owner: self, root })
            }
            #[must_use]
            pub const fn hole(&self) -> Option<EmbeddingHole> {
                self.observation.hole
            }
            #[must_use]
            pub fn diagnostics(&self) -> &Diagnostics {
                self.observation.diagnostics()
            }
            #[must_use]
            pub fn comments(&self) -> &[oxc_ast::ast::Comment] {
                self.observation.comments()
            }
            #[must_use]
            pub const fn content(&self) -> &'a str {
                self.observation.content
            }
            #[must_use]
            pub const fn source_type(&self) -> SourceType {
                self.observation.source_type
            }
            #[must_use]
            pub const fn parser_content_span(&self) -> Span {
                self.observation.content_span
            }
        }
        impl<'p, 'a> $proof<'p, 'a> {
            #[must_use]
            pub const fn $accessor(&self) -> &'a $root {
                self.root
            }
            #[must_use]
            pub const fn content(&self) -> &'a str {
                self.owner.observation.content
            }
            #[must_use]
            pub const fn parser_content_span(&self) -> Span {
                self.owner.observation.content_span
            }
            #[must_use]
            pub fn parser_container_span(&self) -> Span {
                self.root.span()
            }
            #[must_use]
            pub const fn source_type(&self) -> SourceType {
                self.owner.observation.source_type
            }
            #[must_use]
            pub const fn options(&self) -> ParseOptions {
                self.owner.observation.options
            }
        }
    };
}

projection!(ExpressionObservation, AdmittedExpression, Expression<'a>, expression);
projection!(HandlerBodyObservation, AdmittedHandlerBody, FunctionBody<'a>, body);
projection!(ParametersObservation, AdmittedParameters, FormalParameters<'a>, parameters);

impl AdmittedExpression<'_, '_> {
    /// Whether the original stock lexer decoded a legacy numeric literal or
    /// string escape forbidden in a strict module. This borrows the complete
    /// original parser observation; it adds no parse or source/AST scan.
    /// Syntax admission is unchanged, and false does not certify semantics.
    #[must_use]
    pub const fn has_legacy_literals(&self) -> bool {
        self.owner.observation.parsed.has_legacy_literals
    }
}

impl<'a> EmbeddingObservation<'a> {
    /// Consumption uses only the original stored arena, not a caller arena.
    ///
    /// ```compile_fail,E0061
    /// use oxc_allocator::Allocator;
    /// use oxc_parser::EmbeddingObservation;
    /// fn substitute<'a>(owner: EmbeddingObservation<'a>, other: &'a Allocator) {
    ///     owner.into_expression(other);
    /// }
    /// ```
    ///
    /// ```compile_fail,E0451
    /// use oxc_ast::ast::Expression;
    /// use oxc_parser::{AdmittedExpression, ExpressionObservation};
    /// fn forge<'p, 'a>(owner: &'p ExpressionObservation<'a>, root: &'a Expression<'a>) -> AdmittedExpression<'p, 'a> {
    ///     AdmittedExpression { owner, root }
    /// }
    /// ```
    pub fn into_expression(mut self) -> Result<ExpressionObservation<'a>, Box<Self>> {
        if self.goal != EmbeddingGoal::Expr {
            return Err(Box::new(self));
        }
        let root = if self.hole.is_none()
            && let Some(Statement::ExpressionStatement(statement)) = self.parsed.program.body.pop()
            && let Expression::ParenthesizedExpression(wrapper) = statement.unbox().expression
        {
            Some(&*self.allocator.alloc(wrapper.unbox().expression))
        } else {
            None
        };
        if self.hole.is_none() && root.is_none() {
            self.hole = Some(EmbeddingHole::InvalidExpressionShape);
        }
        Ok(ExpressionObservation { observation: self, root })
    }

    /// Raw FunctionBody roots cannot mint original-parser authority.
    ///
    /// ```compile_fail,E0451
    /// use oxc_ast::ast::FunctionBody;
    /// use oxc_parser::{AdmittedHandlerBody, HandlerBodyObservation};
    /// fn forge<'p, 'a>(owner: &'p HandlerBodyObservation<'a>, root: &'a FunctionBody<'a>) -> AdmittedHandlerBody<'p, 'a> {
    ///     AdmittedHandlerBody { owner, root }
    /// }
    /// ```
    ///
    /// ```compile_fail,E0308
    /// use oxc_ast::ast::FunctionBody;
    /// use oxc_parser::AdmittedHandlerBody;
    /// fn mutate<'p, 'a>(token: AdmittedHandlerBody<'p, 'a>) -> &'a mut FunctionBody<'a> {
    ///     token.body()
    /// }
    /// ```
    pub fn into_handler_body(mut self) -> Result<HandlerBodyObservation<'a>, Box<Self>> {
        if self.goal != EmbeddingGoal::HandlerBody {
            return Err(Box::new(self));
        }
        let root = if self.hole.is_none()
            && let Some(Statement::ExpressionStatement(statement)) = self.parsed.program.body.pop()
            && let Expression::ArrowFunctionExpression(arrow) = statement.unbox().expression
        {
            Some(&*self.allocator.alloc(arrow.unbox().body.unbox()))
        } else {
            None
        };
        if self.hole.is_none() && root.is_none() {
            self.hole = Some(EmbeddingHole::InvalidWrappedShape);
        }
        Ok(HandlerBodyObservation { observation: self, root })
    }

    /// Raw formal parameters cannot replace the original selected owner.
    ///
    /// ```compile_fail,E0451
    /// use oxc_ast::ast::FormalParameters;
    /// use oxc_parser::{AdmittedParameters, ParametersObservation};
    /// fn forge<'p, 'a>(owner: &'p ParametersObservation<'a>, root: &'a FormalParameters<'a>) -> AdmittedParameters<'p, 'a> {
    ///     AdmittedParameters { owner, root }
    /// }
    /// ```
    pub fn into_parameters(mut self) -> Result<ParametersObservation<'a>, Box<Self>> {
        if self.goal != EmbeddingGoal::Parameters {
            return Err(Box::new(self));
        }
        let root = if self.hole.is_none()
            && let Some(Statement::ExpressionStatement(statement)) = self.parsed.program.body.pop()
            && let Expression::ArrowFunctionExpression(arrow) = statement.unbox().expression
        {
            Some(&*self.allocator.alloc(arrow.unbox().params.unbox()))
        } else {
            None
        };
        if self.hole.is_none() && root.is_none() {
            self.hole = Some(EmbeddingHole::InvalidWrappedShape);
        }
        Ok(ParametersObservation { observation: self, root })
    }
}
