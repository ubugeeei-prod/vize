//! Input event types.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use super::keyboard::KeyEvent;
use super::mouse::MouseEvent;

/// Top-level input event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// Keyboard event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Focus gained
    FocusGained,
    /// Focus lost
    FocusLost,
    /// Paste event (bracketed paste)
    Paste(CompactString),
}

impl Event {
    /// Check if this is a key event.
    pub fn is_key(&self) -> bool {
        matches!(self, Event::Key(_))
    }

    /// Check if this is a mouse event.
    pub fn is_mouse(&self) -> bool {
        matches!(self, Event::Mouse(_))
    }

    /// Check if this is a resize event.
    pub fn is_resize(&self) -> bool {
        matches!(self, Event::Resize(_, _))
    }

    /// Get as key event if applicable.
    pub fn as_key(&self) -> Option<&KeyEvent> {
        match self {
            Event::Key(k) => Some(k),
            _ => None,
        }
    }

    /// Get as mouse event if applicable.
    pub fn as_mouse(&self) -> Option<&MouseEvent> {
        match self {
            Event::Mouse(m) => Some(m),
            _ => None,
        }
    }
}

impl From<crossterm::event::Event> for Event {
    fn from(event: crossterm::event::Event) -> Self {
        match event {
            crossterm::event::Event::Key(k) => Event::Key(k.into()),
            crossterm::event::Event::Mouse(m) => Event::Mouse(m.into()),
            crossterm::event::Event::Resize(w, h) => Event::Resize(w, h),
            crossterm::event::Event::FocusGained => Event::FocusGained,
            crossterm::event::Event::FocusLost => Event::FocusLost,
            crossterm::event::Event::Paste(s) => Event::Paste(CompactString::from(s)),
        }
    }
}

/// Poll for events with a timeout.
pub fn poll(timeout_ms: u64) -> std::io::Result<Option<Event>> {
    use crossterm::event::{poll, read};
    use std::time::Duration;

    if poll(Duration::from_millis(timeout_ms))? {
        Ok(Some(read()?.into()))
    } else {
        Ok(None)
    }
}

/// Poll for events without blocking.
pub fn poll_nonblocking() -> std::io::Result<Option<Event>> {
    poll(0)
}

/// Read an event, blocking until one is available.
pub fn read_event() -> std::io::Result<Event> {
    use crossterm::event::read;
    Ok(read()?.into())
}

#[cfg(test)]
mod tests {
    use super::Event;
    use crate::input::keyboard::KeyEvent;

    #[test]
    fn test_event_is_key() {
        use super::super::keyboard::{Key, KeyModifiers};

        let event = Event::Key(KeyEvent {
            key: Key::Char('a'),
            modifiers: KeyModifiers::NONE,
        });
        assert!(event.is_key());
        assert!(!event.is_mouse());
    }

    #[test]
    fn test_event_resize() {
        let event = Event::Resize(80, 24);
        assert!(event.is_resize());
        if let Event::Resize(w, h) = event {
            assert_eq!(w, 80);
            assert_eq!(h, 24);
        }
    }
}
