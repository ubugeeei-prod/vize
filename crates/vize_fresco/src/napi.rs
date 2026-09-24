//! NAPI bindings for Fresco TUI.
//!
//! Provides JavaScript/Node.js bindings for the Fresco terminal UI framework.

#[expect(
    clippy::disallowed_macros,
    reason = "the N-API boundary exchanges std strings with napi-rs and its derives"
)]
mod frame_output;
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the N-API boundary exchanges std strings with napi-rs and its derives"
)]
mod input;
mod input_size;
#[expect(
    clippy::disallowed_macros,
    reason = "the N-API boundary exchanges std strings with napi-rs and its derives"
)]
mod layout;
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "the N-API boundary exchanges std strings with napi-rs and its derives"
)]
mod render;
#[expect(
    clippy::disallowed_macros,
    reason = "the N-API boundary exchanges std strings with napi-rs and its derives"
)]
mod render_payload;
#[cfg(test)]
mod render_tests;
#[expect(
    clippy::disallowed_macros,
    reason = "the N-API boundary exchanges std strings with napi-rs and its derives"
)]
mod terminal;
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "the N-API boundary exchanges std strings with napi-rs and its derives"
)]
mod terminal_types;
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the N-API boundary exchanges std strings with napi-rs and its derives"
)]
mod types;

pub use frame_output::FrameOutputTelemetryNapi;
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
    clear_rect, fill_rect, get_last_render_layouts, hide_cursor, render_box, render_text,
    render_tree, set_cursor, set_cursor_shape, show_cursor,
};
pub use terminal::{
    clear_screen, flush_terminal, flush_terminal_measured, get_terminal_info, init_terminal,
    init_terminal_with_mouse, init_terminal_with_options, restore_terminal, sync_terminal_size,
};
pub use terminal_types::{TerminalInfoNapi, TerminalOptionsNapi};
pub use types::{
    FlexStyleNapi, ImeStateNapi, InputEventNapi, LayoutResultNapi, ModifiersNapi, RenderNodeNapi,
    StyleNapi,
};
