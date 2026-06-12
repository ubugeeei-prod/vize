//! Event-handler expression generation and the lightweight JS scanning helpers
//! used to classify handler bodies (callable references vs. inline callbacks).

use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;

use crate::virtual_ts::expressions::rewrite_reserved_template_prop;
use crate::virtual_ts::helpers::generated_text_range;
use crate::virtual_ts::types::VizeMapping;

use super::context::EventHandlerExprContext;

/// Generate event handler expressions inside a closure.
pub(super) fn generate_event_handler_expressions(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    scope_id: u32,
    ctx: &EventHandlerExprContext<'_>,
) {
    if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id) {
        for expr in exprs {
            let content = expr.content.as_str();
            let is_implicit_reference =
                ctx.data.has_implicit_event && is_callable_handler_reference(content);
            let inline_callback_arg = inline_callback_event_argument(content);
            let src_start = (ctx.template_offset + expr.start) as usize;
            let src_end = (ctx.template_offset + expr.end) as usize;
            let guard = expr.vif_guard.as_ref().map(|guard| {
                let trimmed_guard = guard.as_str().trim();
                rewrite_reserved_template_prop(trimmed_guard, ctx.template_prop_names)
                    .unwrap_or_else(|| String::from(guard.as_str()))
            });
            if let Some(ref guard) = guard {
                append!(*ts, "{indent}if ({guard}) {{\n", indent = ctx.indent);
            }
            let handler_indent = if guard.is_some() {
                cstr!("{}  ", ctx.indent)
            } else {
                String::from(ctx.indent)
            };

            let gen_stmt_start = ts.len();
            // Component `@event` handlers carry the full emit listener type so
            // multi-arg emits keep every parameter (#1512). Both a bare callable
            // reference and an inline arrow/function are checked against the
            // listener type and invoked through the typed const with the full
            // argument spread. `__vize_args` is `Parameters<listener>` (a tuple),
            // so the spread always targets the listener's own parameter list,
            // verifying each parameter while avoiding TS2556.
            if let Some(listener_type) = ctx.event_listener_type
                && (is_implicit_reference || inline_callback_arg.is_some())
            {
                let handler_name = cstr!("__vize_handler_{scope_id}_{}", expr.start);
                append!(
                    *ts,
                    "{indent}const {handler_name} = ((handler: {listener_type}) => handler)(({content}));\n",
                    indent = handler_indent,
                );
                append!(
                    *ts,
                    "{indent}{handler_name}(...__vize_args);  // handler expression\n",
                    indent = handler_indent,
                );
            } else if is_implicit_reference {
                let handler_name = cstr!("__vize_handler_{scope_id}_{}", expr.start);
                append!(
                    *ts,
                    "{indent}const {handler_name} = ((handler: ($event: {event_type}) => unknown) => handler)(({content}));\n",
                    indent = handler_indent,
                    event_type = ctx.event_type,
                );
                append!(
                    *ts,
                    "{indent}{handler_name}($event);  // handler expression\n",
                    indent = handler_indent,
                );
            } else if let Some(event_arg) = inline_callback_arg {
                append!(
                    *ts,
                    "{indent}({content})({event_arg});  // handler expression\n",
                    indent = ctx.indent,
                );
            } else {
                append!(
                    *ts,
                    "{indent}{content};  // handler expression\n",
                    indent = handler_indent
                );
            }
            let gen_stmt_end = ts.len();
            mappings.push(VizeMapping {
                gen_range: if is_implicit_reference {
                    gen_stmt_start..gen_stmt_end
                } else {
                    generated_text_range(&ts[gen_stmt_start..gen_stmt_end], content, gen_stmt_start)
                },
                src_range: src_start..src_end,
                sub_spans: Vec::new(),
            });
            append!(
                *ts,
                "{indent}// @vize-map: handler -> {src_start}:{src_end}\n",
                indent = handler_indent,
            );
            if guard.is_some() {
                append!(*ts, "{indent}}}\n", indent = ctx.indent);
            }
        }
    }
}

fn inline_callback_event_argument(content: &str) -> Option<&'static str> {
    let trimmed = content.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(arrow_idx) = trimmed.find("=>") {
        let before_arrow = strip_async_prefix(trimmed[..arrow_idx].trim_end()).trim();
        if before_arrow.is_empty() {
            return None;
        }

        if let Some(is_empty) = parenthesized_params_are_empty(before_arrow) {
            return Some(if is_empty { "" } else { "$event" });
        }

        return is_identifier_segment(before_arrow).then_some("$event");
    }

    let rest = trimmed.strip_prefix("function")?;
    let paren_start = trimmed.len() - rest.len() + rest.find('(')?;
    let paren_end = matching_paren_index(trimmed, paren_start)?;
    let inner = &trimmed[paren_start + 1..paren_end];
    Some(if inner.trim().is_empty() {
        ""
    } else {
        "$event"
    })
}

fn strip_async_prefix(input: &str) -> &str {
    let Some(rest) = input.strip_prefix("async") else {
        return input;
    };
    if rest.chars().next().is_some_and(char::is_whitespace) {
        rest.trim_start()
    } else {
        input
    }
}

fn parenthesized_params_are_empty(input: &str) -> Option<bool> {
    if !input.starts_with('(') {
        return None;
    }
    let close = matching_paren_index(input, 0)?;
    if !input[close + 1..].trim().is_empty() {
        return None;
    }
    Some(input[1..close].trim().is_empty())
}

fn matching_paren_index(input: &str, open_index: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.get(open_index) != Some(&b'(') {
        return None;
    }

    let mut depth = 0u32;
    for (idx, byte) in bytes.iter().enumerate().skip(open_index) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }

    None
}

fn is_callable_handler_reference(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }

    let Some(mut idx) = parse_identifier_segment(trimmed, 0) else {
        return false;
    };

    loop {
        idx = skip_ascii_whitespace(trimmed, idx);
        if idx == trimmed.len() {
            return true;
        }

        let rest = &trimmed[idx..];
        if rest.starts_with("?.[") {
            idx += 2;
            let Some(next_idx) = parse_bracket_member(trimmed, idx) else {
                return false;
            };
            idx = next_idx;
        } else if rest.starts_with("?.") {
            let Some(next_idx) = parse_identifier_segment(trimmed, idx + 2) else {
                return false;
            };
            idx = next_idx;
        } else if rest.starts_with('.') {
            let Some(next_idx) = parse_identifier_segment(trimmed, idx + 1) else {
                return false;
            };
            idx = next_idx;
        } else if rest.starts_with('[') {
            let Some(next_idx) = parse_bracket_member(trimmed, idx) else {
                return false;
            };
            idx = next_idx;
        } else {
            return false;
        }
    }
}

fn is_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first == '$' || first.is_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch == '$' || ch.is_alphanumeric())
}

fn parse_identifier_segment(input: &str, start: usize) -> Option<usize> {
    let mut chars = input.get(start..)?.char_indices();
    let (_, first) = chars.next()?;
    if !is_identifier_start(first) {
        return None;
    }

    let mut end = start + first.len_utf8();
    for (offset, ch) in chars {
        if !is_identifier_continue(ch) {
            break;
        }
        end = start + offset + ch.len_utf8();
    }
    Some(end)
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphanumeric()
}

fn skip_ascii_whitespace(input: &str, mut idx: usize) -> usize {
    while input
        .as_bytes()
        .get(idx)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        idx += 1;
    }
    idx
}

fn parse_bracket_member(input: &str, open_index: usize) -> Option<usize> {
    if input.as_bytes().get(open_index) != Some(&b'[') {
        return None;
    }

    let mut depth = 0u32;
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in input
        .char_indices()
        .skip_while(|(idx, _)| *idx < open_index)
    {
        if let Some(quote_ch) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '[' => depth += 1,
            ']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(idx + ch.len_utf8());
                }
            }
            _ => {}
        }
    }

    None
}
