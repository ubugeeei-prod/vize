use super::{
    Box, ExpressionNode, IRProp, SimpleExpressionNode, SourceLocation, String, TransformContext,
    Vec,
};
use crate::generate::escape_js_string_literal;
use vize_carton::cstr;

/// Transform v-model on component (helper for transform_component)
pub(super) fn transform_component_v_model<'a>(
    ctx: &mut TransformContext<'a>,
    dir: &vize_atelier_core::DirectiveNode<'a>,
    props: &mut Vec<'a, IRProp<'a>>,
) {
    let Some(ExpressionNode::Simple(binding)) = dir.exp.as_ref() else {
        return;
    };
    let key_node = dir
        .arg
        .as_ref()
        .map(|arg| match arg {
            ExpressionNode::Simple(s) => SimpleExpressionNode::from_node(s),
            _ => SimpleExpressionNode::new("modelValue", true, dir.loc.clone()),
        })
        .unwrap_or_else(|| SimpleExpressionNode::new("modelValue", true, dir.loc.clone()));
    let prop_name = key_node.content;
    let is_static = key_node.is_static;

    let key = Box::new_in(key_node, &ctx.allocator);
    let mut values = Vec::new_in(&ctx.allocator);
    let val_node = SimpleExpressionNode::from_node(binding);
    values.push(Box::new_in(val_node, &ctx.allocator));
    props.push(IRProp {
        key,
        values,
        is_component: true,
    });

    let event_key = if is_static {
        cstr!("onUpdate:{prop_name}")
    } else {
        cstr!("\"onUpdate:\" + ({prop_name})")
    };
    let mut event_key_node = SimpleExpressionNode::new(
        ctx.allocator.alloc_str(&event_key),
        is_static,
        dir.loc.clone(),
    );
    event_key_node.is_handler_key = true;
    let event_key_box = Box::new_in(event_key_node, &ctx.allocator);
    // Resolve the assignment through the normal expression pipeline so loop
    // aliases and computed targets retain their lexical scope.
    let handler_content = cstr!("$event => (({}) = $event)", binding.content);
    let handler_node = SimpleExpressionNode::new(
        ctx.allocator.alloc_str(&handler_content),
        false,
        SourceLocation::STUB,
    );
    let mut handler_values = Vec::new_in(&ctx.allocator);
    handler_values.push(Box::new_in(handler_node, &ctx.allocator));
    props.push(IRProp {
        key: event_key_box,
        values: handler_values,
        is_component: true,
    });

    if !dir.modifiers.is_empty() {
        let mod_key_name = if is_static && matches!(prop_name, "modelValue" | "model-value") {
            String::from("modelModifiers")
        } else if is_static {
            cstr!("{prop_name}Modifiers")
        } else {
            cstr!("({prop_name}) + \"Modifiers\"")
        };
        let mod_key_node = SimpleExpressionNode::new(
            ctx.allocator.alloc_str(&mod_key_name),
            is_static,
            dir.loc.clone(),
        );
        let mod_key = Box::new_in(mod_key_node, &ctx.allocator);
        let mut mod_content = String::from("{ ");
        for (i, m) in dir.modifiers.iter().enumerate() {
            if i > 0 {
                mod_content.push_str(", ");
            }
            mod_content.push('"');
            mod_content.push_str(&escape_js_string_literal(m.content));
            mod_content.push_str("\": true");
        }
        mod_content.push_str(" }");
        let mod_val = ctx.allocator.alloc_str(&mod_content);
        let mod_val_node = SimpleExpressionNode::new(mod_val, false, SourceLocation::STUB);
        let mut mod_values = Vec::new_in(&ctx.allocator);
        mod_values.push(Box::new_in(mod_val_node, &ctx.allocator));
        props.push(IRProp {
            key: mod_key,
            values: mod_values,
            is_component: true,
        });
    }
}
