//! `v-custom={[value, 'arg', ['a','b']]}` — babel-plugin-jsx's array encoding
//! for a *custom* directive's value, argument and modifiers (#3421).
//!
//! The built-in encodings live next door: `v_model.rs` destructures
//! `v-model={[value, ['trim']]}`, and `v_models.rs` the two-dimensional
//! `v-models` form. This module handles everything else that is not a built-in.

use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_span::GetSpan;
use vize_relief::SourceLocation;
use vize_relief::{DirectiveNode, PropNode, SimpleExpressionNode};

use super::Lowerer;

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Unpack babel-plugin-jsx's array encoding for a custom directive.
    ///
    /// Babel places the elements positionally, so `[val, 'arg', ['a','b']]`
    /// becomes `[dir, val, 'arg', { a: true, b: true }]`. Only the shapes babel
    /// actually encodes are unpacked:
    ///
    /// - `[value]`
    /// - `[value, 'arg']`
    /// - `[value, ['a', 'b']]`
    /// - `[value, 'arg', ['a', 'b']]`
    ///
    /// Every other shape returns `None` and keeps the whole array as the bound
    /// value, which is what vize did before this existed. That fallback is the
    /// point rather than a limitation: a partial unpack would place the elements
    /// it recognizes and silently drop the rest, which is exactly the
    /// degrade-instead-of-diagnose failure #3421 is about. A non-literal value
    /// (`v-custom={someArray}`) never reaches here at all, so a user passing an
    /// array-valued directive keeps passing it.
    pub(crate) fn lower_custom_directive_array(
        &self,
        array: &oxc_ast::ast::ArrayExpression<'_>,
        name: &'a str,
        loc: &SourceLocation,
    ) -> Option<PropNode<'a>> {
        let mut elements = std::vec::Vec::new();
        for element in &array.elements {
            match element {
                ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_) => {
                    return None;
                }
                _ => elements.push(element.as_expression()?),
            }
        }

        let value_expr = *elements.first()?;
        let (arg, modifiers) = match elements.len() {
            1 => (None, None),
            2 => match elements[1] {
                Expression::StringLiteral(argument) => (Some(argument), None),
                Expression::ArrayExpression(modifiers) => (None, Some(modifiers)),
                _ => return None,
            },
            3 => match (elements[1], elements[2]) {
                (Expression::StringLiteral(argument), Expression::ArrayExpression(modifiers)) => {
                    (Some(argument), Some(modifiers))
                }
                _ => return None,
            },
            _ => return None,
        };

        // Validate before building: a modifiers list holding anything but string
        // literals is not babel's encoding, and dropping those entries would be
        // the same silent degradation.
        let modifier_names = match modifiers {
            None => std::vec::Vec::new(),
            Some(modifiers) => modifiers
                .elements
                .iter()
                .map(|element| match element.as_expression() {
                    Some(Expression::StringLiteral(name)) => Some(name.value.as_str()),
                    _ => None,
                })
                .collect::<Option<std::vec::Vec<_>>>()?,
        };

        let mut directive = DirectiveNode::new(self.bump(), name, loc.clone());
        directive.exp = Some(self.dyn_expr(value_expr.span()));
        if let Some(argument) = arg {
            directive.arg = Some(self.static_expr(
                self.bump().alloc_str(argument.value.as_str()),
                argument.span,
            ));
        }
        for modifier in modifier_names {
            directive.modifiers.push(SimpleExpressionNode::new(
                self.bump().alloc_str(modifier),
                false,
                loc.clone(),
            ));
        }
        Some(PropNode::Directive(self.boxed(directive)))
    }
}
