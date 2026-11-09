//! Whole original interpolation ownership before a private File receiver.

use crate::expr::JsExpr;
use crate::expr::js::JsCoordinateError;
use vize_l1::embed::syntax::EmbedHole;
use vize_l1::markup::{
    NativeChild, NativeInterpolationOperand, NativeInterpolationView, NativeTemplateComponent,
};

mod transfer;

/// A failed join or coordinate transfer leaves the whole operand owned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeInterpolationInputError {
    UnadmittedOriginal,
    Hole(EmbedHole),
    Coordinates(JsCoordinateError),
}

/// The complete original operand and, after its first join, neutral metadata.
///
/// Moving this input preserves the ordinary stock parse observation, comments,
/// diagnostics, intrinsic grammar, full construct and original child identity.
/// It grants no completed File, body cursor or runtime binding access.
///
/// ```compile_fail
/// use vize_l2::lang::js::NativeInterpolationInput;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeInterpolationInput<'static>>();
/// ```
/// ```compile_fail
/// use vize_l2::{expr::JsExpr, lang::js::NativeInterpolationInput};
/// fn raw(expression: JsExpr<'_>) {
///     let _ = NativeInterpolationInput::from_operand(expression);
/// }
/// ```
pub struct NativeInterpolationInput<'a> {
    operand: NativeInterpolationOperand<'a>,
    expression: Option<&'a JsExpr<'a>>,
}

impl<'a> NativeInterpolationInput<'a> {
    /// Move the genuine whole L1 operand, including any typed syntax hole.
    #[must_use]
    pub const fn from_operand(operand: NativeInterpolationOperand<'a>) -> Self {
        Self {
            operand,
            expression: None,
        }
    }

    #[must_use]
    pub const fn operand(&self) -> &NativeInterpolationOperand<'a> {
        &self.operand
    }

    /// Join this actual selected owner and original child before metadata use.
    ///
    /// The first admitted join derives its allocator and full authored source
    /// from that selected owner. It transfers existing AST/decode coordinates
    /// once; later admitted joins reuse metadata while preserving the operand.
    /// A private File consumer must still check its own selected-owner identity,
    /// ordered cursor and attached same-File resolution before node admission.
    pub fn admitted_for<'s>(
        &'s mut self,
        selected: &'s NativeTemplateComponent<'a>,
        child: NativeChild<'s, 'a>,
    ) -> Result<NativeInterpolationInputView<'s, 'a>, NativeInterpolationInputError> {
        if let Some(hole) = self.operand.syntax().hole() {
            return Err(NativeInterpolationInputError::Hole(hole));
        }
        let original = self
            .operand
            .admitted_for(selected, child)
            .ok_or(NativeInterpolationInputError::UnadmittedOriginal)?;
        let expression = if let Some(expression) = self.expression {
            expression
        } else {
            let expression =
                transfer::retain(&original).map_err(NativeInterpolationInputError::Coordinates)?;
            self.expression = Some(expression);
            expression
        };
        Ok(NativeInterpolationInputView {
            input: self,
            original,
            expression,
        })
    }
}

/// A short genuine original join, not a neutral-body or completed File token.
///
/// ```compile_fail
/// use vize_l1::markup::{NativeChild, NativeTemplateComponent};
/// use vize_l2::lang::js::{NativeInterpolationInput, NativeInterpolationInputView};
/// fn escape<'a>(input: &mut NativeInterpolationInput<'a>,
///     selected: &NativeTemplateComponent<'a>, child: NativeChild<'_, 'a>)
///     -> NativeInterpolationInputView<'static, 'a> {
///     match input.admitted_for(selected, child) {
///         Ok(view) => view,
///         Err(_) => loop {},
///     }
/// }
/// ```
/// ```compile_fail
/// use vize_l1::markup::{NativeChild, NativeTemplateComponent};
/// use vize_l2::lang::js::NativeInterpolationInput;
/// fn discard<'a>(mut input: NativeInterpolationInput<'a>,
///     selected: &NativeTemplateComponent<'a>, child: NativeChild<'_, 'a>) {
///     let view = match input.admitted_for(selected, child) {
///         Ok(view) => view,
///         Err(_) => return,
///     };
///     drop(input);
///     let _ = view.original().operand().syntax().comments().count();
/// }
/// ```
pub struct NativeInterpolationInputView<'s, 'a> {
    input: &'s NativeInterpolationInput<'a>,
    original: NativeInterpolationView<'s, 'a>,
    expression: &'a JsExpr<'a>,
}

impl<'s, 'a> NativeInterpolationInputView<'s, 'a> {
    #[must_use]
    pub const fn input(&self) -> &'s NativeInterpolationInput<'a> {
        self.input
    }
    #[must_use]
    pub const fn original(&self) -> &NativeInterpolationView<'s, 'a> {
        &self.original
    }
    #[must_use]
    pub const fn expression(&self) -> &'a JsExpr<'a> {
        self.expression
    }
}

#[cfg(test)]
mod tests;
