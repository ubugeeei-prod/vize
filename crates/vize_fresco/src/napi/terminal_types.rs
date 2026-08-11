//! NAPI type definitions for terminal info and initialization options.

use napi_derive::napi;

/// Terminal info for NAPI.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct TerminalInfoNapi {
    /// Terminal width in columns
    pub width: i32,
    /// Terminal height in rows
    pub height: i32,
    /// Whether colors are supported
    pub colors: bool,
    /// Whether true color (24-bit) is supported
    pub true_color: bool,
    /// Maximum color depth: monochrome, ansi-16, ansi-256, or true-color
    pub color_depth: String,
    /// Stable explanation for the selected color depth
    pub color_reason: String,
    /// Whether Unicode presentation is enabled
    pub unicode: bool,
    /// Stable explanation for Unicode or ASCII presentation
    pub unicode_reason: String,
    /// Whether interactive terminal modes are safe
    pub interactive: bool,
    /// Stable explanation for interactive-mode selection
    pub interactive_reason: String,
    /// Whether standard output is redirected
    pub redirected: bool,
    /// Whether the current viewport selects the narrow layout
    pub narrow: bool,
    /// Width below which the narrow layout is selected
    pub narrow_width: i32,
}

/// Terminal initialization options for NAPI.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct TerminalOptionsNapi {
    /// Enable raw mode
    #[napi(js_name = "rawMode")]
    pub raw_mode: Option<bool>,
    /// Enable the alternate screen buffer
    #[napi(js_name = "alternateScreen")]
    pub alternate_screen: Option<bool>,
    /// Enable mouse capture
    pub mouse: Option<bool>,
    /// Enable bracketed paste mode
    #[napi(js_name = "bracketedPaste")]
    pub bracketed_paste: Option<bool>,
    /// Hide the terminal cursor
    #[napi(js_name = "hideCursor")]
    pub hide_cursor: Option<bool>,
}
