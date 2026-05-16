//! v-bind transform for Vapor mode.
//!
//! Transforms v-bind (: shorthand) directives into SetPropIRNode.

use vize_carton::{camelize, Box, Bump, Vec};

use crate::ir::{IRProp, OperationNode, SetPropIRNode};
use vize_atelier_core::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, SimpleExpressionNode,
};

/// Transform v-bind directive to IR
pub fn transform_v_bind<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
) -> Option<OperationNode<'a>> {
    let key = extract_prop_key(allocator, dir)?;
    let values = extract_prop_values(allocator, dir);

    let set_prop = SetPropIRNode {
        element: element_id,
        prop: IRProp {
            key,
            values,
            is_component: el.tag_type == ElementType::Component,
        },
        tag: el.tag.clone(),
        camel: has_modifier(dir, "camel"),
        prop_modifier: has_modifier(dir, "prop"),
    };

    Some(OperationNode::SetProp(set_prop))
}

/// Transform v-bind without argument (v-bind="obj")
pub fn transform_v_bind_dynamic<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
    element_id: usize,
) -> Option<OperationNode<'a>> {
    // v-bind without argument requires merging props
    let mut props = Vec::new_in(allocator);

    if let Some(ref exp) = dir.exp {
        if let ExpressionNode::Simple(simple) = exp {
            let node = SimpleExpressionNode::new(
                simple.content.clone(),
                simple.is_static,
                simple.loc.clone(),
            );
            props.push(Box::new_in(node, allocator));
        }
    }

    Some(OperationNode::SetDynamicProps(
        crate::ir::SetDynamicPropsIRNode {
            element: element_id,
            props,
            is_event: false,
        },
    ))
}

/// Extract prop key from directive argument
fn extract_prop_key<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    dir.arg.as_ref().map(|arg| match arg {
        ExpressionNode::Simple(exp) => {
            // Apply camel modifier if present
            let content = if has_modifier(dir, "camel") {
                camelize(&exp.content)
            } else {
                exp.content.clone()
            };

            let node = SimpleExpressionNode::new(content, exp.is_static, exp.loc.clone());
            Box::new_in(node, allocator)
        }
        ExpressionNode::Compound(compound) => {
            let node =
                SimpleExpressionNode::new(compound.loc.source.clone(), false, compound.loc.clone());
            Box::new_in(node, allocator)
        }
    })
}

/// Extract prop values from directive expression
fn extract_prop_values<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
) -> Vec<'a, Box<'a, SimpleExpressionNode<'a>>> {
    let mut values = Vec::new_in(allocator);

    if let Some(ref exp) = dir.exp {
        match exp {
            ExpressionNode::Simple(simple) => {
                let node = SimpleExpressionNode::new(
                    simple.content.clone(),
                    simple.is_static,
                    simple.loc.clone(),
                );
                values.push(Box::new_in(node, allocator));
            }
            ExpressionNode::Compound(compound) => {
                let node = SimpleExpressionNode::new(
                    compound.loc.source.clone(),
                    false,
                    compound.loc.clone(),
                );
                values.push(Box::new_in(node, allocator));
            }
        }
    }

    values
}

/// Check if directive has a specific modifier
fn has_modifier(dir: &DirectiveNode<'_>, name: &str) -> bool {
    dir.modifiers.iter().any(|m| m.content == name)
}

/// Check if binding is dynamic (needs effect)
pub fn is_dynamic_binding(dir: &DirectiveNode<'_>) -> bool {
    // Check if argument is dynamic
    if let Some(ref arg) = dir.arg {
        match arg {
            ExpressionNode::Simple(simple) => {
                if !simple.is_static {
                    return true;
                }
            }
            ExpressionNode::Compound(_) => return true,
        }
    } else {
        // v-bind="obj" is always dynamic
        return true;
    }

    // Check if value is dynamic
    if let Some(ref exp) = dir.exp {
        match exp {
            ExpressionNode::Simple(simple) => !simple.is_static,
            ExpressionNode::Compound(_) => true,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use vize_carton::camelize;

    #[test]
    fn test_camelize() {
        assert_eq!(camelize("foo-bar").as_str(), "fooBar");
        assert_eq!(camelize("foo-bar-baz").as_str(), "fooBarBaz");
        assert_eq!(camelize("foo").as_str(), "foo");
    }
}
