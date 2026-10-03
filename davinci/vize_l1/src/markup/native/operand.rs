//! Once-retained conditional syntax and its original complete header event.
//!
//! The movable owner retains no borrow of a selected/root wrapper. Its private
//! Element/Attribute addresses identify immutable nonempty arena backing, are
//! compared only, and are never dereferenced. Admission reborrows the actual
//! selected owner and current complete Attribute. This is not File/body custody.

use alloc::boxed::Box;
use oxc_parser::AdmittedExpression;
use vize_l0::{SourceBlock, Span};

use super::{NativeAttribute, NativeTemplateComponent, NativeTemplateGrammar};
pub use crate::dialect::vue3::operand::NativeConditionKind;
use crate::dialect::vue3::operand::conditional_head;
use crate::embed::syntax::{NativeSyntax, RetainedExpression, parse_once};
use crate::embed::{Embed, Grammar, Lang, Shape, SourceError, prepare_attribute_value};
use crate::markup::DirectiveNameError;
use crate::{Attribute, Element};

mod for_head;
pub use for_head::{
    NativeAttributeForHead, NativeAttributeForHeadFailure, NativeAttributeForHeadView,
};
mod origin;
use origin::Origin;

/// Preparation refusals leave the original selected surface owner untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeAttributeOperandError {
    ForeignComponent,
    RecoveredComponent,
    Verbatim,
    UnsupportedDirective,
    Directive(DirectiveNameError),
    IncompleteValue,
    Source(SourceError),
    UnexpectedShape,
}

/// Any unexpectedly shaped parser artifact remains normally owned and intact.
#[derive(Debug)]
pub struct NativeAttributeExpressionFailure<'a> {
    kind: NativeAttributeOperandError,
    syntax: Option<Box<NativeSyntax<'a>>>,
}
impl NativeAttributeExpressionFailure<'_> {
    #[must_use]
    pub const fn kind(&self) -> NativeAttributeOperandError {
        self.kind
    }
    #[must_use]
    pub fn syntax(&self) -> Option<&NativeSyntax<'_>> {
        self.syntax.as_deref()
    }
}
impl<'a> From<NativeAttributeOperandError> for NativeAttributeExpressionFailure<'a> {
    fn from(kind: NativeAttributeOperandError) -> Self {
        Self { kind, syntax: None }
    }
}

/// The original decoded value, one stock expression parse and private origin.
/// No caller AST, language, map, source range or allocator can assemble it.
///
/// ```compile_fail
/// use vize_l1::markup::NativeAttributeExpression;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeAttributeExpression<'static>>();
/// ```
pub struct NativeAttributeExpression<'a> {
    origin: Origin<'a>,
    kind: NativeConditionKind,
    syntax: RetainedExpression<'a>,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Replace the current header visit's source preparation and expression parse.
    /// This operation calls each once, deriving Module JS/TS from this selection.
    /// Syntax holes retain complete source/comments/diagnostics in the result.
    pub fn observe_attribute_expression(
        &self,
        attribute: NativeAttribute<'_, 'a>,
    ) -> Result<NativeAttributeExpression<'a>, NativeAttributeExpressionFailure<'a>> {
        Origin::check_original_header(self, &attribute)?;
        let block = self.component().block();
        let name_offset = block.offset_of(attribute.surface().name.text).ok_or(
            NativeAttributeOperandError::Source(SourceError::InvalidAuthoredSpan),
        )?;
        let kind = conditional_head(
            attribute.surface().name.text,
            name_offset,
            block.root_source(),
        )
        .map_err(NativeAttributeOperandError::Directive)?
        .ok_or(NativeAttributeOperandError::UnsupportedDirective)?;
        let origin = Origin::from_attribute(self, &attribute)?;
        let source = prepare_attribute_value(
            self.component().allocator(),
            origin.block.root_source(),
            origin.value_span,
        )
        .map_err(NativeAttributeOperandError::Source)?;
        let lang = match self.grammar() {
            NativeTemplateGrammar::JavaScriptModule => Lang::Js,
            NativeTemplateGrammar::TypeScriptModule => Lang::Ts,
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
        .map_err(|syntax| NativeAttributeExpressionFailure {
            kind: NativeAttributeOperandError::UnexpectedShape,
            syntax: Some(syntax),
        })?;
        Ok(NativeAttributeExpression {
            origin,
            kind,
            syntax,
        })
    }
}

impl<'a> NativeAttributeExpression<'a> {
    #[must_use]
    pub const fn kind(&self) -> NativeConditionKind {
        self.kind
    }
    /// Readonly retained observations, also available for local syntax holes.
    #[must_use]
    pub fn syntax(&self) -> &RetainedExpression<'a> {
        &self.syntax
    }
    /// The original full value, before entity decoding and without quote bytes.
    #[must_use]
    pub const fn raw_value(&self) -> &'a str {
        self.origin.raw_value
    }
    #[must_use]
    pub const fn value_span(&self) -> Span {
        self.origin.value_span
    }
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.origin.name_span
    }
    /// Reborrow this movable owner at its actual selected header event.
    /// Foreign/sibling/copied parses cannot confer the original event's origin.
    pub fn admitted_for<'s>(
        &'s self,
        selected: &'s NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'s, 'a>,
    ) -> Option<NativeAttributeExpressionView<'s, 'a>> {
        if !self.origin.matches(selected, &attribute) {
            return None;
        }
        self.syntax.admitted_expression()?;
        Some(NativeAttributeExpressionView {
            owner: self,
            selected,
            attribute,
        })
    }
    /// Moving out raw retained syntax deliberately discards header authority.
    #[must_use]
    pub fn into_syntax(self) -> RetainedExpression<'a> {
        self.syntax
    }
}

/// A short actual header/operand join, not a native body completion certificate.
pub struct NativeAttributeExpressionView<'s, 'a> {
    owner: &'s NativeAttributeExpression<'a>,
    selected: &'s NativeTemplateComponent<'a>,
    attribute: NativeAttribute<'s, 'a>,
}
impl<'s, 'a> NativeAttributeExpressionView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &NativeAttribute<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn operand(&self) -> &'s NativeAttributeExpression<'a> {
        self.owner
    }
    /// This proof is always borrowed from the same private original stock owner.
    #[must_use]
    pub fn expression(&self) -> Option<AdmittedExpression<'s, 'a>> {
        self.owner.syntax.admitted_expression()
    }
}

#[cfg(test)]
mod tests;
