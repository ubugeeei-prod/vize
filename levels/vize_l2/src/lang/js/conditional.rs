//! Whole conditional header ownership, distinct from an interpolation input.

use crate::expr::{
    JsExpr,
    js::{JsCoordinateError, JsCoordinates},
};
use vize_l1::embed::syntax::EmbedHole;
use vize_l1::markup::{
    NativeAttribute, NativeAttributeExpression, NativeConditionKind, NativeTemplateComponent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeConditionalInputError {
    UnadmittedOriginal,
    UnsupportedCondition,
    Hole(EmbedHole),
    Coordinates(JsCoordinateError),
}

/// No raw AST, source span or interpolation can construct this whole owner.
/// ```compile_fail
/// use vize_l2::lang::js::NativeConditionalInput;
/// fn duplicate(input: NativeConditionalInput<'_>) { let _ = input.clone(); }
/// ```
pub struct NativeConditionalInput<'a> {
    operand: NativeAttributeExpression<'a>,
    expression: Option<&'a JsExpr<'a>>,
}

impl<'a> NativeConditionalInput<'a> {
    #[must_use]
    pub const fn from_operand(operand: NativeAttributeExpression<'a>) -> Self {
        Self {
            operand,
            expression: None,
        }
    }
    #[must_use]
    pub const fn operand(&self) -> &NativeAttributeExpression<'a> {
        &self.operand
    }
    #[must_use]
    pub const fn expression(&self) -> Option<&'a JsExpr<'a>> {
        self.expression
    }

    /// Only the once-transferred original wrapper can satisfy this receipt.
    #[must_use]
    pub fn matches_expression(&self, expression: &JsExpr<'a>) -> bool {
        self.expression.is_some_and(|actual| {
            core::ptr::eq(actual, expression)
                && core::ptr::eq(actual.ast, expression.ast)
                && core::ptr::eq(actual.source, expression.source)
                && actual.span == expression.span
                && actual.matches_authored_source(self.operand.syntax().source().authored_root())
                && matches!((actual.coordinates, expression.coordinates), (Some(own), Some(candidate)) if core::ptr::eq(own, candidate))
        })
    }

    /// Authenticate the actual header before transferring its existing AST once.
    pub fn admitted_for<'s>(
        &'s mut self,
        selected: &'s NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'s, 'a>,
    ) -> Result<&'a JsExpr<'a>, NativeConditionalInputError> {
        if self.operand.kind() != NativeConditionKind::If {
            return Err(NativeConditionalInputError::UnsupportedCondition);
        }
        let syntax = self.operand.syntax();
        if let Some(hole) = syntax.hole() {
            return Err(NativeConditionalInputError::Hole(hole));
        }
        let original = self
            .operand
            .admitted_for(selected, attribute)
            .ok_or(NativeConditionalInputError::UnadmittedOriginal)?;
        let admitted = original
            .expression()
            .ok_or(NativeConditionalInputError::UnadmittedOriginal)?;
        let ast = admitted.expression();
        let source = syntax.source();
        let Some(identifier) = ast.get_identifier_reference() else {
            return Err(NativeConditionalInputError::UnsupportedCondition);
        };
        if syntax.comments().next().is_some()
            || source.decode_map().is_some()
            || source.text() != identifier.name.as_str()
            || !core::ptr::eq(
                source.authored_root(),
                selected.component().block().root_source(),
            )
        {
            return Err(NativeConditionalInputError::UnsupportedCondition);
        }
        if let Some(expression) = self.expression {
            return Ok(expression);
        }
        let allocator = selected.component().allocator();
        let coordinates = JsCoordinates::checked(
            source.authored_root(),
            source.text(),
            source.span(),
            syntax.parser_prefix(),
            &[],
        )
        .map_err(NativeConditionalInputError::Coordinates)?;
        let expression =
            JsExpr::from_retained_in(allocator, ast, source.text(), source.span(), coordinates)
                .map_err(NativeConditionalInputError::Coordinates)?;
        self.expression = Some(expression);
        Ok(expression)
    }
}

#[cfg(test)]
mod tests;
