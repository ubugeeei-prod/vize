//! Mouse input handling.

use serde::{Deserialize, Serialize};

use super::keyboard::KeyModifiers;

/// Mouse event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MouseEvent {
    /// Type of mouse event
    pub kind: MouseEventKind,
    /// Column (x coordinate)
    pub column: u16,
    /// Row (y coordinate)
    pub row: u16,
    /// Active modifier keys
    pub modifiers: KeyModifiers,
}

impl MouseEvent {
    /// Create a new mouse event.
    pub fn new(kind: MouseEventKind, column: u16, row: u16, modifiers: KeyModifiers) -> Self {
        Self {
            kind,
            column,
            row,
            modifiers,
        }
    }

    /// Check if this is a left click.
    pub fn is_left_click(&self) -> bool {
        matches!(
            self.kind,
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left)
        )
    }

    /// Check if this is a right click.
    pub fn is_right_click(&self) -> bool {
        matches!(
            self.kind,
            MouseEventKind::Down(MouseButton::Right) | MouseEventKind::Up(MouseButton::Right)
        )
    }

    /// Check if this is a scroll event.
    pub fn is_scroll(&self) -> bool {
        matches!(
            self.kind,
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
        )
    }

    /// Check if this is a drag event.
    pub fn is_drag(&self) -> bool {
        matches!(self.kind, MouseEventKind::Drag(_))
    }

    /// Check if this is a move event.
    pub fn is_move(&self) -> bool {
        matches!(self.kind, MouseEventKind::Moved)
    }

    /// Get position as (x, y) tuple.
    pub fn position(&self) -> (u16, u16) {
        (self.column, self.row)
    }
}

impl From<crossterm::event::MouseEvent> for MouseEvent {
    fn from(event: crossterm::event::MouseEvent) -> Self {
        Self {
            kind: event.kind.into(),
            column: event.column,
            row: event.row,
            modifiers: event.modifiers.into(),
        }
    }
}

/// Mouse event kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseEventKind {
    /// Button pressed down
    Down(MouseButton),
    /// Button released
    Up(MouseButton),
    /// Drag with button held
    Drag(MouseButton),
    /// Mouse moved (no button)
    Moved,
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Scroll left
    ScrollLeft,
    /// Scroll right
    ScrollRight,
}

impl From<crossterm::event::MouseEventKind> for MouseEventKind {
    fn from(kind: crossterm::event::MouseEventKind) -> Self {
        use crossterm::event::MouseEventKind as CT;
        match kind {
            CT::Down(b) => MouseEventKind::Down(b.into()),
            CT::Up(b) => MouseEventKind::Up(b.into()),
            CT::Drag(b) => MouseEventKind::Drag(b.into()),
            CT::Moved => MouseEventKind::Moved,
            CT::ScrollUp => MouseEventKind::ScrollUp,
            CT::ScrollDown => MouseEventKind::ScrollDown,
            CT::ScrollLeft => MouseEventKind::ScrollLeft,
            CT::ScrollRight => MouseEventKind::ScrollRight,
        }
    }
}

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    /// Left button
    Left,
    /// Right button
    Right,
    /// Middle button (scroll wheel)
    Middle,
}

impl From<crossterm::event::MouseButton> for MouseButton {
    fn from(button: crossterm::event::MouseButton) -> Self {
        use crossterm::event::MouseButton as CT;
        match button {
            CT::Left => MouseButton::Left,
            CT::Right => MouseButton::Right,
            CT::Middle => MouseButton::Middle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MouseButton, MouseEvent, MouseEventKind};
    use crate::input::keyboard::KeyModifiers;

    #[test]
    fn test_mouse_event_new() {
        let event = MouseEvent::new(
            MouseEventKind::Down(MouseButton::Left),
            10,
            20,
            KeyModifiers::NONE,
        );
        assert_eq!(event.column, 10);
        assert_eq!(event.row, 20);
        assert!(event.is_left_click());
    }

    #[test]
    fn test_mouse_position() {
        let event = MouseEvent::new(MouseEventKind::Moved, 5, 15, KeyModifiers::NONE);
        assert_eq!(event.position(), (5, 15));
        assert!(event.is_move());
    }

    #[test]
    fn test_scroll() {
        let event = MouseEvent::new(MouseEventKind::ScrollUp, 0, 0, KeyModifiers::NONE);
        assert!(event.is_scroll());
    }
}
