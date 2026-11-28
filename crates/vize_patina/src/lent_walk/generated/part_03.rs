// Generated from OXC's pinned immutable walk; see mod.rs for provenance.
use super::*;

#[inline]
pub(crate) fn walk_binding_property<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a BindingProperty<'a>,
) {
    walk_span(visitor, &it.span);
    walk_property_key(visitor, &it.key);
    walk_binding_pattern(visitor, &it.value);
}

#[inline]
pub(crate) fn walk_array_pattern<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a ArrayPattern<'a>) {
    walk_span(visitor, &it.span);
    for el in it.elements.iter().flatten() {
        walk_binding_pattern(visitor, el);
    }
    if let Some(rest) = &it.rest {
        walk_binding_rest_element(visitor, rest);
    }
}

#[inline]
pub(crate) fn walk_binding_rest_element<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a BindingRestElement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_pattern(visitor, &it.argument);
}

pub(crate) fn walk_function<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a Function<'a>,
    flags: ScopeFlags,
) {
    walk_span(visitor, &it.span);
    if let Some(id) = &it.id {
        walk_binding_identifier(visitor, id);
    }
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
    if let Some(body) = &it.body {
        walk_function_body(visitor, body);
    }
}

#[inline]
pub(crate) fn walk_formal_parameters<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a FormalParameters<'a>,
) {
    walk_span(visitor, &it.span);
    walk_formal_parameter_list(visitor, &it.items);
    if let Some(rest) = &it.rest {
        walk_formal_parameter_rest(visitor, rest);
    }
}

#[inline]
pub(crate) fn walk_formal_parameter<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a FormalParameter<'a>,
) {
    walk_span(visitor, &it.span);
    walk_decorators(visitor, &it.decorators);
    walk_binding_pattern(visitor, &it.pattern);
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type_annotation(visitor, type_annotation);
    }
    if let Some(initializer) = &it.initializer {
        walk_expression(visitor, initializer);
    }
}

#[inline]
pub(crate) fn walk_formal_parameter_rest<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a FormalParameterRest<'a>,
) {
    walk_span(visitor, &it.span);
    walk_decorators(visitor, &it.decorators);
    walk_binding_rest_element(visitor, &it.rest);
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type_annotation(visitor, type_annotation);
    }
}

#[inline]
pub(crate) fn walk_function_body<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a FunctionBody<'a>) {
    walk_span(visitor, &it.span);
    walk_directives(visitor, &it.directives);
    walk_statements(visitor, &it.statements);
}

#[inline]
pub(crate) fn walk_arrow_function_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArrowFunctionExpression<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(type_parameters) = &it.type_parameters {
        walk_ts_type_parameter_declaration(visitor, type_parameters);
    }
    walk_formal_parameters(visitor, &it.params);
    if let Some(return_type) = &it.return_type {
        walk_ts_type_annotation(visitor, return_type);
    }
    walk_function_body(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_yield_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a YieldExpression<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(argument) = &it.argument {
        walk_expression(visitor, argument);
    }
}

pub(crate) fn walk_class<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Class<'a>) {
    walk_span(visitor, &it.span);
    walk_decorators(visitor, &it.decorators);
    if let Some(id) = &it.id {
        walk_binding_identifier(visitor, id);
    }
    if let Some(type_parameters) = &it.type_parameters {
        walk_ts_type_parameter_declaration(visitor, type_parameters);
    }
    if let Some(super_class) = &it.super_class {
        walk_expression(visitor, super_class);
    }
    if let Some(super_type_arguments) = &it.super_type_arguments {
        walk_ts_type_parameter_instantiation(visitor, super_type_arguments);
    }
    walk_ts_class_implements_list(visitor, &it.implements);
    walk_class_body(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_class_body<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a ClassBody<'a>) {
    walk_span(visitor, &it.span);
    walk_class_elements(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_class_element<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a ClassElement<'a>) {
    // No `AstKind` for this type
    match it {
        ClassElement::StaticBlock(it) => walk_static_block(visitor, it),
        ClassElement::MethodDefinition(it) => walk_method_definition(visitor, it),
        ClassElement::PropertyDefinition(it) => walk_property_definition(visitor, it),
        ClassElement::AccessorProperty(it) => walk_accessor_property(visitor, it),
        ClassElement::TSIndexSignature(it) => walk_ts_index_signature(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_method_definition<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a MethodDefinition<'a>,
) {
    walk_span(visitor, &it.span);
    walk_decorators(visitor, &it.decorators);
    walk_property_key(visitor, &it.key);
    {
        let flags = match it.kind {
            MethodDefinitionKind::Get => ScopeFlags::Function | ScopeFlags::GetAccessor,
            MethodDefinitionKind::Set => ScopeFlags::Function | ScopeFlags::SetAccessor,
            MethodDefinitionKind::Constructor => ScopeFlags::Function | ScopeFlags::Constructor,
            MethodDefinitionKind::Method => ScopeFlags::Function,
        };
        walk_function(visitor, &it.value, flags);
    }
}

#[inline]
pub(crate) fn walk_property_definition<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a PropertyDefinition<'a>,
) {
    walk_span(visitor, &it.span);
    walk_decorators(visitor, &it.decorators);
    walk_property_key(visitor, &it.key);
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type_annotation(visitor, type_annotation);
    }
    if let Some(value) = &it.value {
        walk_expression(visitor, value);
    }
}

#[inline]
pub(crate) fn walk_private_identifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a PrivateIdentifier<'a>,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_static_block<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a StaticBlock<'a>) {
    walk_span(visitor, &it.span);
    walk_statements(visitor, &it.body);
}

pub(crate) fn walk_module_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ModuleDeclaration<'a>,
) {
    // No `AstKind` for this type
    match it {
        ModuleDeclaration::ImportDeclaration(it) => walk_import_declaration(visitor, it),
        ModuleDeclaration::ExportAllDeclaration(it) => walk_export_all_declaration(visitor, it),
        ModuleDeclaration::ExportDefaultDeclaration(it) => {
            walk_export_default_declaration(visitor, it)
        }
        ModuleDeclaration::ExportNamedDeclaration(it) => walk_export_named_declaration(visitor, it),
        ModuleDeclaration::TSExportAssignment(it) => walk_ts_export_assignment(visitor, it),
        ModuleDeclaration::TSNamespaceExportDeclaration(it) => {
            walk_ts_namespace_export_declaration(visitor, it)
        }
    }
}

#[inline]
pub(crate) fn walk_accessor_property<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AccessorProperty<'a>,
) {
    walk_span(visitor, &it.span);
    walk_decorators(visitor, &it.decorators);
    walk_property_key(visitor, &it.key);
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type_annotation(visitor, type_annotation);
    }
    if let Some(value) = &it.value {
        walk_expression(visitor, value);
    }
}

#[inline]
pub(crate) fn walk_import_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ImportExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.source);
    if let Some(options) = &it.options {
        walk_expression(visitor, options);
    }
}

#[inline]
pub(crate) fn walk_import_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ImportDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(specifiers) = &it.specifiers {
        walk_import_declaration_specifiers(visitor, specifiers);
    }
    walk_string_literal(visitor, &it.source);
    if let Some(with_clause) = &it.with_clause {
        walk_with_clause(visitor, with_clause);
    }
}

#[inline]
pub(crate) fn walk_import_declaration_specifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ImportDeclarationSpecifier<'a>,
) {
    // No `AstKind` for this type
    match it {
        ImportDeclarationSpecifier::ImportSpecifier(it) => walk_import_specifier(visitor, it),
        ImportDeclarationSpecifier::ImportDefaultSpecifier(it) => {
            walk_import_default_specifier(visitor, it)
        }
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(it) => {
            walk_import_namespace_specifier(visitor, it)
        }
    }
}

#[inline]
pub(crate) fn walk_import_specifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ImportSpecifier<'a>,
) {
    walk_span(visitor, &it.span);
    walk_module_export_name(visitor, &it.imported);
    walk_binding_identifier(visitor, &it.local);
}

#[inline]
pub(crate) fn walk_import_default_specifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ImportDefaultSpecifier<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_identifier(visitor, &it.local);
}

#[inline]
pub(crate) fn walk_import_namespace_specifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ImportNamespaceSpecifier<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_identifier(visitor, &it.local);
}

#[inline]
pub(crate) fn walk_with_clause<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a WithClause<'a>) {
    walk_span(visitor, &it.span);
    walk_import_attributes(visitor, &it.with_entries);
}

#[inline]
pub(crate) fn walk_import_attribute<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ImportAttribute<'a>,
) {
    walk_span(visitor, &it.span);
    walk_import_attribute_key(visitor, &it.key);
    walk_string_literal(visitor, &it.value);
}

#[inline]
pub(crate) fn walk_import_attribute_key<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ImportAttributeKey<'a>,
) {
    // No `AstKind` for this type
    match it {
        ImportAttributeKey::Identifier(it) => walk_identifier_name(visitor, it),
        ImportAttributeKey::StringLiteral(it) => walk_string_literal(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_export_named_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ExportNamedDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(declaration) = &it.declaration {
        walk_declaration(visitor, declaration);
    }
    walk_export_specifiers(visitor, &it.specifiers);
    if let Some(source) = &it.source {
        walk_string_literal(visitor, source);
    }
    if let Some(with_clause) = &it.with_clause {
        walk_with_clause(visitor, with_clause);
    }
}

#[inline]
pub(crate) fn walk_export_default_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ExportDefaultDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_export_default_declaration_kind(visitor, &it.declaration);
}

#[inline]
pub(crate) fn walk_export_all_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ExportAllDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(exported) = &it.exported {
        walk_module_export_name(visitor, exported);
    }
    walk_string_literal(visitor, &it.source);
    if let Some(with_clause) = &it.with_clause {
        walk_with_clause(visitor, with_clause);
    }
}

#[inline]
pub(crate) fn walk_export_specifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ExportSpecifier<'a>,
) {
    walk_span(visitor, &it.span);
    walk_module_export_name(visitor, &it.local);
    walk_module_export_name(visitor, &it.exported);
}

#[inline]
pub(crate) fn walk_export_default_declaration_kind<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ExportDefaultDeclarationKind<'a>,
) {
    // No `AstKind` for this type
    match it {
        ExportDefaultDeclarationKind::FunctionDeclaration(it) => {
            let flags = ScopeFlags::Function;
            walk_function(visitor, it, flags)
        }
        ExportDefaultDeclarationKind::ClassDeclaration(it) => walk_class(visitor, it),
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(it) => {
            walk_ts_interface_declaration(visitor, it)
        }
        match_expression!(ExportDefaultDeclarationKind) => {
            walk_expression(visitor, it.to_expression())
        }
    }
}

#[inline]
pub(crate) fn walk_module_export_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ModuleExportName<'a>,
) {
    // No `AstKind` for this type
    match it {
        ModuleExportName::IdentifierName(it) => walk_identifier_name(visitor, it),
        ModuleExportName::IdentifierReference(it) => walk_identifier_reference(visitor, it),
        ModuleExportName::StringLiteral(it) => walk_string_literal(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_v8_intrinsic_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a V8IntrinsicExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_identifier_name(visitor, &it.name);
    walk_arguments(visitor, &it.arguments);
}

#[inline]
pub(crate) fn walk_boolean_literal<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a BooleanLiteral) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_null_literal<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a NullLiteral) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_numeric_literal<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a NumericLiteral<'a>,
) {
    walk_span(visitor, &it.span);
}
