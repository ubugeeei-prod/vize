//! NAPI bindings for Fresco TUI.
//!
//! Provides JavaScript/Node.js bindings for the Fresco terminal UI framework.

#[allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]
mod input;
#[allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]
mod layout;
#[allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]
mod render;
#[allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]
mod terminal;
#[allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]
mod types;

pub use input::{
    disable_ime, enable_ime, get_ime_state, poll_event, poll_event_non_blocking, read_event,
    set_ime_mode,
};
pub use layout::{
    add_layout_child, clear_layout, compute_layout, create_layout_leaf, create_layout_node,
    get_all_layouts, get_layout, init_layout, remove_layout_child, remove_layout_node,
    set_layout_root, set_layout_style,
};
pub use render::{
    clear_rect, fill_rect, hide_cursor, render_box, render_text, render_tree, set_cursor,
    set_cursor_shape, show_cursor,
};
pub use terminal::{
    clear_screen, flush_terminal, get_terminal_info, init_terminal, init_terminal_with_mouse,
    restore_terminal, sync_terminal_size,
};
pub use types::{
    FlexStyleNapi, ImeStateNapi, InputEventNapi, LayoutResultNapi, ModifiersNapi, RenderNodeNapi,
    StyleNapi, TerminalInfoNapi,
};
