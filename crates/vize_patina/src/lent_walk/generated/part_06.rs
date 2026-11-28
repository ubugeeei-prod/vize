// Generated from OXC's pinned immutable walk; see mod.rs for provenance.
use super::*;

#[inline]
pub(crate) fn walk_ts_import_type_qualifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSImportTypeQualifier<'a>,
) {
    // No `AstKind` for this type
    match it {
        TSImportTypeQualifier::Identifier(it) => walk_identifier_name(visitor, it),
        TSImportTypeQualifier::QualifiedName(it) => walk_ts_import_type_qualified_name(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_ts_import_type_qualified_name<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSImportTypeQualifiedName<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_import_type_qualifier(visitor, &it.left);
    walk_identifier_name(visitor, &it.right);
}

#[inline]
pub(crate) fn walk_ts_function_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSFunctionType<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(type_parameters) = &it.type_parameters {
        walk_ts_type_parameter_declaration(visitor, type_parameters);
    }
    if let Some(this_param) = &it.this_param {
        walk_ts_this_parameter(visitor, this_param);
    }
    walk_formal_parameters(visitor, &it.params);
    walk_ts_type_annotation(visitor, &it.return_type);
}

#[inline]
pub(crate) fn walk_ts_constructor_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSConstructorType<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(type_parameters) = &it.type_parameters {
        walk_ts_type_parameter_declaration(visitor, type_parameters);
    }
    walk_formal_parameters(visitor, &it.params);
    walk_ts_type_annotation(visitor, &it.return_type);
}

#[inline]
pub(crate) fn walk_ts_mapped_type<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TSMappedType<'a>) {
    walk_span(visitor, &it.span);
    walk_binding_identifier(visitor, &it.key);
    walk_ts_type(visitor, &it.constraint);
    if let Some(name_type) = &it.name_type {
        walk_ts_type(visitor, name_type);
    }
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type(visitor, type_annotation);
    }
}

#[inline]
pub(crate) fn walk_ts_template_literal_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTemplateLiteralType<'a>,
) {
    walk_span(visitor, &it.span);
    walk_template_elements(visitor, &it.quasis);
    walk_ts_types(visitor, &it.types);
}

#[inline]
pub(crate) fn walk_ts_as_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSAsExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_satisfies_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSSatisfiesExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_ts_type_assertion<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSTypeAssertion<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.type_annotation);
    walk_expression(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_ts_import_equals_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSImportEqualsDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_identifier(visitor, &it.id);
    walk_ts_module_reference(visitor, &it.module_reference);
}

#[inline]
pub(crate) fn walk_ts_module_reference<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSModuleReference<'a>,
) {
    // No `AstKind` for this type
    match it {
        TSModuleReference::ExternalModuleReference(it) => {
            walk_ts_external_module_reference(visitor, it)
        }
        TSModuleReference::IdentifierReference(it) => walk_identifier_reference(visitor, it),
        TSModuleReference::QualifiedName(it) => walk_ts_qualified_name(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_ts_external_module_reference<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSExternalModuleReference<'a>,
) {
    walk_span(visitor, &it.span);
    walk_string_literal(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_ts_non_null_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSNonNullExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_decorator<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Decorator<'a>) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_ts_export_assignment<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSExportAssignment<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_ts_namespace_export_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSNamespaceExportDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_identifier_name(visitor, &it.id);
}

#[inline]
pub(crate) fn walk_ts_instantiation_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a TSInstantiationExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
    walk_ts_type_parameter_instantiation(visitor, &it.type_arguments);
}

#[inline]
pub(crate) fn walk_js_doc_nullable_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSDocNullableType<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_js_doc_non_nullable_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSDocNonNullableType<'a>,
) {
    walk_span(visitor, &it.span);
    walk_ts_type(visitor, &it.type_annotation);
}

#[inline]
pub(crate) fn walk_js_doc_unknown_type<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a JSDocUnknownType,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_span<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Span) {
    // No `AstKind` for this type
}

#[inline]
pub(crate) fn walk_directives<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, Directive<'a>>,
) {
    for el in it {
        walk_directive(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_statements<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, Statement<'a>>,
) {
    for el in it {
        walk_statement(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_array_expression_elements<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, ArrayExpressionElement<'a>>,
) {
    for el in it {
        walk_array_expression_element(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_object_property_kinds<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, ObjectPropertyKind<'a>>,
) {
    for el in it {
        walk_object_property_kind(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_template_elements<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, TemplateElement<'a>>,
) {
    for el in it {
        walk_template_element(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_expressions<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, Expression<'a>>,
) {
    for el in it {
        walk_expression(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_arguments<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, Argument<'a>>,
) {
    for el in it {
        match el {
            oxc_ast::ast::Argument::SpreadElement(spread) => {
                walk_spread_element(visitor, spread);
            }
            _ => {
                walk_expression(visitor, el.to_expression());
            }
        }
    }
}

#[inline]
pub(crate) fn walk_assignment_target_properties<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, AssignmentTargetProperty<'a>>,
) {
    for el in it {
        walk_assignment_target_property(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_variable_declarators<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, VariableDeclarator<'a>>,
) {
    for el in it {
        walk_variable_declarator(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_switch_cases<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, SwitchCase<'a>>,
) {
    for el in it {
        walk_switch_case(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_binding_properties<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, BindingProperty<'a>>,
) {
    for el in it {
        walk_binding_property(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_formal_parameter_list<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, FormalParameter<'a>>,
) {
    for el in it {
        walk_formal_parameter(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_decorators<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, Decorator<'a>>,
) {
    for el in it {
        walk_decorator(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_ts_class_implements_list<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, TSClassImplements<'a>>,
) {
    for el in it {
        walk_ts_class_implements(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_class_elements<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, ClassElement<'a>>,
) {
    for el in it {
        walk_class_element(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_import_declaration_specifiers<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, ImportDeclarationSpecifier<'a>>,
) {
    for el in it {
        walk_import_declaration_specifier(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_import_attributes<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, ImportAttribute<'a>>,
) {
    for el in it {
        walk_import_attribute(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_export_specifiers<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, ExportSpecifier<'a>>,
) {
    for el in it {
        walk_export_specifier(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_jsx_children<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, JSXChild<'a>>,
) {
    for el in it {
        walk_jsx_child(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_jsx_attribute_items<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, JSXAttributeItem<'a>>,
) {
    for el in it {
        walk_jsx_attribute_item(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_ts_enum_members<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, TSEnumMember<'a>>,
) {
    for el in it {
        walk_ts_enum_member(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_ts_types<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, TSType<'a>>,
) {
    for el in it {
        walk_ts_type(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_ts_tuple_elements<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, TSTupleElement<'a>>,
) {
    for el in it {
        walk_ts_tuple_element(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_ts_type_parameters<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, TSTypeParameter<'a>>,
) {
    for el in it {
        walk_ts_type_parameter(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_ts_interface_heritages<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, TSInterfaceHeritage<'a>>,
) {
    for el in it {
        walk_ts_interface_heritage(visitor, el);
    }
}
