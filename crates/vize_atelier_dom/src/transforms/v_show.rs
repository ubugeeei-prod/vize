//! v-show transform for DOM elements.
//!
//! v-show toggles the element's display CSS property.

use vize_atelier_core::{DirectiveNode, RuntimeHelper};
use vize_carton::{cstr, String};

/// Runtime helper for v-show
pub const V_SHOW: RuntimeHelper = RuntimeHelper::WithDirectives;

/// Check if directive is v-show
pub fn is_v_show(dir: &DirectiveNode<'_>) -> bool {
    dir.name.as_str() == "show"
}

/// Generate v-show style expression
pub fn generate_show_style(dir: &DirectiveNode<'_>) -> String {
    if let Some(vize_atelier_core::ExpressionNode::Simple(simple)) = &dir.exp {
        return cstr!("display: ({}) ? '' : 'none'", simple.content);
    }
    String::from("display: ''")
}

/// Generate v-show directive registration for withDirectives
pub fn generate_show_directive(dir: &DirectiveNode<'_>) -> String {
    if let Some(vize_atelier_core::ExpressionNode::Simple(simple)) = &dir.exp {
        return cstr!("[vShow, {}]", simple.content);
    }
    String::from("[vShow, true]")
}

#[cfg(test)]
mod tests {
    use super::{generate_show_directive, generate_show_style, is_v_show, RuntimeHelper, V_SHOW};
    use vize_atelier_core::{DirectiveNode, ExpressionNode, SimpleExpressionNode, SourceLocation};
    use vize_carton::{Box, Bump};

    fn create_show_directive<'a>(allocator: &'a Bump, exp: &str) -> DirectiveNode<'a> {
        let mut dir = DirectiveNode::new(allocator, "show", SourceLocation::STUB);
        let exp_node = SimpleExpressionNode::new(exp, false, SourceLocation::STUB);
        let boxed = Box::new_in(exp_node, allocator);
        dir.exp = Some(ExpressionNode::Simple(boxed));
        dir
    }

    #[test]
    fn test_v_show_helper() {
        assert_eq!(V_SHOW, RuntimeHelper::WithDirectives);
    }

    #[test]
    fn test_is_v_show_true() {
        let allocator = Bump::new();
        let dir = create_show_directive(&allocator, "visible");
        assert!(is_v_show(&dir));
    }

    #[test]
    fn test_is_v_show_false() {
        let allocator = Bump::new();
        let dir = DirectiveNode::new(&allocator, "if", SourceLocation::STUB);
        assert!(!is_v_show(&dir));
    }

    #[test]
    fn test_generate_show_style() {
        let allocator = Bump::new();
        let dir = create_show_directive(&allocator, "visible");
        let style = generate_show_style(&dir);
        assert_eq!(style, "display: (visible) ? '' : 'none'");
    }

    #[test]
    fn test_generate_show_directive() {
        let allocator = Bump::new();
        let dir = create_show_directive(&allocator, "isActive");
        let result = generate_show_directive(&dir);
        assert_eq!(result, "[vShow, isActive]");
    }
}
