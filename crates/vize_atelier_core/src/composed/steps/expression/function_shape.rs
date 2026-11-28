// DO NOT EDIT. Generated from crates/vize_atelier_core/src/steps/expression/function_shape.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
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
