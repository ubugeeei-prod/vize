//! Sealed short binding selection from the same original header observation.

use super::super::{AttributeExpressionFailure, AttributeOperandError, observe_expression_value};
use super::{AttributeBindingExpression, AttributeHead};
use crate::dialect::vue3::operand::static_binding_parts;
use vize_l0::Span;

/// A non-Clone short original selection, never a caller-assembled argument.
/// Consuming it observes one original value through the shared preparation tail.
/// Repeated separate public selections remain possible; a receiver takes its
/// current view once at the actual header event and parks the resulting owner.
///
/// ```compile_fail
/// use vize_l1::markup::StaticBindingHead;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<StaticBindingHead<'static, 'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l0::Span;
/// use vize_l1::markup::{AttributeHead, StaticBindingHead};
/// fn forge<'h, 'o, 'a>(head: &'h AttributeHead<'o, 'a>, argument: Span)
///     -> StaticBindingHead<'h, 'o, 'a> {
///     StaticBindingHead { head, argument }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{AttributeHead, StaticBindingHead};
/// fn escape<'o, 'a>(head: AttributeHead<'o, 'a>)
///     -> StaticBindingHead<'o, 'o, 'a> {
///     head.static_binding().unwrap().unwrap()
/// }
/// ```
pub struct StaticBindingHead<'h, 'o, 'a> {
    head: &'h AttributeHead<'o, 'a>,
    argument: Span,
}

impl<'o, 'a> AttributeHead<'o, 'a> {
    /// Reuse this original dialect decomposition without another name scan.
    /// Plain/other families return None; unsupported Bind/Prop shapes refuse.
    pub fn static_binding(
        &self,
    ) -> Result<Option<StaticBindingHead<'_, 'o, 'a>>, AttributeOperandError> {
        let argument = static_binding_parts(self.directive(), self.name_block())?;
        Ok(argument.map(|argument| StaticBindingHead {
            head: self,
            argument,
        }))
    }
}

impl<'h, 'o, 'a> StaticBindingHead<'h, 'o, 'a> {
    #[must_use]
    pub fn head(&self) -> &'h AttributeHead<'o, 'a> {
        self.head
    }
    /// The exact file-absolute argument already selected by the original head.
    #[must_use]
    pub const fn argument_span(&self) -> Span {
        self.argument
    }
    /// Preserve the real current value, selected grammar and stock parse once.
    /// An absent/incomplete value leaves an actual failure, never a surrogate.
    pub fn observe_expression(
        self,
    ) -> Result<AttributeBindingExpression<'a>, AttributeExpressionFailure<'a>> {
        let (origin, syntax) =
            observe_expression_value(self.head.selected(), self.head.attribute().reborrow())?;
        Ok(AttributeBindingExpression {
            origin,
            argument: self.argument,
            syntax,
        })
    }
}
