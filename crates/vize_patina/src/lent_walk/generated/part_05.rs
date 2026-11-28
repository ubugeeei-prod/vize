// Generated from OXC's pinned immutable walk; see mod.rs for provenance.
use super::*;

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

#[inline]
pub(crate) fn walk_ts_any_keyword<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSAnyKeyword) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_string_keyword<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSStringKeyword,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_boolean_keyword<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSBooleanKeyword,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_number_keyword<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSNumberKeyword,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_never_keyword<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSNeverKeyword) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_intrinsic_keyword<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSIntrinsicKeyword,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_unknown_keyword<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSUnknownKeyword,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_null_keyword<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSNullKeyword) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_undefined_keyword<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSUndefinedKeyword,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_void_keyword<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSVoidKeyword) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_symbol_keyword<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSSymbolKeyword,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_this_type<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSThisType) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_object_keyword<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSObjectKeyword,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_big_int_keyword<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSBigIntKeyword,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_ts_type_reference<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeReference<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type_name(visitor, &it.type_name);
    if let Some(type_arguments) = &it.type_arguments {
        walk_ts_type_parameter_instantiation(visitor, type_arguments);
    }
}

#[inline]
pub(crate) fn walk_ts_type_name<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSTypeName<'a>) {
    // No `AstKind` for this type
    match it {
        TSTypeName::IdentifierReference(it) => walk_identifier_reference(visitor, it),
        TSTypeName::QualifiedName(it) => walk_ts_qualified_name(visitor, it),
        TSTypeName::ThisExpression(it) => walk_this_expression(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_ts_qualified_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSQualifiedName<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type_name(visitor, &it.left);
    walk_identifier_name(visitor, &it.right);
}

#[inline]
pub(crate) fn walk_ts_type_parameter_instantiation<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeParameterInstantiation<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_types(visitor, &it.params);
}

#[inline]
pub(crate) fn walk_ts_type_parameter<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeParameter<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_identifier(visitor, &it.name);
    if let Some(constraint) = &it.constraint {
        walk_ts_type(visitor, constraint);
    }
    if let Some(default) = &it.default {
        walk_ts_type(visitor, default);
    }
}

#[inline]
pub(crate) fn walk_ts_type_parameter_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeParameterDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type_parameters(visitor, &it.params);
}

#[inline]
pub(crate) fn walk_ts_type_alias_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeAliasDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_identifier(visitor, &it.id);
    if let Some(type_parameters) = &it.type_parameters {
        walk_ts_type_parameter_declaration(visitor, type_parameters);
    }
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_class_implements<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSClassImplements<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type_name(visitor, &it.expression);
    if let Some(type_arguments) = &it.type_arguments {
        walk_ts_type_parameter_instantiation(visitor, type_arguments);
    }
}

#[inline]
pub(crate) fn walk_ts_interface_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSInterfaceDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_identifier(visitor, &it.id);
    if let Some(type_parameters) = &it.type_parameters {
        walk_ts_type_parameter_declaration(visitor, type_parameters);
    }
    walk_ts_interface_heritages(visitor, &it.extends);
    walk_ts_interface_body(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_ts_interface_body<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSInterfaceBody<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_signatures(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_ts_property_signature<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSPropertySignature<'a>,
) {
    walk_span(visitor, &it.span);
    walk_property_key(visitor, &it.key);
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type_annotation(visitor, type_annotation);
    }
}

#[inline]
pub(crate) fn walk_ts_signature<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSSignature<'a>) {
    // No `AstKind` for this type
    match it {
        TSSignature::TSIndexSignature(it) => walk_ts_index_signature(visitor, it),
        TSSignature::TSPropertySignature(it) => walk_ts_property_signature(visitor, it),
        TSSignature::TSCallSignatureDeclaration(it) => {
            walk_ts_call_signature_declaration(visitor, it)
        }
        TSSignature::TSConstructSignatureDeclaration(it) => {
            walk_ts_construct_signature_declaration(visitor, it)
        }
        TSSignature::TSMethodSignature(it) => walk_ts_method_signature(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_ts_index_signature<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSIndexSignature<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_index_signature_names(visitor, &it.parameters);
    walk_ts_type_annotation(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_call_signature_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSCallSignatureDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(type_parameters) = &it.type_parameters {
        walk_ts_type_parameter_declaration(visitor, type_parameters);
    }
    if let Some(this_param) = &it.this_param {
        walk_ts_this_parameter(visitor, this_param);
    }
    walk_formal_parameters(visitor, &it.params);
    if let Some(return_type) = &it.return_type {
        walk_ts_type_annotation(visitor, return_type);
    }
}

pub(crate) fn walk_ts_method_signature<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSMethodSignature<'a>,
) {
    walk_span(visitor, &it.span);
    walk_property_key(visitor, &it.key);
    if let Some(type_parameters) = &it.type_parameters {
        walk_ts_type_parameter_declaration(visitor, type_parameters);
    }
    if let Some(this_param) = &it.this_param {
        walk_ts_this_parameter(visitor, this_param);
    }
    walk_formal_parameters(visitor, &it.params);
    if let Some(return_type) = &it.return_type {
        walk_ts_type_annotation(visitor, return_type);
    }
}

#[inline]
pub(crate) fn walk_ts_construct_signature_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSConstructSignatureDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(type_parameters) = &it.type_parameters {
        walk_ts_type_parameter_declaration(visitor, type_parameters);
    }
    walk_formal_parameters(visitor, &it.params);
    if let Some(return_type) = &it.return_type {
        walk_ts_type_annotation(visitor, return_type);
    }
}

#[inline]
pub(crate) fn walk_ts_index_signature_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSIndexSignatureName<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type_annotation(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_interface_heritage<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSInterfaceHeritage<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
    if let Some(type_arguments) = &it.type_arguments {
        walk_ts_type_parameter_instantiation(visitor, type_arguments);
    }
}

#[inline]
pub(crate) fn walk_ts_type_predicate<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypePredicate<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type_predicate_name(visitor, &it.parameter_name);
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type_annotation(visitor, type_annotation);
    }
}

#[inline]
pub(crate) fn walk_ts_type_predicate_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypePredicateName<'a>,
) {
    // No `AstKind` for this type
    match it {
        TSTypePredicateName::Identifier(it) => walk_identifier_name(visitor, it),
        TSTypePredicateName::This(it) => walk_ts_this_type(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_ts_module_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSModuleDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_module_declaration_name(visitor, &it.id);
    if let Some(body) = &it.body {
        walk_ts_module_declaration_body(visitor, body);
    }
}

#[inline]
pub(crate) fn walk_ts_module_declaration_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSModuleDeclarationName<'a>,
) {
    // No `AstKind` for this type
    match it {
        TSModuleDeclarationName::Identifier(it) => walk_binding_identifier(visitor, it),
        TSModuleDeclarationName::StringLiteral(it) => walk_string_literal(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_ts_module_declaration_body<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSModuleDeclarationBody<'a>,
) {
    // No `AstKind` for this type
    match it {
        TSModuleDeclarationBody::TSModuleDeclaration(it) => walk_ts_module_declaration(visitor, it),
        TSModuleDeclarationBody::TSModuleBlock(it) => walk_ts_module_block(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_ts_global_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSGlobalDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_span(visitor, &it.global_span);
    walk_ts_module_block(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_ts_module_block<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSModuleBlock<'a>,
) {
    walk_span(visitor, &it.span);
    walk_directives(visitor, &it.directives);
    walk_statements(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_ts_type_literal<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeLiteral<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_signatures(visitor, &it.members);
}

#[inline]
pub(crate) fn walk_ts_infer_type<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSInferType<'a>) {
    walk_span(visitor, &it.span);
    walk_ts_type_parameter(visitor, &it.type_parameter);
}

#[inline]
pub(crate) fn walk_ts_type_query<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSTypeQuery<'a>) {
    walk_span(visitor, &it.span);
    walk_ts_type_query_expr_name(visitor, &it.expr_name);
    if let Some(type_arguments) = &it.type_arguments {
        walk_ts_type_parameter_instantiation(visitor, type_arguments);
    }
}

#[inline]
pub(crate) fn walk_ts_type_query_expr_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeQueryExprName<'a>,
) {
    // No `AstKind` for this type
    match it {
        TSTypeQueryExprName::TSImportType(it) => walk_ts_import_type(visitor, it),
        match_ts_type_name!(TSTypeQueryExprName) => {
            walk_ts_type_name(visitor, it.to_ts_type_name())
        }
    }
}

#[inline]
pub(crate) fn walk_ts_import_type<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSImportType<'a>) {
    walk_span(visitor, &it.span);
    walk_string_literal(visitor, &it.source);
    if let Some(options) = &it.options {
        walk_object_expression(visitor, options);
    }
    if let Some(qualifier) = &it.qualifier {
        walk_ts_import_type_qualifier(visitor, qualifier);
    }
    if let Some(type_arguments) = &it.type_arguments {
        walk_ts_type_parameter_instantiation(visitor, type_arguments);
    }
}
