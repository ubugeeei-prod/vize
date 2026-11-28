// Generated from OXC's pinned immutable walk; see mod.rs for provenance.
use super::super::LentNodes;
use super::{
    walk_arguments, walk_array_expression_elements, walk_arrow_function_expression,
    walk_assignment_target_maybe_default, walk_assignment_target_properties, walk_await_expression,
    walk_big_int_literal, walk_boolean_literal, walk_chain_expression, walk_class, walk_directives,
    walk_expressions, walk_function, walk_hashbang, walk_import_expression, walk_jsx_element,
    walk_jsx_fragment, walk_null_literal, walk_numeric_literal, walk_object_property_kinds,
    walk_parenthesized_expression, walk_private_identifier, walk_reg_exp_literal,
    walk_sequence_expression, walk_span, walk_statements, walk_string_literal, walk_super,
    walk_template_elements, walk_ts_as_expression, walk_ts_instantiation_expression,
    walk_ts_non_null_expression, walk_ts_satisfies_expression, walk_ts_type_assertion,
    walk_ts_type_parameter_instantiation, walk_v8_intrinsic_expression, walk_yield_expression,
};
use oxc_ast::ast::{
    ArrayAssignmentTarget, ArrayExpression, ArrayExpressionElement, AssignmentExpression,
    AssignmentTarget, AssignmentTargetPattern, AssignmentTargetRest, BinaryExpression,
    BindingIdentifier, CallExpression, ComputedMemberExpression, ConditionalExpression, Elision,
    Expression, IdentifierName, IdentifierReference, ImportMeta, LabelIdentifier,
    LogicalExpression, MemberExpression, NewExpression, NewTarget, ObjectAssignmentTarget,
    ObjectExpression, ObjectProperty, ObjectPropertyKind, PrivateFieldExpression,
    PrivateInExpression, Program, PropertyKey, SimpleAssignmentTarget, SpreadElement,
    StaticMemberExpression, TaggedTemplateExpression, TemplateElement, TemplateLiteral,
    ThisExpression, UnaryExpression, UpdateExpression, match_assignment_target_pattern,
    match_expression, match_member_expression, match_simple_assignment_target,
};
use oxc_syntax::scope::ScopeFlags;

#[inline]
pub(crate) fn walk_program<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Program<'a>) {
    walk_span(visitor, &it.span);
    if let Some(hashbang) = &it.hashbang {
        walk_hashbang(visitor, hashbang);
    }
    walk_directives(visitor, &it.directives);
    walk_statements(visitor, &it.body);
}

pub(crate) fn walk_expression<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Expression<'a>) {
    // No `AstKind` for this type
    match it {
        Expression::BooleanLiteral(it) => walk_boolean_literal(visitor, it),
        Expression::NullLiteral(it) => walk_null_literal(visitor, it),
        Expression::NumericLiteral(it) => walk_numeric_literal(visitor, it),
        Expression::BigIntLiteral(it) => walk_big_int_literal(visitor, it),
        Expression::RegExpLiteral(it) => walk_reg_exp_literal(visitor, it),
        Expression::StringLiteral(it) => walk_string_literal(visitor, it),
        Expression::TemplateLiteral(it) => walk_template_literal(visitor, it),
        Expression::Identifier(it) => walk_identifier_reference(visitor, it),
        Expression::Super(it) => walk_super(visitor, it),
        Expression::ArrayExpression(it) => walk_array_expression(visitor, it),
        Expression::ArrowFunctionExpression(it) => walk_arrow_function_expression(visitor, it),
        Expression::AssignmentExpression(it) => walk_assignment_expression(visitor, it),
        Expression::AwaitExpression(it) => walk_await_expression(visitor, it),
        Expression::BinaryExpression(it) => walk_binary_expression(visitor, it),
        Expression::CallExpression(it) => walk_call_expression(visitor, it),
        Expression::ChainExpression(it) => walk_chain_expression(visitor, it),
        Expression::ClassExpression(it) => walk_class(visitor, it),
        Expression::ConditionalExpression(it) => walk_conditional_expression(visitor, it),
        Expression::FunctionExpression(it) => {
            let flags = ScopeFlags::Function;
            walk_function(visitor, it, flags)
        }
        Expression::ImportExpression(it) => walk_import_expression(visitor, it),
        Expression::LogicalExpression(it) => walk_logical_expression(visitor, it),
        Expression::NewExpression(it) => walk_new_expression(visitor, it),
        Expression::ObjectExpression(it) => walk_object_expression(visitor, it),
        Expression::ParenthesizedExpression(it) => walk_parenthesized_expression(visitor, it),
        Expression::SequenceExpression(it) => walk_sequence_expression(visitor, it),
        Expression::TaggedTemplateExpression(it) => walk_tagged_template_expression(visitor, it),
        Expression::ThisExpression(it) => walk_this_expression(visitor, it),
        Expression::UnaryExpression(it) => walk_unary_expression(visitor, it),
        Expression::UpdateExpression(it) => walk_update_expression(visitor, it),
        Expression::YieldExpression(it) => walk_yield_expression(visitor, it),
        Expression::PrivateInExpression(it) => walk_private_in_expression(visitor, it),
        Expression::ImportMeta(it) => walk_import_meta(visitor, it),
        Expression::NewTarget(it) => walk_new_target(visitor, it),
        Expression::JSXElement(it) => walk_jsx_element(visitor, it),
        Expression::JSXFragment(it) => walk_jsx_fragment(visitor, it),
        Expression::TSAsExpression(it) => walk_ts_as_expression(visitor, it),
        Expression::TSSatisfiesExpression(it) => walk_ts_satisfies_expression(visitor, it),
        Expression::TSTypeAssertion(it) => walk_ts_type_assertion(visitor, it),
        Expression::TSNonNullExpression(it) => walk_ts_non_null_expression(visitor, it),
        Expression::TSInstantiationExpression(it) => walk_ts_instantiation_expression(visitor, it),
        Expression::V8IntrinsicExpression(it) => walk_v8_intrinsic_expression(visitor, it),
        match_member_expression!(Expression) => {
            walk_member_expression(visitor, it.to_member_expression())
        }
    }
}

#[inline]
pub(crate) fn walk_identifier_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a IdentifierName<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_identifier_reference<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a IdentifierReference<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_binding_identifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a BindingIdentifier<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_label_identifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a LabelIdentifier<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_this_expression<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a ThisExpression) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_array_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArrayExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_array_expression_elements(visitor, &it.elements);
}

#[inline]
pub(crate) fn walk_array_expression_element<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArrayExpressionElement<'a>,
) {
    // No `AstKind` for this type
    match it {
        ArrayExpressionElement::SpreadElement(it) => walk_spread_element(visitor, it),
        ArrayExpressionElement::Elision(it) => walk_elision(visitor, it),
        match_expression!(ArrayExpressionElement) => walk_expression(visitor, it.to_expression()),
    }
}

#[inline]
pub(crate) fn walk_elision<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Elision) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_object_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ObjectExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_object_property_kinds(visitor, &it.properties);
}

#[inline]
pub(crate) fn walk_object_property_kind<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ObjectPropertyKind<'a>,
) {
    // No `AstKind` for this type
    match it {
        ObjectPropertyKind::ObjectProperty(it) => walk_object_property(visitor, it),
        ObjectPropertyKind::SpreadProperty(it) => walk_spread_element(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_object_property<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ObjectProperty<'a>,
) {
    walk_span(visitor, &it.span);
    walk_property_key(visitor, &it.key);
    walk_expression(visitor, &it.value);
}

#[inline]
pub(crate) fn walk_property_key<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a PropertyKey<'a>) {
    // No `AstKind` for this type
    match it {
        PropertyKey::StaticIdentifier(it) => walk_identifier_name(visitor, it),
        PropertyKey::PrivateIdentifier(it) => walk_private_identifier(visitor, it),
        match_expression!(PropertyKey) => walk_expression(visitor, it.to_expression()),
    }
}

#[inline]
pub(crate) fn walk_template_literal<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TemplateLiteral<'a>,
) {
    walk_span(visitor, &it.span);
    walk_template_elements(visitor, &it.quasis);
    walk_expressions(visitor, &it.expressions);
}

#[inline]
pub(crate) fn walk_tagged_template_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TaggedTemplateExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.tag);
    if let Some(type_arguments) = &it.type_arguments {
        walk_ts_type_parameter_instantiation(visitor, type_arguments);
    }
    walk_template_literal(visitor, &it.quasi);
}

#[inline]
pub(crate) fn walk_template_element<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TemplateElement<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_member_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a MemberExpression<'a>,
) {
    // No `AstKind` for this type
    match it {
        MemberExpression::ComputedMemberExpression(it) => {
            walk_computed_member_expression(visitor, it)
        }
        MemberExpression::StaticMemberExpression(it) => walk_static_member_expression(visitor, it),
        MemberExpression::PrivateFieldExpression(it) => walk_private_field_expression(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_computed_member_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ComputedMemberExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.object);
    walk_expression(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_static_member_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a StaticMemberExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.object);
    walk_identifier_name(visitor, &it.property);
}

#[inline]
pub(crate) fn walk_private_field_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a PrivateFieldExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.object);
    walk_private_identifier(visitor, &it.field);
}

#[inline]
pub(crate) fn walk_call_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a CallExpression<'a>,
) {
    visitor.call_expression(it);

    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.callee);
    if let Some(type_arguments) = &it.type_arguments {
        walk_ts_type_parameter_instantiation(visitor, type_arguments);
    }
    walk_arguments(visitor, &it.arguments);
}

#[inline]
pub(crate) fn walk_new_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a NewExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.callee);
    if let Some(type_arguments) = &it.type_arguments {
        walk_ts_type_parameter_instantiation(visitor, type_arguments);
    }
    walk_arguments(visitor, &it.arguments);
}

#[inline]
pub(crate) fn walk_import_meta<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a ImportMeta) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_new_target<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a NewTarget) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_spread_element<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a SpreadElement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.argument);
}

#[inline]
pub(crate) fn walk_update_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a UpdateExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_simple_assignment_target(visitor, &it.argument);
}

#[inline]
pub(crate) fn walk_unary_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a UnaryExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.argument);
}

#[inline]
pub(crate) fn walk_binary_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a BinaryExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.left);
    walk_expression(visitor, &it.right);
}

#[inline]
pub(crate) fn walk_private_in_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a PrivateInExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_private_identifier(visitor, &it.left);
    walk_expression(visitor, &it.right);
}

#[inline]
pub(crate) fn walk_logical_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a LogicalExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.left);
    walk_expression(visitor, &it.right);
}

#[inline]
pub(crate) fn walk_conditional_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ConditionalExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.test);
    walk_expression(visitor, &it.consequent);
    walk_expression(visitor, &it.alternate);
}

#[inline]
pub(crate) fn walk_assignment_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_assignment_target(visitor, &it.left);
    walk_expression(visitor, &it.right);
}

#[inline]
pub(crate) fn walk_assignment_target<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentTarget<'a>,
) {
    // No `AstKind` for this type
    match it {
        match_simple_assignment_target!(AssignmentTarget) => {
            walk_simple_assignment_target(visitor, it.to_simple_assignment_target())
        }
        match_assignment_target_pattern!(AssignmentTarget) => {
            walk_assignment_target_pattern(visitor, it.to_assignment_target_pattern())
        }
    }
}

pub(crate) fn walk_simple_assignment_target<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a SimpleAssignmentTarget<'a>,
) {
    // No `AstKind` for this type
    match it {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(it) => {
            walk_identifier_reference(visitor, it)
        }
        SimpleAssignmentTarget::TSAsExpression(it) => walk_ts_as_expression(visitor, it),
        SimpleAssignmentTarget::TSSatisfiesExpression(it) => {
            walk_ts_satisfies_expression(visitor, it)
        }
        SimpleAssignmentTarget::TSNonNullExpression(it) => walk_ts_non_null_expression(visitor, it),
        SimpleAssignmentTarget::TSTypeAssertion(it) => walk_ts_type_assertion(visitor, it),
        match_member_expression!(SimpleAssignmentTarget) => {
            walk_member_expression(visitor, it.to_member_expression())
        }
    }
}

#[inline]
pub(crate) fn walk_assignment_target_pattern<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentTargetPattern<'a>,
) {
    // No `AstKind` for this type
    match it {
        AssignmentTargetPattern::ArrayAssignmentTarget(it) => {
            walk_array_assignment_target(visitor, it)
        }
        AssignmentTargetPattern::ObjectAssignmentTarget(it) => {
            walk_object_assignment_target(visitor, it)
        }
    }
}

#[inline]
pub(crate) fn walk_array_assignment_target<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArrayAssignmentTarget<'a>,
) {
    walk_span(visitor, &it.span);
    for el in it.elements.iter().flatten() {
        walk_assignment_target_maybe_default(visitor, el);
    }
    if let Some(rest) = &it.rest {
        walk_assignment_target_rest(visitor, rest);
    }
}

#[inline]
pub(crate) fn walk_object_assignment_target<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ObjectAssignmentTarget<'a>,
) {
    walk_span(visitor, &it.span);
    walk_assignment_target_properties(visitor, &it.properties);
    if let Some(rest) = &it.rest {
        walk_assignment_target_rest(visitor, rest);
    }
}

#[inline]
pub(crate) fn walk_assignment_target_rest<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentTargetRest<'a>,
) {
    walk_span(visitor, &it.span);
    walk_assignment_target(visitor, &it.target);
}
