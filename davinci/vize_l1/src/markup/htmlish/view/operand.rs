//! Once-retained conditional syntax and its original complete header event.
//!
//! The movable owner retains no borrow of a selected/root wrapper. Its private
//! Element/Attribute addresses identify immutable nonempty arena backing, are
//! compared only, and are never dereferenced. Admission reborrows the actual
//! selected owner and current complete Attribute. This is not File/body custody.

use alloc::boxed::Box;
use oxc_parser::AdmittedExpression;
use vize_l0::{SourceBlock, Span};

use super::{AttributeView, NativeTemplateComponent, TemplateGrammar};
pub use crate::framework::vue::version::v3::operand::ConditionKind;
use crate::framework::vue::version::v3::operand::conditional_head;
use crate::embed::syntax::{EmbedSyntax, RetainedExpression, parse_once};
use crate::embed::{Embed, Grammar, Lang, Shape, SourceError, prepare_attribute_value};
use crate::markup::DirectiveNameError;
use crate::markup::{Attribute, Element};

mod event;
pub use event::{AttributeHandler, AttributeHandlerFailure, AttributeHandlerView};
mod for_head;
pub use for_head::{AttributeForHead, AttributeForHeadFailure, AttributeForHeadView};
mod head;
pub use head::AttributeHead;
mod else_head;
pub use else_head::{AttributeElse, AttributeElseView};
mod binding;
pub use binding::{AttributeBindingExpression, AttributeBindingExpressionView, StaticBindingHead};
mod model;
pub use model::{AttributeModelExpression, AttributeModelExpressionView, ModelHead};
mod origin;
use origin::Origin;
mod value;
pub use value::{AttributeValueFailure, AttributeValueOperand, AttributeValueView};

/// Preparation refusals leave the original selected surface owner untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOperandError {
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
pub struct AttributeExpressionFailure<'a> {
    kind: AttributeOperandError,
    syntax: Option<Box<EmbedSyntax<'a>>>,
}
impl AttributeExpressionFailure<'_> {
    #[must_use]
    pub const fn kind(&self) -> AttributeOperandError {
        self.kind
    }
    #[must_use]
    pub fn syntax(&self) -> Option<&EmbedSyntax<'_>> {
        self.syntax.as_deref()
    }
}
impl<'a> From<AttributeOperandError> for AttributeExpressionFailure<'a> {
    fn from(kind: AttributeOperandError) -> Self {
        Self { kind, syntax: None }
    }
}

/// The original decoded value, one stock expression parse and private origin.
/// No caller AST, language, map, source range or allocator can assemble it.
///
/// ```compile_fail
/// use vize_l1::markup::AttributeExpression;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<AttributeExpression<'static>>();
/// ```
pub struct AttributeExpression<'a> {
    origin: Origin<'a>,
    kind: ConditionKind,
    syntax: RetainedExpression<'a>,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Replace the current header visit's source preparation and expression parse.
    /// This operation calls each once, deriving Module JS/TS from this selection.
    /// Syntax holes retain complete source/comments/diagnostics in the result.
    pub fn observe_attribute_expression(
        &self,
        attribute: AttributeView<'_, 'a>,
    ) -> Result<AttributeExpression<'a>, AttributeExpressionFailure<'a>> {
        Origin::check_original_header(self, &attribute)?;
        let block = self.component().block();
        let name_offset =
            block
                .offset_of(attribute.surface().name.text)
                .ok_or(AttributeOperandError::Source(
                    SourceError::InvalidAuthoredSpan,
                ))?;
        let kind = conditional_head(
            attribute.surface().name.text,
            name_offset,
            block.root_source(),
        )
        .map_err(AttributeOperandError::Directive)?
        .ok_or(AttributeOperandError::UnsupportedDirective)?;
        observe_conditional(self, attribute, kind)
    }
}

// Both routes have checked the actual original header and dialect selection.
// Only this shared tail prepares and parses the complete original value.
fn observe_conditional<'a>(
    selected: &NativeTemplateComponent<'a>,
    attribute: AttributeView<'_, 'a>,
    kind: ConditionKind,
) -> Result<AttributeExpression<'a>, AttributeExpressionFailure<'a>> {
    let (origin, syntax) = observe_expression_value(selected, attribute)?;
    Ok(AttributeExpression {
        origin,
        kind,
        syntax,
    })
}

// The genuine head selection has already checked the original event. Both
// expression families use this complete value preparation and stock parse once.
fn observe_expression_value<'a>(
    selected: &NativeTemplateComponent<'a>,
    attribute: AttributeView<'_, 'a>,
) -> Result<(Origin<'a>, RetainedExpression<'a>), AttributeExpressionFailure<'a>> {
    let origin = Origin::from_attribute(selected, &attribute)?;
    let source = prepare_attribute_value(
        selected.component().allocator(),
        origin.block.root_source(),
        origin.value_span,
    )
    .map_err(AttributeOperandError::Source)?;
    let lang = match selected.grammar() {
        TemplateGrammar::JavaScriptModule => Lang::Js,
        TemplateGrammar::TypeScriptModule => Lang::Ts,
    };
    let syntax = parse_once(
        selected.component().allocator(),
        Embed {
            grammar: Grammar {
                shape: Shape::Expr,
                lang,
            },
            source,
        },
    )
    .into_expression()
    .map_err(|syntax| AttributeExpressionFailure {
        kind: AttributeOperandError::UnexpectedShape,
        syntax: Some(syntax),
    })?;
    Ok((origin, syntax))
}

impl<'a> AttributeExpression<'a> {
    #[must_use]
    pub const fn kind(&self) -> ConditionKind {
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
        attribute: AttributeView<'s, 'a>,
    ) -> Option<AttributeExpressionView<'s, 'a>> {
        if !self.origin.matches(selected, &attribute) {
            return None;
        }
        self.syntax.admitted_expression()?;
        Some(AttributeExpressionView {
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
pub struct AttributeExpressionView<'s, 'a> {
    owner: &'s AttributeExpression<'a>,
    selected: &'s NativeTemplateComponent<'a>,
    attribute: AttributeView<'s, 'a>,
}
impl<'s, 'a> AttributeExpressionView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &AttributeView<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn operand(&self) -> &'s AttributeExpression<'a> {
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
