//! v-on transform for DOM events.
//!
//! Handles event modifiers and key modifiers.

use vize_atelier_core::DirectiveNode;
use vize_carton::cstr;
use vize_carton::String;

/// Parsed event modifiers
#[derive(Debug, Default, Clone)]
pub struct EventModifiers {
    /// Event options (passive, once, capture)
    pub options: EventOptions,
    /// Propagation modifiers (stop, prevent)
    pub propagation: PropagationModifiers,
    /// Key modifiers for keyboard events
    pub keys: Vec<String>,
    /// System modifiers (ctrl, alt, shift, meta)
    pub system: SystemModifiers,
    /// Mouse button modifiers
    pub mouse: MouseModifiers,
    /// Exact modifier
    pub exact: bool,
    /// Self modifier
    pub self_only: bool,
}

/// Event listener options
#[derive(Debug, Default, Clone)]
pub struct EventOptions {
    pub passive: bool,
    pub once: bool,
    pub capture: bool,
}

/// Propagation modifiers
#[derive(Debug, Default, Clone)]
pub struct PropagationModifiers {
    pub stop: bool,
    pub prevent: bool,
}

/// System key modifiers
#[derive(Debug, Default, Clone)]
pub struct SystemModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// Mouse button modifiers
#[derive(Debug, Default, Clone)]
pub struct MouseModifiers {
    pub left: bool,
    pub middle: bool,
    pub right: bool,
}

impl EventModifiers {
    /// Parse modifiers from directive
    pub fn from_directive(dir: &DirectiveNode<'_>) -> Self {
        let mut modifiers = Self::default();

        for modifier in dir.modifiers.iter() {
            let m = modifier.content.as_str();
            match m {
                // Event options
                "passive" => modifiers.options.passive = true,
                "once" => modifiers.options.once = true,
                "capture" => modifiers.options.capture = true,

                // Propagation
                "stop" => modifiers.propagation.stop = true,
                "prevent" => modifiers.propagation.prevent = true,

                // System modifiers
                "ctrl" => modifiers.system.ctrl = true,
                "alt" => modifiers.system.alt = true,
                "shift" => modifiers.system.shift = true,
                "meta" => modifiers.system.meta = true,

                // Mouse modifiers
                "left" => modifiers.mouse.left = true,
                "middle" => modifiers.mouse.middle = true,
                "right" => modifiers.mouse.right = true,

                // Special
                "exact" => modifiers.exact = true,
                "self" => modifiers.self_only = true,

                // Key modifiers
                _ => {
                    modifiers.keys.push(String::from(m));
                }
            }
        }

        modifiers
    }

    /// Check if has any event options
    pub fn has_options(&self) -> bool {
        self.options.passive || self.options.once || self.options.capture
    }

    /// Check if has any key modifiers
    pub fn has_keys(&self) -> bool {
        !self.keys.is_empty()
    }

    /// Check if has any system modifiers
    pub fn has_system(&self) -> bool {
        self.system.ctrl || self.system.alt || self.system.shift || self.system.meta
    }
}

/// Generate the runtime guard code for modifiers
pub fn generate_modifier_guard(modifiers: &EventModifiers) -> String {
    let mut guards: Vec<&str> = Vec::new();

    // Propagation guards
    if modifiers.propagation.stop {
        guards.push("$event.stopPropagation()");
    }
    if modifiers.propagation.prevent {
        guards.push("$event.preventDefault()");
    }

    // Self guard
    if modifiers.self_only {
        guards.push("if ($event.target !== $event.currentTarget) return");
    }

    // System modifier guards
    let exact_guard;
    if modifiers.exact {
        let mut exact_checks = Vec::new();
        if !modifiers.system.ctrl {
            exact_checks.push("$event.ctrlKey");
        }
        if !modifiers.system.alt {
            exact_checks.push("$event.altKey");
        }
        if !modifiers.system.shift {
            exact_checks.push("$event.shiftKey");
        }
        if !modifiers.system.meta {
            exact_checks.push("$event.metaKey");
        }
        if !exact_checks.is_empty() {
            exact_guard = cstr!("if ({}) return", exact_checks.join(" || "));
            guards.push(&exact_guard);
        }
    } else {
        if modifiers.system.ctrl {
            guards.push("if (!$event.ctrlKey) return");
        }
        if modifiers.system.alt {
            guards.push("if (!$event.altKey) return");
        }
        if modifiers.system.shift {
            guards.push("if (!$event.shiftKey) return");
        }
        if modifiers.system.meta {
            guards.push("if (!$event.metaKey) return");
        }
    }

    // Mouse button guards
    if modifiers.mouse.left {
        guards.push("if ('button' in $event && $event.button !== 0) return");
    }
    if modifiers.mouse.middle {
        guards.push("if ('button' in $event && $event.button !== 1) return");
    }
    if modifiers.mouse.right {
        guards.push("if ('button' in $event && $event.button !== 2) return");
    }

    String::from(guards.join("; "))
}

/// Get key code for a key alias
pub fn resolve_key_alias(key: &str) -> Option<&'static str> {
    match key {
        "esc" => Some("Escape"),
        "space" => Some(" "),
        "up" => Some("ArrowUp"),
        "down" => Some("ArrowDown"),
        "left" => Some("ArrowLeft"),
        "right" => Some("ArrowRight"),
        "delete" => Some("Delete"),
        "backspace" => Some("Backspace"),
        "tab" => Some("Tab"),
        "enter" => Some("Enter"),
        _ => None,
    }
}

/// Generate key guard code
pub fn generate_key_guard(keys: &[String]) -> String {
    if keys.is_empty() {
        return String::default();
    }

    let checks: Vec<String> = keys
        .iter()
        .map(|key| {
            let resolved = resolve_key_alias(key.as_str())
                .map(String::from)
                .unwrap_or_else(|| vize_carton::capitalize(key.as_str()));
            cstr!("$event.key !== \"{resolved}\"")
        })
        .collect();

    cstr!("if ({}) return", checks.join(" && "))
}

#[cfg(test)]
mod tests {
    use super::{generate_key_guard, generate_modifier_guard, resolve_key_alias, EventModifiers};
    use vize_carton::String;

    #[test]
    fn test_parse_modifiers() {
        let modifiers = EventModifiers::default();
        assert!(!modifiers.has_options());
        assert!(!modifiers.has_keys());
    }

    #[test]
    fn test_key_alias() {
        assert_eq!(resolve_key_alias("enter"), Some("Enter"));
        assert_eq!(resolve_key_alias("esc"), Some("Escape"));
        assert_eq!(resolve_key_alias("space"), Some(" "));
        assert_eq!(resolve_key_alias("unknown"), None);
    }

    #[test]
    fn test_generate_key_guard() {
        let keys = vec![String::from("enter")];
        let guard = generate_key_guard(&keys);
        insta::assert_snapshot!(guard.as_str());
    }

    #[test]
    fn test_has_options() {
        let mut mods = EventModifiers::default();
        assert!(!mods.has_options());
        mods.options.passive = true;
        assert!(mods.has_options());
    }

    #[test]
    fn test_has_system() {
        let mut mods = EventModifiers::default();
        assert!(!mods.has_system());
        mods.system.ctrl = true;
        assert!(mods.has_system());
    }

    #[test]
    fn test_generate_modifier_guard_stop_prevent() {
        let mut mods = EventModifiers::default();
        mods.propagation.stop = true;
        mods.propagation.prevent = true;
        let guard = generate_modifier_guard(&mods);
        insta::assert_snapshot!(guard.as_str());
    }

    #[test]
    fn test_generate_modifier_guard_self() {
        let mut mods = EventModifiers::default();
        mods.self_only = true;
        let guard = generate_modifier_guard(&mods);
        insta::assert_snapshot!(guard.as_str());
    }

    #[test]
    fn test_generate_modifier_guard_exact() {
        let mut mods = EventModifiers::default();
        mods.exact = true;
        let guard = generate_modifier_guard(&mods);
        insta::assert_snapshot!(guard.as_str());
    }

    #[test]
    fn test_generate_key_guard_empty() {
        let keys: Vec<String> = vec![];
        let guard = generate_key_guard(&keys);
        assert!(guard.is_empty());
    }

    #[test]
    fn test_generate_key_guard_multiple() {
        let keys = vec![String::from("enter"), String::from("space")];
        let guard = generate_key_guard(&keys);
        insta::assert_snapshot!(guard.as_str());
    }
}
