use crate::ir::SetEventIRNode;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_atelier_core::steps::{is_event_handler_reference_node, is_function_expression_node};
use vize_carton::cstr;

pub(super) use super::super::expression_retained::resolve_inline_handler_node as resolve_inline_handler;
use super::super::{context::GenerateContext, expression::is_simple_path_expression};

/// Generate SetEvent
pub(super) fn generate_set_event(ctx: &mut GenerateContext, set_event: &SetEventIRNode<'_>) {
    ctx.use_helper("createInvoker");

    let element = cstr!("n{}", set_event.element);
    let event_name = &set_event.key.content;

    // Handler references and callbacks keep their expression anchors;
    // rewritten inline bodies are emitted unanchored.
    let invoker_body = if let Some(value) = set_event.value.as_deref() {
        // Keep S3's checked direct-reference path free of expression reparses.
        if is_simple_path_expression(value.content.trim()) || is_event_handler_reference_node(value)
        {
            let mut body = EmitDocument::plain("e => ");
            body.push_spanned(&ctx.spanned_expression_node(value));
            body.push_str("(e)");
            body
        } else if is_function_expression_node(value) {
            // Authored and transformed callbacks already own their parameters.
            ctx.spanned_expression_node(value)
        } else {
            EmitDocument::from(resolve_inline_handler(ctx, value))
        }
    } else {
        EmitDocument::plain("() => {}")
    };

    // Wrap with withModifiers if there are DOM modifiers (stop, prevent, etc.)
    let wrapped_handler = if !set_event.modifiers.non_keys.is_empty() {
        ctx.use_helper("withModifiers");
        let mods = set_event
            .modifiers
            .non_keys
            .iter()
            .map(|m| ["\"", m, "\""].concat())
            .collect::<std::vec::Vec<_>>()
            .join(",");
        wrap_handler("_withModifiers(", &invoker_body, &mods)
    } else {
        invoker_body
    };
    // Key filtering wraps the DOM guard: a non-matching key must not stop
    // propagation or prevent the default action before it is rejected.
    let wrapped_handler = if !set_event.modifiers.keys.is_empty() {
        ctx.use_helper("withKeys");
        let keys = set_event
            .modifiers
            .keys
            .iter()
            .map(|k| ["\"", k, "\""].concat())
            .collect::<std::vec::Vec<_>>()
            .join(",");
        wrap_handler("_withKeys(", &wrapped_handler, &keys)
    } else {
        wrapped_handler
    };

    if set_event.delegate {
        ctx.add_delegate_event(event_name);
        let mut line =
            EmitDocument::plain(&cstr!("{}.$evt{} = _createInvoker(", element, event_name));
        line.push_spanned(&wrapped_handler);
        line.push_str(")");
        ctx.push_line_spanned(&line);
    } else if set_event.effect {
        // Dynamic event - use renderEffect + _on
        ctx.use_helper("on");
        ctx.use_helper("renderEffect");
        let event_expr = ctx.resolve_expression_node(&set_event.key);
        ctx.push_line("_renderEffect(() => {");
        ctx.indent();
        ctx.push_line("");
        ctx.push_line_fmt(format_args!(
            "_on({}, {}, _createInvoker({}), {{",
            element,
            event_expr,
            wrapped_handler.as_str()
        ));
        ctx.indent();
        ctx.push_line("effect: true");
        ctx.deindent();
        ctx.push_line("})");
        ctx.deindent();
        ctx.push_line("})");
    } else {
        // Use _on() for non-delegatable events or events with once/capture/passive
        ctx.use_helper("on");

        let has_options = set_event.modifiers.options.once
            || set_event.modifiers.options.capture
            || set_event.modifiers.options.passive;

        if has_options {
            let mut opts = std::vec::Vec::new();
            if set_event.modifiers.options.once {
                opts.push("once: true");
            }
            if set_event.modifiers.options.capture {
                opts.push("capture: true");
            }
            if set_event.modifiers.options.passive {
                opts.push("passive: true");
            }
            ctx.push_line_fmt(format_args!(
                "_on({}, \"{}\", _createInvoker({}), {{",
                element,
                event_name,
                wrapped_handler.as_str()
            ));
            ctx.indent();
            for (index, opt) in opts.iter().enumerate() {
                let comma = if index + 1 < opts.len() { "," } else { "" };
                ctx.push_line_fmt(format_args!("{opt}{comma}"));
            }
            ctx.deindent();
            ctx.push_line("})");
        } else {
            ctx.push_line_fmt(format_args!(
                "_on({}, \"{}\", _createInvoker({}))",
                element,
                event_name,
                wrapped_handler.as_str()
            ));
        }
    }
}

/// `callee` + handler + `, [` + list + `])`, keeping the handler's anchors.
fn wrap_handler(callee: &str, handler: &EmitDocument, list: &str) -> EmitDocument {
    let mut out = EmitDocument::plain(callee);
    out.push_spanned(handler);
    out.push_str(", [");
    out.push_str(list);
    out.push_str("])");
    out
}
