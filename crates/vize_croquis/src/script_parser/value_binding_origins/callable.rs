//! Closure-stable callable authority from the existing script walk.
//!
//! A const arrow, an unnamed function expression, or a function declaration is
//! stable across nested callbacks. Aliases follow the nearest binding only.
//! Type wrappers, calls, members, `var`/`let`, and shadowed names are not
//! authority. Listener projection still omits a copied branch guard when the
//! guard never spells the handler (#11747); this record does not broaden that.

use super::{ValueBindingKey, ValueBindingOrigins};
use oxc_ast::ast::Expression;

impl ValueBindingOrigins {
    pub(super) fn note_callable_initializer(
        &mut self,
        key: ValueBindingKey,
        init: &Expression<'_>,
    ) {
        if !self.key_names.contains_key(&key) {
            return;
        }
        let (callable, asserted) = classify_initializer(init);
        if asserted {
            self.asserted_keys.insert(key);
        } else if callable {
            self.callable_keys.insert(key);
        }
    }

    pub(super) fn note_declared_function(&mut self, key: ValueBindingKey) {
        if self.key_names.contains_key(&key) {
            self.callable_keys.insert(key);
        }
    }
}

fn classify_initializer(init: &Expression<'_>) -> (bool, bool) {
    let expression = peel_parentheses(init);
    if is_type_wrapper(expression) {
        return (false, true);
    }
    let callable = match expression {
        Expression::ArrowFunctionExpression(_) => true,
        Expression::FunctionExpression(function) => function.id.is_none(),
        _ => false,
    };
    (callable, false)
}

fn peel_parentheses<'a>(mut expression: &'a Expression<'a>) -> &'a Expression<'a> {
    while let Expression::ParenthesizedExpression(parenthesized) = expression {
        expression = &parenthesized.expression;
    }
    expression
}

fn is_type_wrapper(expression: &Expression<'_>) -> bool {
    matches!(
        expression,
        Expression::TSAsExpression(_)
            | Expression::TSTypeAssertion(_)
            | Expression::TSSatisfiesExpression(_)
            | Expression::TSNonNullExpression(_)
            | Expression::TSInstantiationExpression(_)
    )
}
