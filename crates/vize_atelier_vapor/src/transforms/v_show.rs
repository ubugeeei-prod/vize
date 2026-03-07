//! v-show transform for Vapor mode.
//!
//! Transforms v-show directive for toggling display.

use vize_carton::{cstr, Box, Bump, String};

use crate::ir::{DirectiveIRNode, OperationNode};
use vize_atelier_core::{DirectiveNode, ExpressionNode};

/// Transform v-show directive to IR
pub fn transform_v_show<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
    element_id: usize,
) -> OperationNode<'a> {
    // v-show is implemented as a directive that toggles display style
    let new_dir = DirectiveNode::new(allocator, "show", dir.loc.clone());

    let dir_ir = DirectiveIRNode {
        element: element_id,
        dir: Box::new_in(new_dir, allocator),
        name: String::new("show"),
        builtin: true,
        tag: String::new(""),
        input_type: String::new(""),
    };

    OperationNode::Directive(dir_ir)
}

/// Get v-show condition expression
pub fn get_show_condition(dir: &DirectiveNode<'_>) -> Option<String> {
    dir.exp.as_ref().map(|exp| match exp {
        ExpressionNode::Simple(s) => s.content.clone(),
        ExpressionNode::Compound(c) => c.loc.source.clone(),
    })
}

/// Check if v-show needs transition handling
pub fn needs_transition(_el: &vize_atelier_core::ElementNode<'_>) -> bool {
    // Check if element has transition wrapper
    false
}

/// Generate v-show effect code
pub fn generate_v_show_effect(element_var: &str, condition: &str) -> String {
    cstr!("{element_var}.style.display = {condition} ? '' : 'none'")
}

#[cfg(test)]
mod tests {
    use super::generate_v_show_effect;

    #[test]
    fn test_generate_v_show_effect() {
        let result = generate_v_show_effect("_n1", "isVisible");
        assert_eq!(result, "_n1.style.display = isVisible ? '' : 'none'");
    }
}
