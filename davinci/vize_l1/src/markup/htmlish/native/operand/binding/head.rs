//! Sealed short binding selection from the same original header observation.

use super::super::{
    NativeAttributeExpressionFailure, NativeAttributeOperandError, observe_expression_value,
};
use super::{NativeAttributeBindingExpression, NativeAttributeHead};
use crate::dialect::vue3::operand::static_binding_parts;
use vize_l0::Span;

/// A non-Clone short original selection, never a caller-assembled argument.
/// Consuming it observes one original value through the shared preparation tail.
/// Repeated separate public selections remain possible; a receiver takes its
/// current view once at the actual header event and parks the resulting owner.
///
/// ```compile_fail
/// use vize_l1::markup::NativeStaticBindingHead;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeStaticBindingHead<'static, 'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l0::Span;
/// use vize_l1::markup::{NativeAttributeHead, NativeStaticBindingHead};
/// fn forge<'h, 'o, 'a>(head: &'h NativeAttributeHead<'o, 'a>, argument: Span)
///     -> NativeStaticBindingHead<'h, 'o, 'a> {
///     NativeStaticBindingHead { head, argument }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{NativeAttributeHead, NativeStaticBindingHead};
/// fn escape<'o, 'a>(head: NativeAttributeHead<'o, 'a>)
///     -> NativeStaticBindingHead<'o, 'o, 'a> {
///     head.static_binding().unwrap().unwrap()
/// }
/// ```
pub struct NativeStaticBindingHead<'h, 'o, 'a> {
    head: &'h NativeAttributeHead<'o, 'a>,
    argument: Span,
}

impl<'o, 'a> NativeAttributeHead<'o, 'a> {
    /// Reuse this original dialect decomposition without another name scan.
    /// Plain/other families return None; unsupported Bind/Prop shapes refuse.
    pub fn static_binding(
        &self,
    ) -> Result<Option<NativeStaticBindingHead<'_, 'o, 'a>>, NativeAttributeOperandError> {
        let argument = static_binding_parts(self.directive(), self.name_block())?;
        Ok(argument.map(|argument| NativeStaticBindingHead {
            head: self,
            argument,
        }))
    }
}

impl<'h, 'o, 'a> NativeStaticBindingHead<'h, 'o, 'a> {
    #[must_use]
    pub fn head(&self) -> &'h NativeAttributeHead<'o, 'a> {
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
    ) -> Result<NativeAttributeBindingExpression<'a>, NativeAttributeExpressionFailure<'a>> {
        let (origin, syntax) =
            observe_expression_value(self.head.selected(), self.head.attribute().reborrow())?;
        Ok(NativeAttributeBindingExpression {
            origin,
            argument: self.argument,
            syntax,
        })
    }
}
