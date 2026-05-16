//! v-bind directive transform.
//!
//! Transforms v-bind (: shorthand) directives for dynamic props.

use vize_carton::String;

use crate::ast::*;
use crate::transform::TransformContext;

/// Transform v-bind directive - adds required helpers
pub fn process_v_bind(ctx: &mut TransformContext<'_>, dir: &DirectiveNode<'_>) {
    // Get prop name
    let prop_name = dir.arg.as_ref().map(|arg| match arg {
        ExpressionNode::Simple(exp) => exp.content.clone(),
        ExpressionNode::Compound(exp) => exp.loc.source.clone(),
    });

    // Handle v-bind without argument (v-bind="obj")
    if prop_name.is_none() {
        if dir.exp.is_some() {
            ctx.helper(RuntimeHelper::MergeProps);
        }
        return;
    }

    let prop_name = match prop_name {
        Some(name) => name,
        None => return,
    };

    // Handle special props
    match prop_name.as_str() {
        "class" => {
            ctx.helper(RuntimeHelper::NormalizeClass);
        }
        "style" => {
            ctx.helper(RuntimeHelper::NormalizeStyle);
        }
        "key" | "ref" => {
            // Special handling
        }
        _ => {}
    }
}

/// Get binding name from v-bind directive
pub fn get_bind_name(dir: &DirectiveNode<'_>) -> Option<String> {
    dir.arg.as_ref().map(|arg| match arg {
        ExpressionNode::Simple(exp) => exp.content.clone(),
        ExpressionNode::Compound(exp) => exp.loc.source.clone(),
    })
}

/// Get binding value expression
pub fn get_bind_value<'a>(dir: &'a DirectiveNode<'a>) -> Option<&'a ExpressionNode<'a>> {
    dir.exp.as_ref()
}

/// Check if binding has .camel modifier
pub fn has_camel_modifier(dir: &DirectiveNode<'_>) -> bool {
    dir.modifiers.iter().any(|m| m.content == "camel")
}

/// Check if binding has .prop modifier
pub fn has_prop_modifier(dir: &DirectiveNode<'_>) -> bool {
    dir.modifiers.iter().any(|m| m.content == "prop")
}

/// Check if binding has .attr modifier
pub fn has_attr_modifier(dir: &DirectiveNode<'_>) -> bool {
    dir.modifiers.iter().any(|m| m.content == "attr")
}

/// Check if binding is dynamic
pub fn is_dynamic_binding(dir: &DirectiveNode<'_>) -> bool {
    if let Some(arg) = &dir.arg {
        match arg {
            ExpressionNode::Simple(exp) => !exp.is_static,
            ExpressionNode::Compound(_) => true,
        }
    } else {
        true // v-bind="obj" is dynamic
    }
}

// Re-export camelize from vize_carton
pub use vize_carton::camelize;

#[cfg(test)]
mod tests {
    use super::camelize;

    #[test]
    fn test_camelize() {
        assert_eq!(camelize("foo-bar").as_str(), "fooBar");
        assert_eq!(camelize("foo-bar-baz").as_str(), "fooBarBaz");
        assert_eq!(camelize("foo").as_str(), "foo");
    }
}
