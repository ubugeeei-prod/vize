/// Get the TypeScript event type for a DOM event name.
/// Returns the specific event interface (MouseEvent, KeyboardEvent, etc.)
pub(crate) fn get_dom_event_type(event_name: &str) -> &'static str {
    dom_event_type(event_name).unwrap_or("Event")
}

pub(crate) fn is_known_dom_event_name(event_name: &str) -> bool {
    dom_event_type(event_name).is_some()
}

fn dom_event_type(event_name: &str) -> Option<&'static str> {
    match event_name {
        // Mouse events
        "dblclick" | "mousedown" | "mouseup" | "mousemove" | "mouseenter" | "mouseleave"
        | "mouseover" | "mouseout" => Some("MouseEvent"),

        "click" | "auxclick" | "contextmenu" | "pointerdown" | "pointerup" | "pointermove"
        | "pointerenter" | "pointerleave" | "pointerover" | "pointerout" | "pointercancel"
        | "gotpointercapture" | "lostpointercapture" => Some("PointerEvent"),

        // Touch events
        "touchstart" | "touchend" | "touchmove" | "touchcancel" => Some("TouchEvent"),

        // Keyboard events
        "keydown" | "keyup" | "keypress" => Some("KeyboardEvent"),

        // Focus events
        "focus" | "blur" | "focusin" | "focusout" => Some("FocusEvent"),

        // Input events
        "input" | "beforeinput" => Some("InputEvent"),

        // Composition events
        "compositionstart" | "compositionend" | "compositionupdate" => Some("CompositionEvent"),

        // Form events
        "submit" => Some("SubmitEvent"),
        "change" => Some("Event"),
        "reset" => Some("Event"),

        // Drag events
        "drag" | "dragstart" | "dragend" | "dragenter" | "dragleave" | "dragover" | "drop" => {
            Some("DragEvent")
        }

        // Clipboard events
        "cut" | "copy" | "paste" => Some("ClipboardEvent"),

        // Wheel events
        "wheel" => Some("WheelEvent"),

        // Animation events
        "animationstart" | "animationend" | "animationiteration" | "animationcancel" => {
            Some("AnimationEvent")
        }

        // Transition events
        "transitionstart" | "transitionend" | "transitionrun" | "transitioncancel" => {
            Some("TransitionEvent")
        }

        // UI events
        "scroll" | "resize" => Some("Event"),

        // Media events
        "play" | "pause" | "ended" | "loadeddata" | "loadedmetadata" | "timeupdate"
        | "volumechange" | "waiting" | "seeking" | "seeked" | "ratechange" | "durationchange"
        | "canplay" | "canplaythrough" | "playing" | "progress" | "stalled" | "suspend"
        | "emptied" | "abort" => Some("Event"),

        // Error/Load events
        "error" => Some("ErrorEvent"),
        "load" => Some("Event"),

        // Selection events
        "select" | "selectionchange" | "selectstart" => Some("Event"),

        // Modern UI events that were absent - without an entry, `@toggle` /
        // `@beforetoggle` etc. fell back to the bare `Event` interface and
        // the user lost the specific payload members. See #688.
        "toggle" | "beforetoggle" => Some("ToggleEvent"),
        "formdata" => Some("FormDataEvent"),
        "popstate" => Some("PopStateEvent"),
        "hashchange" => Some("HashChangeEvent"),
        "message" => Some("MessageEvent"),
        "storage" => Some("StorageEvent"),
        "online" | "offline" => Some("Event"),
        "securitypolicyviolation" => Some("SecurityPolicyViolationEvent"),

        // Default fallback
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::get_dom_event_type;

    #[test]
    fn maps_legacy_dom_events() {
        assert_eq!(get_dom_event_type("click"), "PointerEvent");
        assert_eq!(get_dom_event_type("auxclick"), "PointerEvent");
        assert_eq!(get_dom_event_type("contextmenu"), "PointerEvent");
        assert_eq!(get_dom_event_type("dblclick"), "MouseEvent");
        assert_eq!(get_dom_event_type("keydown"), "KeyboardEvent");
        assert_eq!(get_dom_event_type("submit"), "SubmitEvent");
    }

    #[test]
    fn maps_modern_dom_events() {
        // These fell back to `Event` before #688 - now they get the specific
        // interface so `e.newState` / `e.formData` etc. complete.
        assert_eq!(get_dom_event_type("toggle"), "ToggleEvent");
        assert_eq!(get_dom_event_type("beforetoggle"), "ToggleEvent");
        assert_eq!(get_dom_event_type("formdata"), "FormDataEvent");
    }

    #[test]
    fn unknown_events_fall_back_to_event() {
        assert_eq!(get_dom_event_type("totally-made-up"), "Event");
    }
}
