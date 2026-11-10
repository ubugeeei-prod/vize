//! Vue's initializer and macro-call binding classification rules.

use super::super::super::{MacroKind, lookup};
use super::SetupBindingKind;
use oxc_ast::ast::{CallExpression, Expression, VariableDeclarationKind};

fn is_callee(call: &CallExpression<'_>, name: &str) -> bool {
    matches!(&call.callee, Expression::Identifier(id) if id.name == name)
}

pub(super) fn is_const_macro_call(call: &CallExpression<'_>) -> bool {
    matches!(&call.callee, Expression::Identifier(id) if matches!(
        lookup(id.name.as_str()).map(|row| row.kind),
        Some(
            MacroKind::DefineProps
                | MacroKind::DefineEmits
                | MacroKind::WithDefaults
                | MacroKind::DefineSlots
        )
    ))
}

/// `defineProps()` or `withDefaults(defineProps(), ...)`, through TS wrappers.
pub(super) fn destructures_props(init: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = transparent(init) else {
        return false;
    };
    is_callee(call, super::super::super::DEFINE_PROPS)
        || (is_callee(call, super::super::super::WITH_DEFAULTS)
            && matches!(
                call.arguments.first(),
                Some(oxc_ast::ast::Argument::CallExpression(inner))
                    if is_callee(inner, super::super::super::DEFINE_PROPS)
            ))
}

/// The call an initializer evaluates to, through TS wrappers and parentheses.
pub(super) fn call<'e, 'a>(expression: &'e Expression<'a>) -> Option<&'e CallExpression<'a>> {
    match transparent(expression) {
        Expression::CallExpression(call) => Some(call),
        _ => None,
    }
}

fn transparent<'e, 'a>(expression: &'e Expression<'a>) -> &'e Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(inner) => transparent(&inner.expression),
        Expression::TSAsExpression(inner) => transparent(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => transparent(&inner.expression),
        Expression::TSTypeAssertion(inner) => transparent(&inner.expression),
        Expression::TSNonNullExpression(inner) => transparent(&inner.expression),
        Expression::TSInstantiationExpression(inner) => transparent(&inner.expression),
        other => other,
    }
}

/// A declaration's kind by Vue's static and never-ref tests.
pub(super) fn by_initializer(
    kind: VariableDeclarationKind,
    init: &Expression<'_>,
    hoist_static: bool,
) -> SetupBindingKind {
    if kind != VariableDeclarationKind::Const {
        return SetupBindingKind::SetupLet;
    }
    if hoist_static && is_static_node(init) {
        return SetupBindingKind::LiteralConst;
    }
    if can_never_be_ref(init) {
        SetupBindingKind::SetupConst
    } else {
        SetupBindingKind::SetupMaybeRef
    }
}

fn can_never_be_ref(init: &Expression<'_>) -> bool {
    match transparent(init) {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::UnaryExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_) => true,
        Expression::SequenceExpression(sequence) => {
            sequence.expressions.last().is_some_and(can_never_be_ref)
        }
        _ => false,
    }
}

pub(super) fn is_static_node(init: &Expression<'_>) -> bool {
    match transparent(init) {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_) => true,
        Expression::UnaryExpression(unary) => is_static_node(&unary.argument),
        Expression::LogicalExpression(logical) => {
            is_static_node(&logical.left) && is_static_node(&logical.right)
        }
        Expression::BinaryExpression(binary) => {
            is_static_node(&binary.left) && is_static_node(&binary.right)
        }
        Expression::ConditionalExpression(conditional) => {
            is_static_node(&conditional.test)
                && is_static_node(&conditional.consequent)
                && is_static_node(&conditional.alternate)
        }
        Expression::SequenceExpression(sequence) => sequence.expressions.iter().all(is_static_node),
        Expression::TemplateLiteral(template) => template.expressions.iter().all(is_static_node),
        _ => false,
    }
}
