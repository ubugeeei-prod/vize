//! v-on transform for Vapor mode.
//!
//! Transforms v-on (@ shorthand) directives into SetEventIRNode.

use vize_carton::{cstr, Box, Bump, String, ToCompactString};

use crate::ir::{EventModifiers, EventOptions, OperationNode, SetEventIRNode};
use vize_atelier_core::{DirectiveNode, ExpressionNode, SimpleExpressionNode};

/// Transform v-on directive to IR
pub fn transform_v_on<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
    element_id: usize,
) -> Option<OperationNode<'a>> {
    let key = extract_event_name(allocator, dir)?;
    let value = extract_handler(allocator, dir);
    let modifiers = parse_modifiers(dir);

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
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    dir.arg.as_ref().map(|arg| match arg {
        ExpressionNode::Simple(exp) => {
            let node =
                SimpleExpressionNode::new(exp.content.clone(), exp.is_static, exp.loc.clone());
            Box::new_in(node, allocator)
        }
        ExpressionNode::Compound(compound) => {
            let node =
                SimpleExpressionNode::new(compound.loc.source.clone(), false, compound.loc.clone());
            Box::new_in(node, allocator)
        }
    })
}

/// Extract handler expression
fn extract_handler<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    dir.exp.as_ref().map(|exp| match exp {
        ExpressionNode::Simple(simple) => {
            let node = SimpleExpressionNode::new(
                simple.content.clone(),
                simple.is_static,
                simple.loc.clone(),
            );
            Box::new_in(node, allocator)
        }
        ExpressionNode::Compound(compound) => {
            let node =
                SimpleExpressionNode::new(compound.loc.source.clone(), false, compound.loc.clone());
            Box::new_in(node, allocator)
        }
    })
}

/// Parse event modifiers
fn parse_modifiers(dir: &DirectiveNode<'_>) -> EventModifiers {
    let mut keys = Vec::new();
    let mut non_keys = Vec::new();
    let mut options = EventOptions::default();

    for modifier in dir.modifiers.iter() {
        match modifier.content.as_str() {
            "capture" => options.capture = true,
            "once" => options.once = true,
            "passive" => options.passive = true,
            "stop" | "prevent" | "self" | "exact" | "left" | "right" | "middle" => {
                non_keys.push(modifier.content.clone());
            }
            _ => {
                // Key modifiers
                keys.push(modifier.content.clone());
            }
        }
    }

    EventModifiers {
        keys,
        non_keys,
        options,
    }
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
    modifiers: &EventModifiers,
) -> String {
    let handler_code = handler.unwrap_or("() => {}");

    if modifiers.non_keys.is_empty() && modifiers.keys.is_empty() {
        return handler_code.to_compact_string();
    }

    // Generate withModifiers/withKeys wrapper
    let mut result = handler_code.to_compact_string();

    if !modifiers.keys.is_empty() {
        let keys: Vec<&str> = modifiers.keys.iter().map(|k| k.as_str()).collect();
        result = cstr!(
            "_withKeys({result}, [{}])",
            keys.iter()
                .map(|k| cstr!("\"{k}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !modifiers.non_keys.is_empty() {
        let mods: Vec<&str> = modifiers.non_keys.iter().map(|m| m.as_str()).collect();
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
    use vize_carton::String;

    #[test]
    fn test_generate_event_handler_simple() {
        let modifiers = EventModifiers::default();
        let result = generate_event_handler("click", Some("handleClick"), &modifiers);
        assert_eq!(result, "handleClick");
    }

    #[test]
    fn test_generate_event_handler_with_modifiers() {
        let mut modifiers = EventModifiers::default();
        modifiers.non_keys.push(String::new("stop"));

        let result = generate_event_handler("click", Some("handleClick"), &modifiers);
        insta::assert_snapshot!(result.as_str());
    }
}
