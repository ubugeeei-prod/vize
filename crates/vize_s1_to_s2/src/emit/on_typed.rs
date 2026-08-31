//! TypeScript-specific `v-on` handler detection.

use oxc_ast::ast::Expression;

pub(super) fn is_typed_arrow(expr: &Expression<'_>) -> bool {
    let Expression::ArrowFunctionExpression(arrow) = expr else {
        return false;
    };
    arrow.type_parameters.is_some()
        || arrow.return_type.is_some()
        || arrow
            .params
            .items
            .iter()
            .any(|param| param.type_annotation.is_some())
        || arrow
            .params
            .rest
            .as_ref()
            .is_some_and(|rest| rest.type_annotation.is_some())
}
