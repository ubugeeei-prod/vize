//! Keyboard input handling.

use serde::{Deserialize, Serialize};

/// Keyboard event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEvent {
    /// The key pressed
    pub key: Key,
    /// Active modifier keys
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    /// Create a new key event.
    pub fn new(key: Key, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    /// Create a key event from just a key (no modifiers).
    pub fn key(key: Key) -> Self {
        Self::new(key, KeyModifiers::NONE)
    }

    /// Create a key event for a character.
    pub fn char(c: char) -> Self {
        Self::key(Key::Char(c))
    }

    /// Check if Control is held.
    pub fn ctrl(&self) -> bool {
        self.modifiers.ctrl
    }

    /// Check if Alt is held.
    pub fn alt(&self) -> bool {
        self.modifiers.alt
    }

    /// Check if Shift is held.
    pub fn shift(&self) -> bool {
        self.modifiers.shift
    }

    /// Check if this is Ctrl+C.
    pub fn is_ctrl_c(&self) -> bool {
        self.ctrl() && self.key == Key::Char('c')
    }

    /// Check if this is Escape.
    pub fn is_escape(&self) -> bool {
        self.key == Key::Esc
    }

    /// Check if this is Enter.
    pub fn is_enter(&self) -> bool {
        self.key == Key::Enter
    }

    /// Check if this is Backspace.
    pub fn is_backspace(&self) -> bool {
        self.key == Key::Backspace
    }

    /// Check if this is a printable character.
    pub fn is_printable(&self) -> bool {
        matches!(self.key, Key::Char(c) if !c.is_control())
    }
}

impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(event: crossterm::event::KeyEvent) -> Self {
        Self {
            key: event.code.into(),
            modifiers: event.modifiers.into(),
        }
    }
}

/// Key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    /// Printable character
    Char(char),
    /// Function key (F1-F12)
    F(u8),
    /// Backspace
    Backspace,
    /// Enter/Return
    Enter,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Home
    Home,
    /// End
    End,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Tab
    Tab,
    /// Shift+Tab (backtab)
    BackTab,
    /// Delete
    Delete,
    /// Insert
    Insert,
    /// Escape
    Esc,
    /// Caps lock
    CapsLock,
    /// Scroll lock
    ScrollLock,
    /// Num lock
    NumLock,
    /// Print screen
    PrintScreen,
    /// Pause
    Pause,
    /// Menu key
    Menu,
    /// Null (Ctrl+Space)
    Null,
}

impl From<crossterm::event::KeyCode> for Key {
    fn from(code: crossterm::event::KeyCode) -> Self {
        use crossterm::event::KeyCode;
        match code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::F(n) => Key::F(n),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::BackTab,
            KeyCode::Delete => Key::Delete,
            KeyCode::Insert => Key::Insert,
            KeyCode::Esc => Key::Esc,
            KeyCode::CapsLock => Key::CapsLock,
            KeyCode::ScrollLock => Key::ScrollLock,
            KeyCode::NumLock => Key::NumLock,
            KeyCode::PrintScreen => Key::PrintScreen,
            KeyCode::Pause => Key::Pause,
            KeyCode::Menu => Key::Menu,
            KeyCode::Null => Key::Null,
            _ => Key::Null,
        }
    }
}

/// Key modifiers (Ctrl, Alt, Shift, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    /// Shift key
    pub shift: bool,
    /// Control key
    pub ctrl: bool,
    /// Alt/Option key
    pub alt: bool,
    /// Super/Command/Windows key
    pub super_key: bool,
    /// Hyper key
    pub hyper: bool,
    /// Meta key
    pub meta: bool,
}

impl KeyModifiers {
    /// No modifiers
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        super_key: false,
        hyper: false,
        meta: false,
    };

    /// Check if no modifiers are active.
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.super_key && !self.hyper && !self.meta
    }
}

impl From<crossterm::event::KeyModifiers> for KeyModifiers {
    fn from(mods: crossterm::event::KeyModifiers) -> Self {
        Self {
            shift: mods.contains(crossterm::event::KeyModifiers::SHIFT),
            ctrl: mods.contains(crossterm::event::KeyModifiers::CONTROL),
            alt: mods.contains(crossterm::event::KeyModifiers::ALT),
            super_key: mods.contains(crossterm::event::KeyModifiers::SUPER),
            hyper: mods.contains(crossterm::event::KeyModifiers::HYPER),
            meta: mods.contains(crossterm::event::KeyModifiers::META),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, KeyEvent, KeyModifiers};

    #[test]
    fn test_key_event_new() {
        let mods = KeyModifiers {
            ctrl: true,
            ..KeyModifiers::NONE
        };
        let event = KeyEvent::new(Key::Char('a'), mods);
        assert_eq!(event.key, Key::Char('a'));
        assert!(event.ctrl());
    }

    #[test]
    fn test_key_event_char() {
        let event = KeyEvent::char('x');
        assert_eq!(event.key, Key::Char('x'));
        assert!(event.modifiers.is_empty());
    }

    #[test]
    fn test_is_ctrl_c() {
        let mods = KeyModifiers {
            ctrl: true,
            ..KeyModifiers::NONE
        };
        let event = KeyEvent::new(Key::Char('c'), mods);
        assert!(event.is_ctrl_c());

        let event = KeyEvent::char('c');
        assert!(!event.is_ctrl_c());
    }

    #[test]
    fn test_is_printable() {
        let event = KeyEvent::char('a');
        assert!(event.is_printable());

        let event = KeyEvent::key(Key::Enter);
        assert!(!event.is_printable());
    }

    #[test]
    fn test_modifiers_empty() {
        let mods = KeyModifiers::NONE;
        assert!(mods.is_empty());

        let mods = KeyModifiers {
            ctrl: true,
            ..KeyModifiers::NONE
        };
        assert!(!mods.is_empty());
    }
}
