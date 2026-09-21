//! Shared native-event semantics for legacy and S3 lowering.

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

    pub(crate) fn can_delegate(&self, name: &str) -> bool {
        !self.options.once
            && !self.options.capture
            && !self.options.passive
            && matches!(
                name,
                "click"
                    | "dblclick"
                    | "mousedown"
                    | "mouseup"
                    | "mousemove"
                    | "mouseover"
                    | "mouseout"
                    | "keydown"
                    | "keyup"
                    | "keypress"
                    | "pointerdown"
                    | "pointerup"
                    | "pointermove"
                    | "pointerover"
                    | "pointerout"
                    | "touchstart"
                    | "touchend"
                    | "touchmove"
                    | "focusin"
                    | "focusout"
                    | "input"
                    | "change"
                    | "contextmenu"
                    | "wheel"
                    | "drag"
                    | "dragstart"
                    | "dragend"
                    | "dragenter"
                    | "dragleave"
                    | "dragover"
                    | "drop"
            )
    }
}
