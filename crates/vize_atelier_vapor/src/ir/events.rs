//! Shared native-event semantics for legacy and L3 lowering.

use vize_carton::Allocator;

use super::EventModifiers;

impl<'a> EventModifiers<'a> {
    pub(crate) fn from_names(
        allocator: &'a Allocator,
        event: Option<&str>,
        names: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        let mut modifiers = Self::new(allocator);
        let keyboard = event.is_some_and(|name| {
            ["keydown", "keyup", "keypress"]
                .iter()
                .any(|key| name.eq_ignore_ascii_case(key))
        });
        for name in names {
            match name {
                // Requests delegation; it never guards the handler.
                "delegate" => {}
                "once" => modifiers.options.once = true,
                "capture" => modifiers.options.capture = true,
                "passive" => modifiers.options.passive = true,
                "left" | "right" => {
                    if keyboard || event.is_none() {
                        modifiers.keys.push(name);
                    }
                    if !keyboard {
                        modifiers.non_keys.push(name);
                    }
                }
                "stop" | "prevent" | "self" | "ctrl" | "shift" | "alt" | "meta" | "exact"
                | "middle" => modifiers.non_keys.push(name),
                _ if keyboard || event.is_none() => modifiers.keys.push(name),
                _ => {}
            }
        }
        modifiers
    }

    pub(crate) fn event_name<'b>(&self, name: &'b str) -> &'b str {
        if name == "click" {
            if self.non_keys.contains(&"right") {
                return "contextmenu";
            }
            if self.non_keys.contains(&"middle") {
                return "mouseup";
            }
        }
        name
    }

    /// Whether a listener is delegated, as Vue's compiler decides since
    /// 3.6.0-rc.6: only on request (`.delegate`), for a static delegatable
    /// event without listener options, and not when a listener for the same
    /// event on the element stops propagation (`stopped`, which includes this
    /// listener's own `.stop`).
    pub(crate) fn delegates(&self, requested: bool, name: &str, stopped: bool) -> bool {
        requested
            && !stopped
            && !self.options.once
            && !self.options.capture
            && !self.options.passive
            && matches!(
                name,
                "beforeinput"
                    | "click"
                    | "dblclick"
                    | "contextmenu"
                    | "focusin"
                    | "focusout"
                    | "input"
                    | "keydown"
                    | "keyup"
                    | "mousedown"
                    | "mousemove"
                    | "mouseout"
                    | "mouseover"
                    | "mouseup"
                    | "pointerdown"
                    | "pointermove"
                    | "pointerout"
                    | "pointerover"
                    | "pointerup"
                    | "touchend"
                    | "touchmove"
                    | "touchstart"
            )
    }

    /// Whether a static listener with these modifiers stops propagation of
    /// `name`, the event its modifiers resolve to.
    pub(crate) fn stops(&self, event: &str, name: &str) -> bool {
        self.non_keys.contains(&"stop") && self.event_name(event) == name
    }
}
