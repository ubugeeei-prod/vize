//! `_` modifiers (#12313): `onClick_stop` names the `click` listener with the
//! `stop` modifier, and `v-name:arg_a_b` names the argument `arg` with the
//! modifiers `a` and `b`, as `vue-jsx-vapor` and `@vue/babel-plugin-jsx`
//! spell them. JSX attribute names cannot hold a `.`.
//!
//! The split is shared with the Relief lowering, so both lowerings read the
//! same spellings the same way.

/// Whether `modifier` is one `v-on` reads: a listener option, a guard, a
/// system or mouse key, or a key alias.
#[must_use]
pub fn is_event_modifier(modifier: &str) -> bool {
    matches!(
        modifier,
        "capture"
            | "once"
            | "passive"
            | "stop"
            | "prevent"
            | "self"
            | "ctrl"
            | "shift"
            | "alt"
            | "meta"
            | "exact"
            | "left"
            | "middle"
            | "right"
            | "enter"
            | "tab"
            | "delete"
            | "esc"
            | "space"
            | "up"
            | "down"
    )
}

/// `onClick_stop_prevent` as its `on<Event>` head (`onClick`) and its
/// modifiers (`stop_prevent`), split on `_`. `None` unless the head is an
/// `on<Event>` name and every segment is an event modifier, so a prop such as
/// `onSelect_item` stays a prop.
#[must_use]
pub fn split_event_modifiers(name: &str) -> Option<(&str, &str)> {
    let (head, modifiers) = name.split_once('_')?;
    let event = head.strip_prefix("on")?;
    if !event.starts_with(|c: char| c.is_ascii_uppercase())
        || !modifiers.split('_').all(is_event_modifier)
    {
        return None;
    }
    Some((head, modifiers))
}

/// `left_a_b`, a custom directive's argument, as the argument (`left`) and
/// its modifiers (`a_b`). `None` without a `_` or with an empty segment.
#[must_use]
pub fn split_directive_argument(argument: &str) -> Option<(&str, &str)> {
    let (argument, modifiers) = argument.split_once('_')?;
    if argument.is_empty() || modifiers.split('_').any(str::is_empty) {
        return None;
    }
    Some((argument, modifiers))
}

#[cfg(test)]
mod tests {
    use super::{split_directive_argument, split_event_modifiers};

    #[test]
    fn event_names_split_only_into_event_modifiers() {
        assert_eq!(
            split_event_modifiers("onClick_stop_prevent"),
            Some(("onClick", "stop_prevent"))
        );
        assert_eq!(
            split_event_modifiers("onKeyupCapture_enter"),
            Some(("onKeyupCapture", "enter"))
        );
        for name in [
            "onSelect_item",
            "onClick_",
            "on_stop",
            "onclick_stop",
            "click_stop",
        ] {
            assert_eq!(split_event_modifiers(name), None, "{name}");
        }
    }

    #[test]
    fn directive_arguments_split_on_every_underscore() {
        assert_eq!(split_directive_argument("left_a_b"), Some(("left", "a_b")));
        for argument in ["left", "_a", "left_", "left__a"] {
            assert_eq!(split_directive_argument(argument), None, "{argument}");
        }
    }
}
