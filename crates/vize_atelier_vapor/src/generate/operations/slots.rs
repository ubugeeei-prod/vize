use crate::ir::SlotOutletIRNode;
use vize_carton::{String, cstr};

use super::super::{context::GenerateContext, generate_block, setup::escape_js_string_literal};

/// Generate SlotOutlet
pub(super) fn generate_slot_outlet(ctx: &mut GenerateContext, slot: &SlotOutletIRNode<'_>) {
    ctx.use_helper("renderSlot");
    let name = cstr!("n{}", slot.id);
    let slot_name = if slot.name.is_static {
        cstr!(
            "\"{}\"",
            escape_js_string_literal(slot.name.content.as_str())
        )
    } else {
        ctx.resolve_expression(slot.name.content.as_str())
    };

    let slot_props = build_slot_props(ctx, slot);
    match (slot_props, slot.fallback.as_ref()) {
        (None, None) => {
            ctx.push_line_fmt(format_args!(
                "const {name} = _renderSlot($slots, {slot_name})"
            ));
        }
        (Some(props), None) => {
            ctx.push_line_fmt(format_args!(
                "const {name} = _renderSlot($slots, {slot_name}, {props})"
            ));
        }
        (None, Some(fallback)) => {
            ctx.push_line_fmt(format_args!(
                "const {name} = _renderSlot($slots, {slot_name}, {{}}, () => {{"
            ));
            ctx.indent();
            generate_block(ctx, fallback, ctx.element_template_map);
            ctx.deindent();
            ctx.push_line("})");
        }
        (Some(props), Some(fallback)) => {
            ctx.push_line_fmt(format_args!(
                "const {name} = _renderSlot($slots, {slot_name}, {props}, () => {{"
            ));
            ctx.indent();
            generate_block(ctx, fallback, ctx.element_template_map);
            ctx.deindent();
            ctx.push_line("})");
        }
    }
}

fn build_slot_props(ctx: &GenerateContext, slot: &SlotOutletIRNode<'_>) -> Option<String> {
    if slot.props.is_empty() {
        return None;
    }

    let props = slot
        .props
        .iter()
        .map(|prop| {
            let value = prop.values.first().map_or_else(
                || String::from("undefined"),
                |first| {
                    if first.is_static {
                        cstr!("\"{}\"", escape_js_string_literal(first.content.as_str()))
                    } else {
                        ctx.resolve_expression(first.content.as_str())
                    }
                },
            );

            if prop.key.content == "$" {
                return cstr!("...{value}");
            }

            if prop.key.is_static {
                return cstr!(
                    "\"{}\": {value}",
                    escape_js_string_literal(prop.key.content.as_str())
                );
            }

            let key = ctx.resolve_expression(prop.key.content.as_str());
            cstr!("[{key}]: {value}")
        })
        .collect::<Vec<_>>();

    Some(cstr!("{{ {} }}", props.join(", ")))
}
