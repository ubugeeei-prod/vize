use crate::ir::SetEventIRNode;
use vize_carton::{cstr, String, ToCompactString};

use super::super::context::GenerateContext;

/// Generate SetEvent
pub(super) fn generate_set_event(ctx: &mut GenerateContext, set_event: &SetEventIRNode<'_>) {
    ctx.use_helper("createInvoker");

    let element = cstr!("n{}", set_event.element);
    let event_name = &set_event.key.content;

    let handler = if let Some(ref value) = set_event.value {
        value.content.to_compact_string()
    } else {
        String::from("() => {}")
    };

    let resolved_handler = ctx.resolve_expression(&handler);
    // Determine handler format based on content
    let invoker_body: String = if handler.contains("$event") {
        cstr!("$event => ({})", resolved_handler)
    } else if handler.contains("?.") {
        cstr!("(...args) => ({})", resolved_handler)
    } else if is_inline_statement(&handler) || handler.contains('(') {
        cstr!("() => ({})", resolved_handler)
    } else {
        cstr!("e => {}(e)", resolved_handler)
    };

    // Wrap with withModifiers if there are DOM modifiers (stop, prevent, etc.)
    let wrapped_handler = if !set_event.modifiers.non_keys.is_empty() {
        ctx.use_helper("withModifiers");
        let mods = set_event
            .modifiers
            .non_keys
            .iter()
            .map(|m| ["\"", m.as_str(), "\""].concat())
            .collect::<std::vec::Vec<_>>()
            .join(",");
        cstr!("_withModifiers({}, [{}])", invoker_body, mods)
    } else if !set_event.modifiers.keys.is_empty() {
        ctx.use_helper("withKeys");
        let keys = set_event
            .modifiers
            .keys
            .iter()
            .map(|k| ["\"", k.as_str(), "\""].concat())
            .collect::<std::vec::Vec<_>>()
            .join(",");
        cstr!("_withKeys({}, [{}])", invoker_body, keys)
    } else {
        invoker_body
    };

    if set_event.delegate {
        // Use delegation
        ctx.push_line_fmt(format_args!(
            "{}.$evt{} = _createInvoker({})",
            element, event_name, wrapped_handler
        ));
    } else if set_event.effect {
        // Dynamic event - use renderEffect + _on
        ctx.use_helper("on");
        ctx.use_helper("renderEffect");
        let event_expr = ctx.resolve_expression(event_name.as_str());
        ctx.push_line("_renderEffect(() => {");
        ctx.indent();
        ctx.push_line("");
        ctx.push_line_fmt(format_args!(
            "_on({}, {}, _createInvoker({}), {{",
            element, event_expr, wrapped_handler
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
                element, event_name, wrapped_handler
            ));
            ctx.indent();
            for opt in &opts {
                ctx.push_line(opt);
            }
            ctx.deindent();
            ctx.push_line("})");
        } else {
            ctx.push_line_fmt(format_args!(
                "_on({}, \"{}\", _createInvoker({}))",
                element, event_name, wrapped_handler
            ));
        }
    }
}

/// Check if handler is an inline statement (not a function reference)
pub(super) fn is_inline_statement(handler: &str) -> bool {
    // Assignment or increment/decrement operators
    handler.contains("++")
        || handler.contains("--")
        || handler.contains("+=")
        || handler.contains("-=")
        || (handler.contains('=') && !handler.contains("==") && !handler.contains("=>"))
}
