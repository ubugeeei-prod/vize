//! Existing nested-class traversal, separated without behavior changes.
use super::{
    BindingType, ClosureScopeData, CompactString, ScopeBinding, ScriptParseResult,
    extract_function_params_with_occurrences, walk_statement,
};
use oxc_ast::ast::Class;

pub(super) fn walk_nested_class(result: &mut ScriptParseResult, class: &Class<'_>, source: &str) {
    // Add class name as binding
    if let Some(id) = &class.id {
        result.scopes.add_binding(
            CompactString::new(id.name.as_str()),
            ScopeBinding::new(BindingType::SetupConst, class.span.start),
        );
    }
    // Walk class body for methods
    for element in class.body.body.iter() {
        if let oxc_ast::ast::ClassElement::MethodDefinition(method) = element
            && let Some(body) = &method.value.body
        {
            let params = extract_function_params_with_occurrences(result, &method.value.params);
            result.scopes.enter_closure_scope(
                ClosureScopeData {
                    name: None,
                    param_names: params,
                    is_arrow: false,
                    is_async: method.value.r#async,
                    is_generator: method.value.generator,
                },
                method.span.start,
                method.span.end,
            );
            result.install_parameter_occurrences();
            for stmt in body.statements.iter() {
                walk_statement(result, stmt, source);
            }
            result.scopes.exit_scope();
        }
    }
}
