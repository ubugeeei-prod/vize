use vize_croquis::EventHandlerScopeData;

use crate::virtual_ts::helpers::is_known_dom_event_name;

use super::super::handler_shape::{inline_callback_event_argument, is_callable_handler_reference};

pub(super) fn needs_typed_handler_assignment(data: &EventHandlerScopeData) -> bool {
    data.handler_expression.as_ref().is_some_and(|content| {
        (data.has_implicit_event && is_callable_handler_reference(content.as_str()))
            || inline_callback_event_argument(content.as_str()).is_some()
    })
}

pub(super) fn allows_bivariant_handler_assignment(data: &EventHandlerScopeData) -> bool {
    data.handler_expression.as_ref().is_some_and(|content| {
        data.has_implicit_event && is_member_handler_reference(content.as_str())
    })
}

fn is_member_handler_reference(content: &str) -> bool {
    let trimmed = content.trim();
    is_callable_handler_reference(trimmed) && (trimmed.contains('.') || trimmed.contains('['))
}

pub(super) fn transition_hook_signature(
    template_source: Option<&str>,
    directive_start: u32,
    event_name: &str,
) -> Option<(&'static str, &'static str)> {
    let source = template_source?;
    if !is_transition_hook(event_name) || !event_belongs_to_transition(source, directive_start) {
        return None;
    }

    let args = match event_name {
        "enter" | "leave" | "appear" => "[el: Element, done: () => void]",
        _ => "[el: Element]",
    };
    Some(("Element", args))
}

pub(super) fn dynamic_component_custom_event(
    template_source: Option<&str>,
    directive_start: u32,
    event_name: &str,
) -> bool {
    if is_known_dom_event_name(event_name) {
        return false;
    }
    let Some(source) = template_source else {
        return false;
    };
    matches!(event_host_tag(source, directive_start), Some("component"))
}

fn is_transition_hook(event_name: &str) -> bool {
    matches!(
        event_name,
        "before-enter"
            | "beforeEnter"
            | "enter"
            | "after-enter"
            | "afterEnter"
            | "enter-cancelled"
            | "enterCancelled"
            | "before-leave"
            | "beforeLeave"
            | "leave"
            | "after-leave"
            | "afterLeave"
            | "leave-cancelled"
            | "leaveCancelled"
            | "before-appear"
            | "beforeAppear"
            | "appear"
            | "after-appear"
            | "afterAppear"
            | "appear-cancelled"
            | "appearCancelled"
    )
}

fn event_belongs_to_transition(source: &str, directive_start: u32) -> bool {
    matches!(
        event_host_tag(source, directive_start),
        Some("Transition" | "TransitionGroup" | "transition" | "transition-group")
    )
}

fn event_host_tag(source: &str, directive_start: u32) -> Option<&str> {
    let prefix = source.get(..directive_start as usize)?;
    let open = prefix.rfind('<')?;
    if prefix[open..].starts_with("</") {
        return None;
    }
    prefix[open + 1..]
        .trim_start()
        .split(|ch: char| ch.is_ascii_whitespace() || ch == '/' || ch == '>')
        .next()
}
