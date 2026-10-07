use oxc_span::SourceType;

use super::{shape_checks::is_function_shape, with_whole_expression};

/// Classify a complete callback in a TypeScript-capable template handler.
pub fn is_typescript_function_expression(content: &str) -> bool {
    with_whole_expression(
        content,
        SourceType::ts().with_module(true),
        is_function_shape,
    )
    .unwrap_or(false)
}
