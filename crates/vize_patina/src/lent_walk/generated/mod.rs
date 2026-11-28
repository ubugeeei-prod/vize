//! Private lifetime-carrying immutable AST traversal.
//!
//! Adapted from OXC fc702c1fa9f0412d06ec6908b58cd395b826cf7f, `crates/oxc_ast_visit/src/generated/visit.rs`.
//! Original source SHA-256: `5440cd4c4a2e10effd6b9eafc8d7a283258884b91eae1feef76988ca6760490e`.
//! The walk order and AST matches come from that pinned source. Only the
//! three LentNodes hooks are observable; OXC's other hooks were no-ops for
//! the former Lending visitor. Every node input now borrows for `'a`, so
//! rustc proves that the hooks may retain it for the root's borrow.
//!
//! OXC copyright (c) 2024-present VoidZero Inc. & Contributors; (c) 2023
//! Boshen. The complete MIT license is in the adjacent LICENSE file.

mod part_01;
pub(super) use part_01::{
    walk_array_expression_element, walk_assignment_target, walk_binding_identifier,
    walk_call_expression, walk_expression, walk_identifier_name, walk_identifier_reference,
    walk_label_identifier, walk_member_expression, walk_object_expression,
    walk_object_property_kind, walk_program, walk_property_key, walk_spread_element,
    walk_template_element, walk_template_literal, walk_this_expression, walk_unary_expression,
};
mod part_02;
pub(super) use part_02::{
    walk_assignment_target_maybe_default, walk_assignment_target_property, walk_await_expression,
    walk_binding_pattern, walk_chain_expression, walk_declaration, walk_directive, walk_hashbang,
    walk_parenthesized_expression, walk_sequence_expression, walk_statement, walk_super,
    walk_switch_case, walk_variable_declarator,
};
mod part_03;
pub(super) use part_03::{
    walk_array_pattern, walk_arrow_function_expression, walk_binding_property,
    walk_binding_rest_element, walk_boolean_literal, walk_class, walk_class_element,
    walk_export_specifier, walk_formal_parameter, walk_formal_parameters, walk_function,
    walk_import_attribute, walk_import_declaration_specifier, walk_import_expression,
    walk_module_declaration, walk_null_literal, walk_numeric_literal, walk_private_identifier,
    walk_v8_intrinsic_expression, walk_yield_expression,
};
mod part_04;
pub(super) use part_04::{
    walk_big_int_literal, walk_jsx_attribute_item, walk_jsx_child, walk_jsx_element,
    walk_jsx_expression_container, walk_jsx_fragment, walk_reg_exp_literal, walk_string_literal,
    walk_ts_enum_declaration, walk_ts_enum_member, walk_ts_this_parameter, walk_ts_tuple_element,
    walk_ts_type, walk_ts_type_annotation,
};
mod part_05;
pub(super) use part_05::{
    walk_ts_any_keyword, walk_ts_big_int_keyword, walk_ts_boolean_keyword,
    walk_ts_class_implements, walk_ts_global_declaration, walk_ts_import_type,
    walk_ts_import_type_qualifier, walk_ts_index_signature, walk_ts_index_signature_name,
    walk_ts_infer_type, walk_ts_interface_declaration, walk_ts_interface_heritage,
    walk_ts_intrinsic_keyword, walk_ts_module_declaration, walk_ts_never_keyword,
    walk_ts_null_keyword, walk_ts_number_keyword, walk_ts_object_keyword, walk_ts_qualified_name,
    walk_ts_signature, walk_ts_string_keyword, walk_ts_symbol_keyword, walk_ts_this_type,
    walk_ts_type_alias_declaration, walk_ts_type_literal, walk_ts_type_parameter,
    walk_ts_type_parameter_declaration, walk_ts_type_parameter_instantiation,
    walk_ts_type_predicate, walk_ts_type_query, walk_ts_type_reference, walk_ts_undefined_keyword,
    walk_ts_unknown_keyword, walk_ts_void_keyword,
};
mod part_06;
pub(super) use part_06::{
    walk_arguments, walk_array_expression_elements, walk_assignment_target_properties,
    walk_binding_properties, walk_class_elements, walk_decorators, walk_directives,
    walk_export_specifiers, walk_expressions, walk_formal_parameter_list, walk_import_attributes,
    walk_import_declaration_specifiers, walk_js_doc_non_nullable_type, walk_js_doc_nullable_type,
    walk_js_doc_unknown_type, walk_jsx_attribute_items, walk_jsx_children,
    walk_object_property_kinds, walk_span, walk_statements, walk_switch_cases,
    walk_template_elements, walk_ts_as_expression, walk_ts_class_implements_list,
    walk_ts_constructor_type, walk_ts_enum_members, walk_ts_export_assignment,
    walk_ts_function_type, walk_ts_import_equals_declaration, walk_ts_import_type_qualified_name,
    walk_ts_instantiation_expression, walk_ts_interface_heritages, walk_ts_mapped_type,
    walk_ts_namespace_export_declaration, walk_ts_non_null_expression,
    walk_ts_satisfies_expression, walk_ts_signatures, walk_ts_template_literal_type,
    walk_ts_tuple_elements, walk_ts_type_assertion, walk_ts_type_parameters, walk_ts_types,
    walk_variable_declarators,
};
mod part_07;
pub(super) use part_07::walk_ts_index_signature_names;
