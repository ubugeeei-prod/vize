use super::{
    Box, ExpressionNode, IRProp, SimpleExpressionNode, SourceLocation, String, TransformContext,
    Vec,
};

/// Transform v-model on component (helper for transform_component)
pub(super) fn transform_component_v_model<'a>(
    ctx: &mut TransformContext<'a>,
    dir: &vize_atelier_core::DirectiveNode<'a>,
    props: &mut Vec<'a, IRProp<'a>>,
) {
    let binding = if let Some(ref exp) = dir.exp {
        match exp {
            ExpressionNode::Simple(s) => String::new(s.content),
            _ => String::from(""),
        }
    } else {
        String::from("")
    };
    let prop_name = dir
        .arg
        .as_ref()
        .map(|arg| match arg {
            ExpressionNode::Simple(s) => String::new(s.content),
            _ => String::from("modelValue"),
        })
        .unwrap_or_else(|| String::from("modelValue"));

    let key_text = ctx.allocator.alloc_str(&prop_name);
    let val_text = ctx.allocator.alloc_str(&binding);
    let key_node = SimpleExpressionNode::new(key_text, true, SourceLocation::STUB);
    let key = Box::new_in(key_node, &ctx.allocator);
    let mut values = Vec::new_in(&ctx.allocator);
    let val_node = SimpleExpressionNode::new(val_text, false, SourceLocation::STUB);
    values.push(Box::new_in(val_node, &ctx.allocator));
    props.push(IRProp {
        key,
        values,
        is_component: true,
    });

    let event_key = {
        let mut s = String::from("onUpdate:");
        s.push_str(prop_name.as_str());
        ctx.allocator.alloc_str(&s)
    };
    let event_key_node = SimpleExpressionNode::new(event_key, true, SourceLocation::STUB);
    let event_key_box = Box::new_in(event_key_node, &ctx.allocator);
    let handler_content = {
        let mut s = String::from("__RAW__() => _value => (_ctx.");
        s.push_str(binding.as_str());
        s.push_str(" = _value)");
        ctx.allocator.alloc_str(&s)
    };
    let handler_node = SimpleExpressionNode::new(handler_content, true, SourceLocation::STUB);
    let mut handler_values = Vec::new_in(&ctx.allocator);
    handler_values.push(Box::new_in(handler_node, &ctx.allocator));
    props.push(IRProp {
        key: event_key_box,
        values: handler_values,
        is_component: true,
    });

    if !dir.modifiers.is_empty() {
        let mod_key_name = if prop_name == "modelValue" {
            "modelModifiers"
        } else {
            let mut s = prop_name.clone();
            s.push_str("Modifiers");
            ctx.allocator.alloc_str(&s)
        };
        let mod_key_node = SimpleExpressionNode::new(mod_key_name, true, SourceLocation::STUB);
        let mod_key = Box::new_in(mod_key_node, &ctx.allocator);
        let mut mod_content = String::from("__RAW__() => ({ ");
        for (i, m) in dir.modifiers.iter().enumerate() {
            if i > 0 {
                mod_content.push_str(", ");
            }
            mod_content.push_str(m.content);
            mod_content.push_str(": true");
        }
        mod_content.push_str(" })");
        let mod_val = ctx.allocator.alloc_str(&mod_content);
        let mod_val_node = SimpleExpressionNode::new(mod_val, true, SourceLocation::STUB);
        let mut mod_values = Vec::new_in(&ctx.allocator);
        mod_values.push(Box::new_in(mod_val_node, &ctx.allocator));
        props.push(IRProp {
            key: mod_key,
            values: mod_values,
            is_component: true,
        });
    }
}
