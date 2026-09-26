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
    let authored_key = SimpleExpressionNode::from_node(&key_node);

    let key = Box::new_in(key_node, &ctx.allocator);
    let mut values = Vec::new_in(&ctx.allocator);
    let val_node = SimpleExpressionNode::from_node(binding);
    values.push(Box::new_in(val_node, &ctx.allocator));
    props.push(IRProp::new(key, values, true));

    let mut event_key_node = if is_static {
        let event_key = cstr!("onUpdate:{prop_name}");
        SimpleExpressionNode::new(ctx.allocator.alloc_str(&event_key), true, dir.loc.clone())
    } else {
        // The generator derives the event name from this authored argument AST.
        SimpleExpressionNode::from_node(&authored_key)
    };
    event_key_node.is_handler_key = true;
    let event_key_box = Box::new_in(event_key_node, &ctx.allocator);
    // Keep the authored target and retained AST; the shared generator emits
    // its assignment callback without parsing compiler-generated JavaScript.
    let handler_node = SimpleExpressionNode::from_node(binding);
    let mut handler_values = Vec::new_in(&ctx.allocator);
    handler_values.push(Box::new_in(handler_node, &ctx.allocator));
    props.push(
        IRProp::new(event_key_box, handler_values, true)
            .with_value_kind(crate::ir::PropValueKind::ModelUpdate),
    );

    if !dir.modifiers.is_empty() {
        let mod_key_node = if is_static {
            let mod_key_name = if matches!(prop_name, "modelValue" | "model-value") {
                String::from("modelModifiers")
            } else {
                cstr!("{prop_name}Modifiers")
            };
            SimpleExpressionNode::new(
                ctx.allocator.alloc_str(&mod_key_name),
                true,
                dir.loc.clone(),
            )
        } else {
            SimpleExpressionNode::from_node(&authored_key)
        };
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
        props.push(
            IRProp::new(mod_key, mod_values, true)
                .with_value_kind(crate::ir::PropValueKind::ModelModifiers),
        );
    }
}
