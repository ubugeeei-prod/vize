//! Sealed model selection from the actual original header observation.

use super::super::{AttributeExpressionFailure, AttributeOperandError, observe_expression_value};
use super::{AttributeHead, AttributeModelExpression};
use crate::dialect::vue3::operand::{ModelModifier, model_parts};

/// A short original model selection, never a caller-assembled expression.
/// Its value is prepared and parsed once when this selection is consumed.
///
/// ```compile_fail
/// use vize_l1::markup::ModelHead;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<ModelHead<'static, 'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{AttributeHead, ModelHead};
/// fn forge<'h, 'o, 'a>(head: &'h AttributeHead<'o, 'a>)
///     -> ModelHead<'h, 'o, 'a> {
///     ModelHead { head }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{AttributeHead, ModelHead};
/// fn escape<'o, 'a>(head: AttributeHead<'o, 'a>)
///     -> ModelHead<'o, 'o, 'a> {
///     head.model().unwrap().unwrap()
/// }
/// ```
pub struct ModelHead<'h, 'o, 'a> {
    head: &'h AttributeHead<'o, 'a>,
    modifier: Option<ModelModifier>,
}

impl<'o, 'a> AttributeHead<'o, 'a> {
    /// Select full `v-model` with at most one original `.trim`, `.lazy` or `.number`.
    /// Other families return None; arguments and other modifiers are unsupported.
    /// Conditional and static-binding expression routes keep their own contracts.
    pub fn model(&self) -> Result<Option<ModelHead<'_, 'o, 'a>>, AttributeOperandError> {
        Ok(
            model_parts(self.directive(), self.name_block())?.map(|parts| ModelHead {
                head: self,
                modifier: parts.modifier,
            }),
        )
    }
}

impl<'h, 'o, 'a> ModelHead<'h, 'o, 'a> {
    #[must_use]
    pub fn head(&self) -> &'h AttributeHead<'o, 'a> {
        self.head
    }

    /// Retain the actual complete value and stock syntax, including parse holes.
    /// File scope, assignability, read/write roles and targets are later contracts.
    pub fn observe_expression(
        self,
    ) -> Result<AttributeModelExpression<'a>, AttributeExpressionFailure<'a>> {
        let (origin, syntax) =
            observe_expression_value(self.head.selected(), self.head.attribute().reborrow())?;
        Ok(AttributeModelExpression {
            origin,
            syntax,
            modifier: self.modifier,
        })
    }
}
