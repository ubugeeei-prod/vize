//! Sealed model selection from the actual original header observation.

use super::super::{
    NativeAttributeExpressionFailure, NativeAttributeOperandError, observe_expression_value,
};
use super::{NativeAttributeHead, NativeAttributeModelExpression};
use crate::dialect::vue3::operand::model_parts;

/// A short original model selection, never a caller-assembled expression.
/// Its value is prepared and parsed once when this selection is consumed.
///
/// ```compile_fail
/// use vize_l1::markup::NativeModelHead;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeModelHead<'static, 'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{NativeAttributeHead, NativeModelHead};
/// fn forge<'h, 'o, 'a>(head: &'h NativeAttributeHead<'o, 'a>)
///     -> NativeModelHead<'h, 'o, 'a> {
///     NativeModelHead { head }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{NativeAttributeHead, NativeModelHead};
/// fn escape<'o, 'a>(head: NativeAttributeHead<'o, 'a>)
///     -> NativeModelHead<'o, 'o, 'a> {
///     head.model().unwrap().unwrap()
/// }
/// ```
pub struct NativeModelHead<'h, 'o, 'a> {
    head: &'h NativeAttributeHead<'o, 'a>,
}

impl<'o, 'a> NativeAttributeHead<'o, 'a> {
    /// Select only a bare full `v-model` from this original decomposition.
    /// Other families return None; arguments and modifiers remain unsupported.
    /// Conditional and static-binding expression routes keep their own contracts.
    pub fn model(
        &self,
    ) -> Result<Option<NativeModelHead<'_, 'o, 'a>>, NativeAttributeOperandError> {
        Ok(model_parts(self.directive(), self.name_block())?
            .then_some(NativeModelHead { head: self }))
    }
}

impl<'h, 'o, 'a> NativeModelHead<'h, 'o, 'a> {
    #[must_use]
    pub fn head(&self) -> &'h NativeAttributeHead<'o, 'a> {
        self.head
    }

    /// Retain the actual complete value and stock syntax, including parse holes.
    /// File scope, assignability, read/write roles and targets are later contracts.
    pub fn observe_expression(
        self,
    ) -> Result<NativeAttributeModelExpression<'a>, NativeAttributeExpressionFailure<'a>> {
        let (origin, syntax) =
            observe_expression_value(self.head.selected(), self.head.attribute().reborrow())?;
        Ok(NativeAttributeModelExpression { origin, syntax })
    }
}
