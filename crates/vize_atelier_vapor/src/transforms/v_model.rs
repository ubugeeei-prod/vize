//! v-model transform for Vapor mode.
//!
//! Transforms v-model directives for two-way binding.

use vize_carton::{cstr, Box, Bump, String, ToCompactString};

use crate::ir::{DirectiveIRNode, OperationNode};
use vize_atelier_core::{DirectiveNode, ElementNode, ExpressionNode};

/// Transform v-model directive to IR
pub fn transform_v_model<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
    _el: &ElementNode<'a>,
    element_id: usize,
) -> Vec<OperationNode<'a>> {
    let mut operations = Vec::new();

    // v-model is syntactic sugar for :value + @input
    // For Vapor mode, we use the v-model directive directly

    // Create a copy of the directive for IR
    let new_dir = DirectiveNode::new(allocator, "model", dir.loc.clone());

    let dir_ir = DirectiveIRNode {
        element: element_id,
        dir: Box::new_in(new_dir, allocator),
        name: String::new("model"),
        builtin: true,
        tag: String::new(""),
        input_type: String::new(""),
    };

    operations.push(OperationNode::Directive(dir_ir));

    operations
}

/// Get v-model binding expression
pub fn get_model_value(dir: &DirectiveNode<'_>) -> Option<String> {
    dir.exp.as_ref().map(|exp| match exp {
        ExpressionNode::Simple(s) => s.content.clone(),
        ExpressionNode::Compound(c) => c.loc.source.clone(),
    })
}

/// Get v-model argument (for v-model:propName)
pub fn get_model_arg(dir: &DirectiveNode<'_>) -> String {
    dir.arg
        .as_ref()
        .map(|arg| match arg {
            ExpressionNode::Simple(s) => s.content.clone(),
            ExpressionNode::Compound(c) => c.loc.source.clone(),
        })
        .unwrap_or_else(|| String::new("modelValue"))
}

/// Get v-model modifiers
pub fn get_model_modifiers(dir: &DirectiveNode<'_>) -> Vec<String> {
    dir.modifiers.iter().map(|m| m.content.clone()).collect()
}

/// Check if v-model has .lazy modifier
pub fn has_lazy_modifier(dir: &DirectiveNode<'_>) -> bool {
    dir.modifiers.iter().any(|m| m.content == "lazy")
}

/// Check if v-model has .number modifier
pub fn has_number_modifier(dir: &DirectiveNode<'_>) -> bool {
    dir.modifiers.iter().any(|m| m.content == "number")
}

/// Check if v-model has .trim modifier
pub fn has_trim_modifier(dir: &DirectiveNode<'_>) -> bool {
    dir.modifiers.iter().any(|m| m.content == "trim")
}

/// Generate event name for v-model based on element type
pub fn get_model_event(el: &ElementNode<'_>) -> &'static str {
    match el.tag.as_str() {
        "input" => {
            // Check for type attribute to determine event
            "input"
        }
        "select" => "change",
        "textarea" => "input",
        _ => "update:modelValue",
    }
}

/// Generate v-model handler code
pub fn generate_model_handler(value_expr: &str, modifiers: &[String]) -> String {
    let mut event_value = "$event.target.value".to_compact_string();

    // Apply modifiers
    for modifier in modifiers {
        match modifier.as_str() {
            "number" => {
                event_value = cstr!("Number({event_value})");
            }
            "trim" => {
                event_value = cstr!("{event_value}.trim()");
            }
            _ => {}
        }
    }

    cstr!("$event => {{ {value_expr} = {event_value} }}")
}

#[cfg(test)]
mod tests {
    use super::generate_model_handler;
    use vize_carton::String;

    #[test]
    fn test_generate_model_handler_simple() {
        let result = generate_model_handler("text", &[]);
        assert!(result.contains("text = $event.target.value"));
    }

    #[test]
    fn test_generate_model_handler_with_trim() {
        let result = generate_model_handler("text", &[String::new("trim")]);
        assert!(result.contains(".trim()"));
    }

    #[test]
    fn test_generate_model_handler_with_number() {
        let result = generate_model_handler("num", &[String::new("number")]);
        assert!(result.contains("Number("));
    }
}
