//! Terminal control module using crossterm.
//!
//! Provides cross-platform terminal manipulation including:
//! - Raw mode initialization/cleanup
//! - Double-buffered rendering
//! - Cursor management
//! - Cell-based character storage with styles

mod backend;
mod buffer;
mod cell;
mod cursor;

pub use backend::Backend;
pub use buffer::Buffer;
pub use cell::{Cell, Color, Style};
pub use cursor::{Cursor, CursorShape};
