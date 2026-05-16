//! Input handling module.
//!
//! Provides event handling for:
//! - Keyboard events
//! - Mouse events
//! - IME (Input Method Editor) for CJK input

mod event;
pub mod ime;
mod keyboard;
mod mouse;

pub use event::{Event, poll, poll_nonblocking, read_event};
pub use ime::ImeState;
pub use keyboard::{Key, KeyEvent, KeyModifiers};
pub use mouse::{MouseButton, MouseEvent, MouseEventKind};
