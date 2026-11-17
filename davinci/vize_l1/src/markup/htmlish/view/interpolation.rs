//! Once-retained syntax at an original selected interpolation body event.

use alloc::boxed::Box;
use oxc_parser::AdmittedExpression;
use vize_l0::Span;

use super::{ChildView, NativeTemplateComponent, TemplateGrammar};
use crate::embed::syntax::{EmbedSyntax, RetainedExpression, parse_once};
use crate::embed::{Embed, Grammar, Lang, Shape, SourceError, prepare_vue_interpolation_in};
use crate::markup::{ElementClose, SurfaceChild};

mod origin;
use origin::Origin;

/// Preparation refusal leaves the original selected surface owner untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationError {
    ForeignComponent,
    NotInterpolation,
    RecoveredComponent,
    Verbatim,
    IncompleteInterpolation,
    Source(SourceError),
    UnexpectedShape,
}

/// Unexpected parser shapes remain normally owned with all observations.
#[derive(Debug)]
pub struct InterpolationFailure<'a> {
    kind: InterpolationError,
    syntax: Option<Box<EmbedSyntax<'a>>>,
}
impl InterpolationFailure<'_> {
    #[must_use]
    pub const fn kind(&self) -> InterpolationError {
        self.kind
    }
    #[must_use]
    pub fn syntax(&self) -> Option<&EmbedSyntax<'_>> {
        self.syntax.as_deref()
    }
}
impl<'a> From<InterpolationError> for InterpolationFailure<'a> {
    fn from(kind: InterpolationError) -> Self {
        Self { kind, syntax: None }
    }
}

/// Movable original body origin and the complete once-parsed expression owner.
/// No raw interpolation, AST, language, source pair or allocator can mint it.
///
/// ```compile_fail
/// use vize_l1::markup::InterpolationOperand;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<InterpolationOperand<'static>>();
/// ```
pub struct InterpolationOperand<'a> {
    origin: Origin<'a>,
    syntax: RetainedExpression<'a>,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Replace source preparation and parsing at this same original body visit.
    /// Authored HTML whitespace and text entities use the existing decoder once.
    /// The stock expression parse uses this selected owner's Module JS/TS role.
    pub fn observe_interpolation_expression(
        &self,
        child: ChildView<'_, 'a>,
    ) -> Result<InterpolationOperand<'a>, InterpolationFailure<'a>> {
        if !core::ptr::eq(child.component(), self.component()) {
            return Err(InterpolationError::ForeignComponent.into());
        }
        let SurfaceChild::Interpolation(interpolation) = child.surface() else {
            return Err(InterpolationError::NotInterpolation.into());
        };
        let carrier = self.component().carrier();
        if !carrier.errors.is_empty() || !carrier.unsupported.is_empty() {
            return Err(InterpolationError::RecoveredComponent.into());
        }
        if let Some(parent) = child.parent_element() {
            if parent.open.is_verbatim() {
                return Err(InterpolationError::Verbatim.into());
            }
            if parent.open.lt_name.is_missing()
                || parent.open.gt.is_missing()
                || parent
                    .open
                    .slash
                    .as_ref()
                    .is_some_and(|slash| slash.is_missing())
                || match &parent.close {
                    ElementClose::Missing => true,
                    ElementClose::Present(close) => {
                        close.lt_slash_name.is_missing() || close.gt.is_missing()
                    }
                    ElementClose::Implicit | ElementClose::NotExpected => false,
                }
            {
                return Err(InterpolationError::RecoveredComponent.into());
            }
        }
        let origin = Origin::from_child(self, &child, interpolation)?;
        let source = prepare_vue_interpolation_in(
            self.component().allocator(),
            origin.block.root_source(),
            origin.content_span,
        )
        .map_err(InterpolationError::Source)?;
        let lang = match self.grammar() {
            TemplateGrammar::JavaScriptModule => Lang::Js,
            TemplateGrammar::TypeScriptModule => Lang::Ts,
        };
        let syntax = parse_once(
            self.component().allocator(),
            Embed {
                grammar: Grammar {
                    shape: Shape::Expr,
                    lang,
                },
                source,
            },
        )
        .into_expression()
        .map_err(|syntax| InterpolationFailure {
            kind: InterpolationError::UnexpectedShape,
            syntax: Some(syntax),
        })?;
        Ok(InterpolationOperand { origin, syntax })
    }
}

impl<'a> InterpolationOperand<'a> {
    #[must_use]
    pub fn syntax(&self) -> &RetainedExpression<'a> {
        &self.syntax
    }
    /// The complete authored construct, including its original delimiters.
    #[must_use]
    pub const fn full_span(&self) -> Span {
        self.origin.full_span
    }
    /// Complete authored content before trimming or text-entity decoding.
    #[must_use]
    pub const fn content_span(&self) -> Span {
        self.origin.content_span
    }
    #[must_use]
    pub const fn raw_content(&self) -> &'a str {
        self.origin.raw_content
    }
    /// Reborrow at the same original body child; syntax holes cannot admit.
    pub fn admitted_for<'s>(
        &'s self,
        selected: &'s NativeTemplateComponent<'a>,
        child: ChildView<'s, 'a>,
    ) -> Option<InterpolationView<'s, 'a>> {
        if !self.origin.matches(selected, &child) {
            return None;
        }
        self.syntax.admitted_expression()?;
        Some(InterpolationView {
            owner: self,
            selected,
            child,
        })
    }
    /// Moving raw retained syntax out deliberately drops body-origin authority.
    #[must_use]
    pub fn into_syntax(self) -> RetainedExpression<'a> {
        self.syntax
    }
}

/// Short original interpolation/operand join, not a completed body or File.
pub struct InterpolationView<'s, 'a> {
    owner: &'s InterpolationOperand<'a>,
    selected: &'s NativeTemplateComponent<'a>,
    child: ChildView<'s, 'a>,
}
impl<'s, 'a> InterpolationView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn child(&self) -> &ChildView<'s, 'a> {
        &self.child
    }
    #[must_use]
    pub fn operand(&self) -> &'s InterpolationOperand<'a> {
        self.owner
    }
    #[must_use]
    pub fn expression(&self) -> Option<AdmittedExpression<'s, 'a>> {
        self.owner.syntax.admitted_expression()
    }
}

#[cfg(test)]
mod tests;
