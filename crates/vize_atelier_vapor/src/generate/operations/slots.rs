use crate::ir::SlotOutletIRNode;
use vize_carton::{String, cstr};

use super::super::{context::GenerateContext, generate_block, setup::escape_js_string_literal};

/// Generate SlotOutlet
///
/// Emits the Vapor runtime's `createSlot(name?, rawProps?, fallback?)` call,
/// matching `@vue/compiler-vapor`:
/// - a bare default outlet collapses to `_createSlot()`;
/// - dynamic slot names are lazy: `() => (expr)`;
/// - dynamic prop values are getter thunks (`key: () => (expr)`) while static
///   literal values are emitted directly;
/// - `v-bind` spreads join a trailing `$: [...]` source list;
/// - a fallback block is passed as the third argument, with `null` filling the
///   props slot when the outlet has no props.
pub(super) fn generate_slot_outlet(ctx: &mut GenerateContext, slot: &SlotOutletIRNode<'_>) {
    ctx.use_helper("createSlot");
    let name = cstr!("n{}", slot.id);
    let slot_name = if slot.name.is_static {
        cstr!(
            "\"{}\"",
            escape_js_string_literal(slot.name.content.as_str())
        )
    } else {
        cstr!("() => ({})", ctx.resolve_expression_node(&slot.name))
    };

    let slot_props = build_slot_props(ctx, slot);
    match (slot_props, slot.fallback.as_ref()) {
        (None, None) => {
            if slot.name.is_static && slot.name.content == "default" {
                ctx.push_line_fmt(format_args!("const {name} = _createSlot()"));
            } else {
                ctx.push_line_fmt(format_args!("const {name} = _createSlot({slot_name})"));
            }
        }
        (Some(props), None) => {
            ctx.push_line_fmt(format_args!(
                "const {name} = _createSlot({slot_name}, {props})"
            ));
        }
        (None, Some(fallback)) => {
            ctx.push_line_fmt(format_args!(
                "const {name} = _createSlot({slot_name}, null, () => {{"
            ));
            ctx.indent();
            generate_block(ctx, fallback, ctx.element_template_map);
            ctx.deindent();
            ctx.push_line("})");
        }
        (Some(props), Some(fallback)) => {
            ctx.push_line_fmt(format_args!(
                "const {name} = _createSlot({slot_name}, {props}, () => {{"
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

    let mut entries = Vec::new();
    let mut spreads = Vec::new();
    for prop in slot.props.iter() {
        let first = prop.values.first();
        if prop.key.content == "$" {
            let source = first.map_or_else(
                || String::from("undefined"),
                |first| cstr!("() => ({})", ctx.resolve_expression_node(first)),
            );
            spreads.push(source);
            continue;
        }

        // Static literal values are safe to emit directly; dynamic values stay
        // lazy so reading them cannot eagerly touch reactive state.
        let value = first.map_or_else(
            || String::from("undefined"),
            |first| {
                if first.is_static {
                    cstr!("\"{}\"", escape_js_string_literal(first.content.as_str()))
                } else {
                    cstr!("() => ({})", ctx.resolve_expression_node(first))
                }
            },
        );

        if prop.key.is_static {
            entries.push(cstr!("{}: {value}", quote_key(prop.key.content.as_str())));
        } else {
            let key = ctx.resolve_expression_node(&prop.key);
            entries.push(cstr!("[{key}]: {value}"));
        }
    }

    if !spreads.is_empty() {
        entries.push(cstr!("$: [{}]", spreads.join(", ")));
    }

    Some(cstr!("{{ {} }}", entries.join(", ")))
}

/// Emit a static prop key bare when it is a valid identifier, quoted otherwise.
fn quote_key(key: &str) -> String {
    let mut chars = key.chars();
    let ident = chars
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_' || first == '$')
        && chars.all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$');
    if ident {
        key.into()
    } else {
        cstr!("\"{}\"", escape_js_string_literal(key))
    }
}
