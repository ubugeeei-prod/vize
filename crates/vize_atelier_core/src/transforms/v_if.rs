//! v-if directive transform.
//!
//! Transforms elements with v-if, v-else-if, and v-else directives into IfNode.

use crate::ast::*;
use crate::transform::TransformContext;

/// Check if an element has a v-if directive
pub fn has_v_if(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "if"))
}

/// Check if an element has a v-else-if directive
pub fn has_v_else_if(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "else-if"))
}

/// Check if an element has a v-else directive
pub fn has_v_else(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "else"))
}

/// Get the v-if/v-else-if expression from an element
pub fn get_if_condition<'a>(el: &'a ElementNode<'a>) -> Option<&'a ExpressionNode<'a>> {
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            if dir.name == "if" || dir.name == "else-if" {
                return dir.exp.as_ref();
            }
        }
    }
    None
}

/// Remove v-if/v-else-if/v-else directive from element props
pub fn remove_if_directive(el: &mut ElementNode<'_>) {
    let mut i = 0;
    while i < el.props.len() {
        if let PropNode::Directive(dir) = &el.props[i] {
            if dir.name == "if" || dir.name == "else-if" || dir.name == "else" {
                el.props.remove(i);
                return;
            }
        }
        i += 1;
    }
}

/// Process v-if structural directive - adds helpers
pub fn process_v_if(ctx: &mut TransformContext<'_>) {
    ctx.helper(RuntimeHelper::OpenBlock);
    ctx.helper(RuntimeHelper::CreateBlock);
    ctx.helper(RuntimeHelper::CreateElementBlock);
    ctx.helper(RuntimeHelper::Fragment);
    ctx.helper(RuntimeHelper::CreateComment);
}

#[cfg(test)]
mod tests {
    use super::{has_v_else, has_v_else_if, has_v_if, TemplateChildNode};
    use crate::parser::parse;
    use bumpalo::Bump;

    #[test]
    fn test_has_v_if() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, r#"<div v-if="show">test</div>"#);

        if let TemplateChildNode::Element(el) = &root.children[0] {
            assert!(has_v_if(el));
            assert!(!has_v_else(el));
            assert!(!has_v_else_if(el));
        }
    }

    #[test]
    fn test_has_v_else() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, r#"<div v-else>test</div>"#);

        if let TemplateChildNode::Element(el) = &root.children[0] {
            assert!(has_v_else(el));
            assert!(!has_v_if(el));
        }
    }
}
