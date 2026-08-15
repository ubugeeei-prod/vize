//! v-on transform for Vapor mode.
//!
//! Transforms v-on (@ shorthand) directives into SetEventIRNode.

use vize_carton::{Allocator, Box, String, ToCompactString, cstr};

use crate::ir::{EventModifiers, OperationNode, SetEventIRNode};
use vize_atelier_core::{DirectiveNode, ExpressionNode, SimpleExpressionNode};

/// Transform v-on directive to IR
pub fn transform_v_on<'a>(
    allocator: &'a Allocator,
    dir: &DirectiveNode<'a>,
    element_id: usize,
    source: &'a str,
) -> Option<OperationNode<'a>> {
    let key = extract_event_name(allocator, dir, source)?;
    let value = extract_handler(allocator, dir, source);
    let modifiers = parse_modifiers(allocator, dir);

    let set_event = SetEventIRNode {
        element: element_id,
        key,
        value,
        modifiers,
        delegate: should_delegate(dir),
        effect: is_dynamic_handler(dir),
    };

    Some(OperationNode::SetEvent(set_event))
}

/// Extract event name from directive argument
fn extract_event_name<'a>(
    allocator: &'a Allocator,
    dir: &DirectiveNode<'a>,
    source: &'a str,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    dir.arg.as_ref().map(|arg| match arg {
        ExpressionNode::Simple(exp) => {
            let node = SimpleExpressionNode::new(exp.content, exp.is_static, exp.loc.clone());
            Box::new_in(node, &allocator)
        }
        ExpressionNode::Compound(compound) => {
            let node = SimpleExpressionNode::new(
                compound.loc.span.slice(source),
                false,
                compound.loc.clone(),
            );
            Box::new_in(node, &allocator)
        }
    })
}

/// Extract handler expression
fn extract_handler<'a>(
    allocator: &'a Allocator,
    dir: &DirectiveNode<'a>,
    source: &'a str,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    dir.exp.as_ref().map(|exp| match exp {
        ExpressionNode::Simple(simple) => {
            let node =
                SimpleExpressionNode::new(simple.content, simple.is_static, simple.loc.clone());
            Box::new_in(node, &allocator)
        }
        ExpressionNode::Compound(compound) => {
            let node = SimpleExpressionNode::new(
                compound.loc.span.slice(source),
                false,
                compound.loc.clone(),
            );
            Box::new_in(node, &allocator)
        }
    })
}

/// Parse event modifiers
fn parse_modifiers<'a>(allocator: &'a Allocator, dir: &DirectiveNode<'a>) -> EventModifiers<'a> {
    let mut modifiers = EventModifiers::new(allocator);

    for modifier in dir.modifiers.iter() {
        match modifier.content {
            "capture" => modifiers.options.capture = true,
            "once" => modifiers.options.once = true,
            "passive" => modifiers.options.passive = true,
            "stop" | "prevent" | "self" | "exact" | "left" | "right" | "middle" => {
                modifiers.non_keys.push(modifier.content);
            }
            _ => {
                // Key modifiers
                modifiers.keys.push(modifier.content);
            }
        }
    }

    modifiers
}

/// Check if event should use delegation
fn should_delegate(_dir: &DirectiveNode<'_>) -> bool {
    // By default, use delegation for performance
    true
}

/// Check if handler is dynamic (needs effect)
fn is_dynamic_handler(dir: &DirectiveNode<'_>) -> bool {
    if let Some(ref exp) = dir.exp {
        match exp {
            ExpressionNode::Simple(simple) => !simple.is_static,
            ExpressionNode::Compound(_) => true,
        }
    } else {
        false
    }
}

/// Generate event handler code
pub fn generate_event_handler(
    _event_name: &str,
    handler: Option<&str>,
    modifiers: &EventModifiers<'_>,
) -> String {
    let handler_code = handler.unwrap_or("() => {}");

    if modifiers.non_keys.is_empty() && modifiers.keys.is_empty() {
        return handler_code.to_compact_string();
    }

    // Generate withModifiers/withKeys wrapper
    let mut result = handler_code.to_compact_string();

    if !modifiers.keys.is_empty() {
        let keys: Vec<&str> = modifiers.keys.iter().copied().collect();
        result = cstr!(
            "_withKeys({result}, [{}])",
            keys.iter()
                .map(|k| cstr!("\"{k}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !modifiers.non_keys.is_empty() {
        let mods: Vec<&str> = modifiers.non_keys.iter().copied().collect();
        result = cstr!(
            "_withModifiers({result}, [{}])",
            mods.iter()
                .map(|m| cstr!("\"{m}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::generate_event_handler;
    use crate::ir::EventModifiers;
    use vize_carton::Allocator;

    #[test]
    fn test_generate_event_handler_simple() {
        let allocator = Allocator::new();
        let modifiers = EventModifiers::new(&allocator);
        let result = generate_event_handler("click", Some("handleClick"), &modifiers);
        assert_eq!(result, "handleClick");
    }

    #[test]
    fn test_generate_event_handler_with_modifiers() {
        let allocator = Allocator::new();
        let mut modifiers = EventModifiers::new(&allocator);
        modifiers.non_keys.push("stop");

        let result = generate_event_handler("click", Some("handleClick"), &modifiers);
        insta::assert_snapshot!(result.as_str());
    }
}
