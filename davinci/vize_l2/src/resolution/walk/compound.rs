use super::{ReferenceSink, ResolutionError, ResolutionErrorKind, Resolver, Usage};
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind,
    PropertyKey,
};
use oxc_span::GetSpan;
impl<'a, S: ReferenceSink<'a>> Resolver<'a, '_, S> {
    pub(super) fn array(
        &mut self,
        value: &ArrayExpression<'a>,
        next: usize,
    ) -> Result<(), ResolutionError> {
        for element in &value.elements {
            self.visit(element.span(), next)?;
            match element {
                ArrayExpressionElement::Elision(_) => {}
                ArrayExpressionElement::SpreadElement(spread) => {
                    self.expression(&spread.argument, next)?;
                }
                other => self.expression(other.to_expression(), next)?,
            }
        }
        Ok(())
    }
    pub(super) fn object(
        &mut self,
        value: &ObjectExpression<'a>,
        next: usize,
    ) -> Result<(), ResolutionError> {
        for property in &value.properties {
            self.visit(property.span(), next)?;
            match property {
                ObjectPropertyKind::SpreadProperty(spread) => {
                    self.expression(&spread.argument, next)?;
                }
                ObjectPropertyKind::ObjectProperty(property) if !property.method => {
                    if property.shorthand {
                        let Expression::Identifier(identifier) = &property.value else {
                            return Err(
                                self.fail(property.span, ResolutionErrorKind::UnsupportedSyntax)
                            );
                        };
                        self.identifier(identifier, Usage::Read, true)?;
                    } else {
                        if property.computed {
                            let Some(key) = property.key.as_expression() else {
                                return Err(self
                                    .fail(property.span, ResolutionErrorKind::UnsupportedSyntax));
                            };
                            self.expression(key, next)?;
                        } else if matches!(property.key, PropertyKey::PrivateIdentifier(_)) {
                            return Err(
                                self.fail(property.span, ResolutionErrorKind::UnsupportedSyntax)
                            );
                        }
                        self.expression(&property.value, next)?;
                    }
                }
                other => {
                    return Err(self.fail(other.span(), ResolutionErrorKind::UnsupportedSyntax));
                }
            }
        }
        Ok(())
    }
}
