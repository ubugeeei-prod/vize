//! Platform-specific IME implementations.
//!
//! Terminal IME is fundamentally different from GUI IME.
//! In terminals, we typically don't have direct IME access, but we can:
//! 1. Detect IME-related key sequences
//! 2. Handle bracketed paste for committed text
//! 3. Provide a fallback inline input experience

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use compact_str::ToCompactString;

use super::state::{ImeEvent, ImeMode, ImeState};
use crate::input::event::Event;

/// Platform IME trait for handling input method events.
pub trait PlatformIme {
    /// Enable IME.
    fn enable(&mut self) -> bool;

    /// Disable IME.
    fn disable(&mut self) -> bool;

    /// Get current IME state.
    fn state(&self) -> &ImeState;

    /// Get mutable IME state.
    fn state_mut(&mut self) -> &mut ImeState;

    /// Process an input event and return any IME events.
    fn process_event(&mut self, event: &Event) -> Option<ImeEvent>;

    /// Set input mode.
    fn set_mode(&mut self, mode: ImeMode);

    /// Check if IME is active.
    fn is_active(&self) -> bool {
        self.state().active
    }
}

/// Terminal-based IME handler.
///
/// Since terminals don't have direct IME integration like GUIs,
/// this provides a software-based input method experience.
#[derive(Debug, Default)]
pub struct TerminalIme {
    state: ImeState,
}

impl TerminalIme {
    /// Create a new terminal IME handler.
    pub fn new() -> Self {
        Self::default()
    }
}

impl PlatformIme for TerminalIme {
    fn enable(&mut self) -> bool {
        self.state.activate();
        true
    }

    fn disable(&mut self) -> bool {
        self.state.deactivate();
        true
    }

    fn state(&self) -> &ImeState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ImeState {
        &mut self.state
    }

    fn process_event(&mut self, event: &Event) -> Option<ImeEvent> {
        match event {
            Event::Paste(text) => {
                // Bracketed paste - treat as committed text
                Some(ImeEvent::Commit(text.clone()))
            }
            Event::Key(key) => {
                if !self.state.active {
                    return None;
                }

                // Handle composition if in a CJK mode
                if self.state.mode.is_japanese()
                    || self.state.mode.is_chinese()
                    || self.state.mode.is_korean()
                {
                    // In a real implementation, this would handle romaji->kana conversion etc.
                    // For now, we just pass through printable characters
                    if key.is_printable() {
                        if let crate::input::keyboard::Key::Char(c) = key.key {
                            // Simple passthrough for now
                            return Some(ImeEvent::Commit(c.to_compact_string()));
                        }
                    }
                }

                None
            }
            _ => None,
        }
    }

    fn set_mode(&mut self, mode: ImeMode) {
        let old_mode = self.state.mode;
        self.state.set_mode(mode);
        if old_mode != mode {
            // Mode change could trigger composition end
            if self.state.is_composing() {
                self.state.end_composition();
            }
        }
    }
}

/// Create platform-specific IME handler.
pub fn create_ime() -> Box<dyn PlatformIme + Send> {
    // For now, use the generic terminal IME on all platforms
    // Platform-specific implementations can be added later
    Box::new(TerminalIme::new())
}

#[cfg(test)]
mod tests {
    use compact_str::ToCompactString;

    use super::{Event, ImeEvent, ImeMode, PlatformIme, TerminalIme};

    #[test]
    fn test_terminal_ime_new() {
        let ime = TerminalIme::new();
        assert!(!ime.is_active());
    }

    #[test]
    fn test_terminal_ime_enable_disable() {
        let mut ime = TerminalIme::new();
        assert!(ime.enable());
        assert!(ime.is_active());
        assert!(ime.disable());
        assert!(!ime.is_active());
    }

    #[test]
    fn test_terminal_ime_paste() {
        let mut ime = TerminalIme::new();
        ime.enable();

        let event = Event::Paste("テスト".to_compact_string());
        let result = ime.process_event(&event);
        assert!(matches!(result, Some(ImeEvent::Commit(s)) if s == "テスト"));
    }

    #[test]
    fn test_terminal_ime_mode() {
        let mut ime = TerminalIme::new();
        ime.set_mode(ImeMode::Hiragana);
        assert_eq!(ime.state().mode, ImeMode::Hiragana);
    }
}
