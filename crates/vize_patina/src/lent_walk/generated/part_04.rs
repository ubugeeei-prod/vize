// Generated from OXC's pinned immutable walk; see mod.rs for provenance.
use super::super::LentNodes;
use super::{
    walk_binding_identifier, walk_boolean_literal, walk_expression, walk_identifier_name,
    walk_identifier_reference, walk_js_doc_non_nullable_type, walk_js_doc_nullable_type,
    walk_js_doc_unknown_type, walk_jsx_attribute_items, walk_jsx_children, walk_numeric_literal,
    walk_span, walk_template_literal, walk_this_expression, walk_ts_any_keyword,
    walk_ts_big_int_keyword, walk_ts_boolean_keyword, walk_ts_constructor_type,
    walk_ts_enum_members, walk_ts_function_type, walk_ts_import_type, walk_ts_infer_type,
    walk_ts_intrinsic_keyword, walk_ts_mapped_type, walk_ts_never_keyword, walk_ts_null_keyword,
    walk_ts_number_keyword, walk_ts_object_keyword, walk_ts_string_keyword, walk_ts_symbol_keyword,
    walk_ts_template_literal_type, walk_ts_this_type, walk_ts_tuple_elements, walk_ts_type_literal,
    walk_ts_type_parameter_instantiation, walk_ts_type_predicate, walk_ts_type_query,
    walk_ts_type_reference, walk_ts_types, walk_ts_undefined_keyword, walk_ts_unknown_keyword,
    walk_ts_void_keyword, walk_unary_expression,
};
use oxc_ast::ast::{
    BigIntLiteral, JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild,
    JSXClosingElement, JSXClosingFragment, JSXElement, JSXElementName, JSXEmptyExpression,
    JSXExpression, JSXExpressionContainer, JSXFragment, JSXIdentifier, JSXMemberExpression,
    JSXMemberExpressionObject, JSXNamespacedName, JSXOpeningElement, JSXOpeningFragment,
    JSXSpreadAttribute, JSXSpreadChild, JSXText, RegExpLiteral, StringLiteral, TSArrayType,
    TSConditionalType, TSEnumBody, TSEnumDeclaration, TSEnumMember, TSEnumMemberName,
    TSIndexedAccessType, TSIntersectionType, TSLiteral, TSLiteralType, TSNamedTupleMember,
    TSOptionalType, TSParenthesizedType, TSRestType, TSThisParameter, TSTupleElement, TSTupleType,
    TSType, TSTypeAnnotation, TSTypeOperator, TSUnionType, match_expression, match_ts_type,
};

#[inline]
pub(crate) fn walk_string_literal<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a StringLiteral<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_big_int_literal<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a BigIntLiteral<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_reg_exp_literal<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a RegExpLiteral<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_jsx_element<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a JSXElement<'a>) {
    if !visitor.enter_jsx_element(it) {
        return;
    }

    walk_span(visitor, &it.span);
    walk_jsx_opening_element(visitor, &it.opening_element);
    walk_jsx_children(visitor, &it.children);
    if let Some(closing_element) = &it.closing_element {
        walk_jsx_closing_element(visitor, closing_element);
    }
    visitor.leave_jsx_element(it);
}

#[inline]
pub(crate) fn walk_jsx_opening_element<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXOpeningElement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_jsx_element_name(visitor, &it.name);
    if let Some(type_arguments) = &it.type_arguments {
        walk_ts_type_parameter_instantiation(visitor, type_arguments);
    }
    walk_jsx_attribute_items(visitor, &it.attributes);
}

#[inline]
pub(crate) fn walk_jsx_closing_element<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXClosingElement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_jsx_element_name(visitor, &it.name);
}

#[inline]
pub(crate) fn walk_jsx_fragment<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a JSXFragment<'a>) {
    if !visitor.enter_jsx_fragment(it) {
        return;
    }

    walk_span(visitor, &it.span);
    walk_jsx_opening_fragment(visitor, &it.opening_fragment);
    walk_jsx_children(visitor, &it.children);
    walk_jsx_closing_fragment(visitor, &it.closing_fragment);
    visitor.leave_jsx_fragment(it);
}

#[inline]
pub(crate) fn walk_jsx_opening_fragment<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXOpeningFragment,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_jsx_closing_fragment<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXClosingFragment,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_jsx_element_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXElementName<'a>,
) {
    // No `AstKind` for this type
    match it {
        JSXElementName::Identifier(it) => walk_jsx_identifier(visitor, it),
        JSXElementName::IdentifierReference(it) => walk_identifier_reference(visitor, it),
        JSXElementName::NamespacedName(it) => walk_jsx_namespaced_name(visitor, it),
        JSXElementName::MemberExpression(it) => walk_jsx_member_expression(visitor, it),
        JSXElementName::ThisExpression(it) => walk_this_expression(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_jsx_namespaced_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXNamespacedName<'a>,
) {
    walk_span(visitor, &it.span);
    walk_jsx_identifier(visitor, &it.namespace);
    walk_jsx_identifier(visitor, &it.name);
}

#[inline]
pub(crate) fn walk_jsx_member_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXMemberExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_jsx_member_expression_object(visitor, &it.object);
    walk_jsx_identifier(visitor, &it.property);
}

#[inline]
pub(crate) fn walk_jsx_member_expression_object<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXMemberExpressionObject<'a>,
) {
    // No `AstKind` for this type
    match it {
        JSXMemberExpressionObject::IdentifierReference(it) => {
            walk_identifier_reference(visitor, it)
        }
        JSXMemberExpressionObject::MemberExpression(it) => walk_jsx_member_expression(visitor, it),
        JSXMemberExpressionObject::ThisExpression(it) => walk_this_expression(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_jsx_expression_container<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXExpressionContainer<'a>,
) {
    walk_span(visitor, &it.span);
    walk_jsx_expression(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_jsx_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXExpression<'a>,
) {
    // No `AstKind` for this type
    match it {
        JSXExpression::EmptyExpression(it) => walk_jsx_empty_expression(visitor, it),
        match_expression!(JSXExpression) => walk_expression(visitor, it.to_expression()),
    }
}

#[inline]
pub(crate) fn walk_jsx_empty_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXEmptyExpression,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_jsx_attribute_item<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXAttributeItem<'a>,
) {
    // No `AstKind` for this type
    match it {
        JSXAttributeItem::Attribute(it) => walk_jsx_attribute(visitor, it),
        JSXAttributeItem::SpreadAttribute(it) => walk_jsx_spread_attribute(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_jsx_attribute<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a JSXAttribute<'a>) {
    walk_span(visitor, &it.span);
    walk_jsx_attribute_name(visitor, &it.name);
    if let Some(value) = &it.value {
        walk_jsx_attribute_value(visitor, value);
    }
}

#[inline]
pub(crate) fn walk_jsx_spread_attribute<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXSpreadAttribute<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.argument);
}

#[inline]
pub(crate) fn walk_jsx_attribute_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXAttributeName<'a>,
) {
    // No `AstKind` for this type
    match it {
        JSXAttributeName::Identifier(it) => walk_jsx_identifier(visitor, it),
        JSXAttributeName::NamespacedName(it) => walk_jsx_namespaced_name(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_jsx_attribute_value<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXAttributeValue<'a>,
) {
    // No `AstKind` for this type
    match it {
        JSXAttributeValue::StringLiteral(it) => walk_string_literal(visitor, it),
        JSXAttributeValue::ExpressionContainer(it) => walk_jsx_expression_container(visitor, it),
        JSXAttributeValue::Element(it) => walk_jsx_element(visitor, it),
        JSXAttributeValue::Fragment(it) => walk_jsx_fragment(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_jsx_identifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXIdentifier<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_jsx_child<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a JSXChild<'a>) {
    // No `AstKind` for this type
    match it {
        JSXChild::Text(it) => walk_jsx_text(visitor, it),
        JSXChild::Element(it) => walk_jsx_element(visitor, it),
        JSXChild::Fragment(it) => walk_jsx_fragment(visitor, it),
        JSXChild::ExpressionContainer(it) => walk_jsx_expression_container(visitor, it),
        JSXChild::Spread(it) => walk_jsx_spread_child(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_jsx_spread_child<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSXSpreadChild<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_jsx_text<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a JSXText<'a>) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_this_parameter<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSThisParameter<'a>,
) {
    walk_span(visitor, &it.span);
    walk_span(visitor, &it.this_span);
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type_annotation(visitor, type_annotation);
    }
}

#[inline]
pub(crate) fn walk_ts_enum_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSEnumDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_identifier(visitor, &it.id);
    walk_ts_enum_body(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_ts_enum_body<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSEnumBody<'a>) {
    walk_span(visitor, &it.span);
    walk_ts_enum_members(visitor, &it.members);
}

#[inline]
pub(crate) fn walk_ts_enum_member<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSEnumMember<'a>) {
    walk_span(visitor, &it.span);
    walk_ts_enum_member_name(visitor, &it.id);
    if let Some(initializer) = &it.initializer {
        walk_expression(visitor, initializer);
    }
}

#[inline]
pub(crate) fn walk_ts_enum_member_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSEnumMemberName<'a>,
) {
    // No `AstKind` for this type
    match it {
        TSEnumMemberName::Identifier(it) => walk_identifier_name(visitor, it),
        TSEnumMemberName::String(it) => walk_string_literal(visitor, it),
        TSEnumMemberName::ComputedString(it) => walk_string_literal(visitor, it),
        TSEnumMemberName::ComputedTemplateString(it) => walk_template_literal(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_ts_type_annotation<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeAnnotation<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_literal_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSLiteralType<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_literal(visitor, &it.literal);
}

pub(crate) fn walk_ts_literal<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSLiteral<'a>) {
    // No `AstKind` for this type
    match it {
        TSLiteral::BooleanLiteral(it) => walk_boolean_literal(visitor, it),
        TSLiteral::NumericLiteral(it) => walk_numeric_literal(visitor, it),
        TSLiteral::BigIntLiteral(it) => walk_big_int_literal(visitor, it),
        TSLiteral::StringLiteral(it) => walk_string_literal(visitor, it),
        TSLiteral::TemplateLiteral(it) => walk_template_literal(visitor, it),
        TSLiteral::UnaryExpression(it) => walk_unary_expression(visitor, it),
    }
}

pub(crate) fn walk_ts_type<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSType<'a>) {
    // No `AstKind` for this type
    match it {
        TSType::TSAnyKeyword(it) => walk_ts_any_keyword(visitor, it),
        TSType::TSBigIntKeyword(it) => walk_ts_big_int_keyword(visitor, it),
        TSType::TSBooleanKeyword(it) => walk_ts_boolean_keyword(visitor, it),
        TSType::TSIntrinsicKeyword(it) => walk_ts_intrinsic_keyword(visitor, it),
        TSType::TSNeverKeyword(it) => walk_ts_never_keyword(visitor, it),
        TSType::TSNullKeyword(it) => walk_ts_null_keyword(visitor, it),
        TSType::TSNumberKeyword(it) => walk_ts_number_keyword(visitor, it),
        TSType::TSObjectKeyword(it) => walk_ts_object_keyword(visitor, it),
        TSType::TSStringKeyword(it) => walk_ts_string_keyword(visitor, it),
        TSType::TSSymbolKeyword(it) => walk_ts_symbol_keyword(visitor, it),
        TSType::TSUndefinedKeyword(it) => walk_ts_undefined_keyword(visitor, it),
        TSType::TSUnknownKeyword(it) => walk_ts_unknown_keyword(visitor, it),
        TSType::TSVoidKeyword(it) => walk_ts_void_keyword(visitor, it),
        TSType::TSArrayType(it) => walk_ts_array_type(visitor, it),
        TSType::TSConditionalType(it) => walk_ts_conditional_type(visitor, it),
        TSType::TSConstructorType(it) => walk_ts_constructor_type(visitor, it),
        TSType::TSFunctionType(it) => walk_ts_function_type(visitor, it),
        TSType::TSImportType(it) => walk_ts_import_type(visitor, it),
        TSType::TSIndexedAccessType(it) => walk_ts_indexed_access_type(visitor, it),
        TSType::TSInferType(it) => walk_ts_infer_type(visitor, it),
        TSType::TSIntersectionType(it) => walk_ts_intersection_type(visitor, it),
        TSType::TSLiteralType(it) => walk_ts_literal_type(visitor, it),
        TSType::TSMappedType(it) => walk_ts_mapped_type(visitor, it),
        TSType::TSNamedTupleMember(it) => walk_ts_named_tuple_member(visitor, it),
        TSType::TSTemplateLiteralType(it) => walk_ts_template_literal_type(visitor, it),
        TSType::TSThisType(it) => walk_ts_this_type(visitor, it),
        TSType::TSTupleType(it) => walk_ts_tuple_type(visitor, it),
        TSType::TSTypeLiteral(it) => walk_ts_type_literal(visitor, it),
        TSType::TSTypeOperatorType(it) => walk_ts_type_operator(visitor, it),
        TSType::TSTypePredicate(it) => walk_ts_type_predicate(visitor, it),
        TSType::TSTypeQuery(it) => walk_ts_type_query(visitor, it),
        TSType::TSTypeReference(it) => walk_ts_type_reference(visitor, it),
        TSType::TSUnionType(it) => walk_ts_union_type(visitor, it),
        TSType::TSParenthesizedType(it) => walk_ts_parenthesized_type(visitor, it),
        TSType::JSDocNullableType(it) => walk_js_doc_nullable_type(visitor, it),
        TSType::JSDocNonNullableType(it) => walk_js_doc_non_nullable_type(visitor, it),
        TSType::JSDocUnknownType(it) => walk_js_doc_unknown_type(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_ts_conditional_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSConditionalType<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.check_type);
    walk_ts_type(visitor, &it.extends_type);
    walk_ts_type(visitor, &it.true_type);
    walk_ts_type(visitor, &it.false_type);
}

#[inline]
pub(crate) fn walk_ts_union_type<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSUnionType<'a>) {
    walk_span(visitor, &it.span);
    walk_ts_types(visitor, &it.types);
}

#[inline]
pub(crate) fn walk_ts_intersection_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSIntersectionType<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_types(visitor, &it.types);
}

#[inline]
pub(crate) fn walk_ts_parenthesized_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSParenthesizedType<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_type_operator<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeOperator<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_array_type<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSArrayType<'a>) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.element_type);
}

#[inline]
pub(crate) fn walk_ts_indexed_access_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSIndexedAccessType<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.object_type);
    walk_ts_type(visitor, &it.index_type);
}

#[inline]
pub(crate) fn walk_ts_tuple_type<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSTupleType<'a>) {
    walk_span(visitor, &it.span);
    walk_ts_tuple_elements(visitor, &it.element_types);
}

#[inline]
pub(crate) fn walk_ts_named_tuple_member<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSNamedTupleMember<'a>,
) {
    walk_span(visitor, &it.span);
    walk_identifier_name(visitor, &it.label);
    walk_ts_tuple_element(visitor, &it.element_type);
}

#[inline]
pub(crate) fn walk_ts_optional_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSOptionalType<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_rest_type<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSRestType<'a>) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_tuple_element<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTupleElement<'a>,
) {
    // No `AstKind` for this type
    match it {
        TSTupleElement::TSOptionalType(it) => walk_ts_optional_type(visitor, it),
        TSTupleElement::TSRestType(it) => walk_ts_rest_type(visitor, it),
        match_ts_type!(TSTupleElement) => walk_ts_type(visitor, it.to_ts_type()),
    }
}
