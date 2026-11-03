//! Whole original Model operand; no caller AST, range or occurrence list.

use crate::expr::{
    JsExpr,
    js::{JsCoordinateError, JsCoordinates},
};
use vize_l0::Span;
use vize_l1::embed::syntax::EmbedHole;
use vize_l1::markup::{NativeAttribute, NativeAttributeModelExpression, NativeTemplateComponent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeModelInputError {
    UnadmittedOriginal,
    UnsupportedOperand,
    Hole(EmbedHole),
    Coordinates(JsCoordinateError),
}

#[cfg(test)]
mod tests;

/// A distinct original Model owner, never an interpolation or assignment shim.
/// File mutation authority is checked separately against its actual declaration.
/// ```compile_fail
/// use vize_l2::lang::js::NativeModelInput;
/// fn duplicate(input: NativeModelInput<'_>) { let _ = input.clone(); }
/// ```
pub struct NativeModelInput<'a> {
    operand: NativeAttributeModelExpression<'a>,
    expression: Option<&'a JsExpr<'a>>,
    attribute_span: Option<Span>,
}

impl<'a> NativeModelInput<'a> {
    #[must_use]
    pub const fn from_operand(operand: NativeAttributeModelExpression<'a>) -> Self {
        Self {
            operand,
            expression: None,
            attribute_span: None,
        }
    }

    #[must_use]
    pub const fn operand(&self) -> &NativeAttributeModelExpression<'a> {
        &self.operand
    }

    #[must_use]
    pub const fn expression(&self) -> Option<&'a JsExpr<'a>> {
        self.expression
    }

    /// The complete attribute extent derived from the same original tokens.
    #[must_use]
    pub const fn attribute_span(&self) -> Option<Span> {
        self.attribute_span
    }

    #[must_use]
    pub fn matches_expression(&self, expression: &JsExpr<'a>) -> bool {
        self.attribute_span.is_some()
            && self.expression.is_some_and(|actual| {
                core::ptr::eq(actual, expression)
                    && core::ptr::eq(actual.ast, expression.ast)
                    && core::ptr::eq(actual.source, expression.source)
                    && actual.span == expression.span
                    && actual.matches_authored_source(self.operand.syntax().source().authored_root())
                    && matches!((actual.coordinates, expression.coordinates), (Some(own), Some(candidate)) if core::ptr::eq(own, candidate))
            })
    }

    /// Authenticate the actual header, then transfer its one bare Identifier.
    /// Assignability and semantic read/write use come from this Model owner;
    /// they are never manufactured by parsing a generated assignment.
    pub fn admitted_for<'s>(
        &'s mut self,
        selected: &'s NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'s, 'a>,
    ) -> Result<&'a JsExpr<'a>, NativeModelInputError> {
        let syntax = self.operand.syntax();
        if let Some(hole) = syntax.hole() {
            return Err(NativeModelInputError::Hole(hole));
        }
        let original = self
            .operand
            .admitted_for(selected, attribute)
            .ok_or(NativeModelInputError::UnadmittedOriginal)?;
        let admitted = original
            .expression()
            .ok_or(NativeModelInputError::UnadmittedOriginal)?;
        let ast = admitted.expression();
        let source = syntax.source();
        let Some(identifier) = ast.get_identifier_reference() else {
            return Err(NativeModelInputError::UnsupportedOperand);
        };
        if syntax.comments().next().is_some()
            || source.decode_map().is_some()
            || source.text() != identifier.name.as_str()
            || !core::ptr::eq(
                source.authored_root(),
                selected.component().block().root_source(),
            )
        {
            return Err(NativeModelInputError::UnsupportedOperand);
        }
        let value = original
            .attribute()
            .surface()
            .value
            .as_ref()
            .ok_or(NativeModelInputError::UnadmittedOriginal)?;
        let block = selected.component().block();
        let end = match &value.close_quote {
            Some(quote) => {
                block
                    .span_of(quote.text)
                    .ok_or(NativeModelInputError::UnadmittedOriginal)?
                    .end
            }
            None => self.operand.value_span().end,
        };
        let span = Span::new(self.operand.name_span().start, end);
        if !block.contains_block_span(span) {
            return Err(NativeModelInputError::UnadmittedOriginal);
        }
        self.attribute_span = Some(span);
        if let Some(expression) = self.expression {
            return Ok(expression);
        }
        let coordinates = JsCoordinates::checked(
            source.authored_root(),
            source.text(),
            source.span(),
            syntax.parser_prefix(),
            &[],
        )
        .map_err(NativeModelInputError::Coordinates)?;
        let expression = JsExpr::from_retained_in(
            selected.component().allocator(),
            ast,
            source.text(),
            source.span(),
            coordinates,
        )
        .map_err(NativeModelInputError::Coordinates)?;
        self.expression = Some(expression);
        Ok(expression)
    }
}
