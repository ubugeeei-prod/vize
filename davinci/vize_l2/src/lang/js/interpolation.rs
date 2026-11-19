//! Whole original interpolation ownership before a private File receiver.

use crate::expr::JsExpr;
use crate::expr::js::JsCoordinateError;
use vize_l1::embed::syntax::EmbedHole;
use vize_l1::markup::{
    ChildView, InterpolationOperand, InterpolationView, NativeTemplateComponent,
};

mod transfer;

/// A failed join or coordinate transfer leaves the whole operand owned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationInputError {
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
/// use vize_l2::lang::js::InterpolationInput;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<InterpolationInput<'static>>();
/// ```
/// ```compile_fail
/// use vize_l2::{expr::JsExpr, lang::js::InterpolationInput};
/// fn raw(expression: JsExpr<'_>) {
///     let _ = InterpolationInput::from_operand(expression);
/// }
/// ```
pub struct InterpolationInput<'a> {
    operand: InterpolationOperand<'a>,
    expression: Option<&'a JsExpr<'a>>,
}

impl<'a> InterpolationInput<'a> {
    /// Move the genuine whole L1 operand, including any typed syntax hole.
    #[must_use]
    pub const fn from_operand(operand: InterpolationOperand<'a>) -> Self {
        Self {
            operand,
            expression: None,
        }
    }

    #[must_use]
    pub const fn operand(&self) -> &InterpolationOperand<'a> {
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
        child: ChildView<'s, 'a>,
    ) -> Result<InterpolationInputView<'s, 'a>, InterpolationInputError> {
        if let Some(hole) = self.operand.syntax().hole() {
            return Err(InterpolationInputError::Hole(hole));
        }
        let original = self
            .operand
            .admitted_for(selected, child)
            .ok_or(InterpolationInputError::UnadmittedOriginal)?;
        let expression = if let Some(expression) = self.expression {
            expression
        } else {
            let expression =
                transfer::retain(&original).map_err(InterpolationInputError::Coordinates)?;
            self.expression = Some(expression);
            expression
        };
        Ok(InterpolationInputView {
            input: self,
            original,
            expression,
        })
    }
}

/// A short genuine original join, not a neutral-body or completed File token.
///
/// ```compile_fail
/// use vize_l1::markup::{ChildView, NativeTemplateComponent};
/// use vize_l2::lang::js::{InterpolationInput, InterpolationInputView};
/// fn escape<'a>(input: &mut InterpolationInput<'a>,
///     selected: &NativeTemplateComponent<'a>, child: ChildView<'_, 'a>)
///     -> InterpolationInputView<'static, 'a> {
///     match input.admitted_for(selected, child) {
///         Ok(view) => view,
///         Err(_) => loop {},
///     }
/// }
/// ```
/// ```compile_fail
/// use vize_l1::markup::{ChildView, NativeTemplateComponent};
/// use vize_l2::lang::js::InterpolationInput;
/// fn discard<'a>(mut input: InterpolationInput<'a>,
///     selected: &NativeTemplateComponent<'a>, child: ChildView<'_, 'a>) {
///     let view = match input.admitted_for(selected, child) {
///         Ok(view) => view,
///         Err(_) => return,
///     };
///     drop(input);
///     let _ = view.original().operand().syntax().comments().count();
/// }
/// ```
pub struct InterpolationInputView<'s, 'a> {
    input: &'s InterpolationInput<'a>,
    original: InterpolationView<'s, 'a>,
    expression: &'a JsExpr<'a>,
}

impl<'s, 'a> InterpolationInputView<'s, 'a> {
    #[must_use]
    pub const fn input(&self) -> &'s InterpolationInput<'a> {
        self.input
    }
    #[must_use]
    pub const fn original(&self) -> &InterpolationView<'s, 'a> {
        &self.original
    }
    #[must_use]
    pub const fn expression(&self) -> &'a JsExpr<'a> {
        self.expression
    }
}

#[cfg(test)]
mod tests;
