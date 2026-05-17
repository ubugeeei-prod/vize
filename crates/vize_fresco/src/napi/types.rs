//! NAPI type definitions.

use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Style options for NAPI.
#[napi(object)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleNapi {
    /// Foreground color (hex or named)
    pub fg: Option<String>,
    /// Background color (hex or named)
    pub bg: Option<String>,
    /// Bold text
    pub bold: Option<bool>,
    /// Dim text
    pub dim: Option<bool>,
    /// Italic text
    pub italic: Option<bool>,
    /// Underline text
    pub underline: Option<bool>,
    /// Inverse background/foreground
    pub inverse: Option<bool>,
    /// Blinking text
    pub blink: Option<bool>,
    /// Hidden text
    pub hidden: Option<bool>,
    /// Strikethrough text
    pub strikethrough: Option<bool>,
}

/// Flex style options for NAPI.
/// NAPI automatically converts JavaScript camelCase to Rust snake_case.
#[napi(object)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlexStyleNapi {
    pub display: Option<String>,
    pub position: Option<String>,
    pub top: Option<String>,
    pub right: Option<String>,
    pub bottom: Option<String>,
    pub left: Option<String>,
    pub overflow: Option<String>,
    #[napi(js_name = "overflowX")]
    pub overflow_x: Option<String>,
    #[napi(js_name = "overflowY")]
    pub overflow_y: Option<String>,
    #[napi(js_name = "flexDirection")]
    pub flex_direction: Option<String>,
    #[napi(js_name = "flexWrap")]
    pub flex_wrap: Option<String>,
    #[napi(js_name = "justifyContent")]
    pub justify_content: Option<String>,
    #[napi(js_name = "alignItems")]
    pub align_items: Option<String>,
    #[napi(js_name = "alignSelf")]
    pub align_self: Option<String>,
    #[napi(js_name = "alignContent")]
    pub align_content: Option<String>,
    #[napi(js_name = "flexGrow")]
    pub flex_grow: Option<f64>,
    #[napi(js_name = "flexShrink")]
    pub flex_shrink: Option<f64>,
    #[napi(js_name = "flexBasis")]
    pub flex_basis: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    #[napi(js_name = "minWidth")]
    pub min_width: Option<String>,
    #[napi(js_name = "minHeight")]
    pub min_height: Option<String>,
    #[napi(js_name = "maxWidth")]
    pub max_width: Option<String>,
    #[napi(js_name = "maxHeight")]
    pub max_height: Option<String>,
    #[napi(js_name = "aspectRatio")]
    pub aspect_ratio: Option<f64>,
    pub padding: Option<f64>,
    #[napi(js_name = "paddingTop")]
    pub padding_top: Option<f64>,
    #[napi(js_name = "paddingRight")]
    pub padding_right: Option<f64>,
    #[napi(js_name = "paddingBottom")]
    pub padding_bottom: Option<f64>,
    #[napi(js_name = "paddingLeft")]
    pub padding_left: Option<f64>,
    pub margin: Option<f64>,
    #[napi(js_name = "marginTop")]
    pub margin_top: Option<f64>,
    #[napi(js_name = "marginRight")]
    pub margin_right: Option<f64>,
    #[napi(js_name = "marginBottom")]
    pub margin_bottom: Option<f64>,
    #[napi(js_name = "marginLeft")]
    pub margin_left: Option<f64>,
    pub gap: Option<f64>,
    #[napi(js_name = "columnGap")]
    pub column_gap: Option<f64>,
    #[napi(js_name = "rowGap")]
    pub row_gap: Option<f64>,
}

/// Render node for NAPI.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderNodeNapi {
    /// Node ID
    pub id: i64,
    /// Node type: "box" | "text" | "input"
    #[napi(js_name = "nodeType")]
    pub node_type: String,
    /// Text content (for text nodes)
    pub text: Option<String>,
    /// Whether text should wrap
    pub wrap: Option<bool>,
    /// Ink-compatible text wrapping/truncation mode
    #[napi(js_name = "wrapMode")]
    pub wrap_mode: Option<String>,
    /// Input value (for input nodes)
    pub value: Option<String>,
    /// Placeholder text (for input nodes)
    pub placeholder: Option<String>,
    /// Whether input is focused
    pub focused: Option<bool>,
    /// Cursor position in input
    pub cursor: Option<i64>,
    /// Whether to mask input (password)
    pub mask: Option<bool>,
    /// Mask character
    #[napi(js_name = "maskChar")]
    pub mask_char: Option<String>,
    /// Flex style
    pub style: Option<FlexStyleNapi>,
    /// Visual appearance
    pub appearance: Option<StyleNapi>,
    /// Border style: "none" | "single" | "double" | "rounded" | "heavy"
    pub border: Option<String>,
    /// Child node IDs
    pub children: Option<Vec<i64>>,
}

/// Layout result for NAPI.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct LayoutResultNapi {
    /// Node ID
    pub id: i64,
    /// X position
    pub x: i32,
    /// Y position
    pub y: i32,
    /// Width
    pub width: i32,
    /// Height
    pub height: i32,
}

/// Input event for NAPI.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct InputEventNapi {
    /// Event type: "key" | "mouse" | "resize" | "focus" | "paste"
    pub event_type: String,
    /// Key code (for key events)
    pub key: Option<String>,
    /// Character (for key events)
    pub char: Option<String>,
    /// Key event type: "press" | "repeat" | "release"
    #[napi(js_name = "keyEventType")]
    pub key_event_type: Option<String>,
    /// Modifiers: { ctrl, alt, shift, meta }
    pub modifiers: Option<ModifiersNapi>,
    /// Mouse button (for mouse events)
    pub button: Option<String>,
    /// Mouse x position
    pub x: Option<i32>,
    /// Mouse y position
    pub y: Option<i32>,
    /// New width (for resize events)
    pub width: Option<i32>,
    /// New height (for resize events)
    pub height: Option<i32>,
    /// Pasted text (for paste events)
    pub text: Option<String>,
    /// Cursor position (for composition events)
    pub cursor: Option<i32>,
}

/// Key modifiers for NAPI.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct ModifiersNapi {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    #[napi(js_name = "super")]
    pub super_key: bool,
    pub hyper: bool,
    #[napi(js_name = "capsLock")]
    pub caps_lock: bool,
    #[napi(js_name = "numLock")]
    pub num_lock: bool,
}

/// IME state for NAPI.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct ImeStateNapi {
    /// Whether IME is active
    pub active: bool,
    /// Current input mode
    pub mode: String,
    /// Whether currently composing
    pub composing: bool,
    /// Preedit text
    pub preedit: Option<String>,
    /// Cursor position in preedit
    pub preedit_cursor: Option<i32>,
    /// Candidate list
    pub candidates: Option<Vec<String>>,
    /// Selected candidate index
    pub selected: Option<i32>,
}

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

fn modifiers_from_key(key: &crate::input::KeyEvent) -> ModifiersNapi {
    ModifiersNapi {
        ctrl: key.ctrl(),
        alt: key.alt(),
        shift: key.shift(),
        meta: key.modifiers.meta,
        super_key: key.modifiers.super_key,
        hyper: key.modifiers.hyper,
        caps_lock: false,
        num_lock: false,
    }
}

impl From<crate::input::Event> for InputEventNapi {
    fn from(event: crate::input::Event) -> Self {
        use crate::input::Event;

        match event {
            Event::Key(key) => {
                let key_str = match key.key {
                    crate::input::Key::Char(c) => {
                        return InputEventNapi {
                            event_type: "key".to_string(),
                            key: None,
                            char: Some(c.to_string()),
                            key_event_type: Some(key.kind.as_str().to_string()),
                            modifiers: Some(modifiers_from_key(&key)),
                            button: None,
                            x: None,
                            y: None,
                            width: None,
                            height: None,
                            text: None,
                            cursor: None,
                        };
                    }
                    crate::input::Key::Enter => "enter",
                    crate::input::Key::Backspace => "backspace",
                    crate::input::Key::Delete => "delete",
                    crate::input::Key::Left => "left",
                    crate::input::Key::Right => "right",
                    crate::input::Key::Up => "up",
                    crate::input::Key::Down => "down",
                    crate::input::Key::Home => "home",
                    crate::input::Key::End => "end",
                    crate::input::Key::PageUp => "pageup",
                    crate::input::Key::PageDown => "pagedown",
                    crate::input::Key::Tab => "tab",
                    crate::input::Key::BackTab => "backtab",
                    crate::input::Key::Esc => "escape",
                    crate::input::Key::F(n) => {
                        #[allow(clippy::disallowed_macros)]
                        return InputEventNapi {
                            event_type: "key".to_string(),
                            key: Some(format!("f{}", n)),
                            char: None,
                            key_event_type: Some(key.kind.as_str().to_string()),
                            modifiers: Some(modifiers_from_key(&key)),
                            button: None,
                            x: None,
                            y: None,
                            width: None,
                            height: None,
                            text: None,
                            cursor: None,
                        };
                    }
                    _ => "unknown",
                };

                InputEventNapi {
                    event_type: "key".to_string(),
                    key: Some(key_str.to_string()),
                    char: None,
                    key_event_type: Some(key.kind.as_str().to_string()),
                    modifiers: Some(modifiers_from_key(&key)),
                    button: None,
                    x: None,
                    y: None,
                    width: None,
                    height: None,
                    text: None,
                    cursor: None,
                }
            }
            Event::Mouse(mouse) => {
                let button = match mouse.kind {
                    crate::input::MouseEventKind::Down(b)
                    | crate::input::MouseEventKind::Up(b)
                    | crate::input::MouseEventKind::Drag(b) => match b {
                        crate::input::MouseButton::Left => Some("left".to_string()),
                        crate::input::MouseButton::Right => Some("right".to_string()),
                        crate::input::MouseButton::Middle => Some("middle".to_string()),
                    },
                    _ => None,
                };

                InputEventNapi {
                    event_type: "mouse".to_string(),
                    key: None,
                    char: None,
                    key_event_type: None,
                    modifiers: None,
                    button,
                    x: Some(mouse.column as i32),
                    y: Some(mouse.row as i32),
                    width: None,
                    height: None,
                    text: None,
                    cursor: None,
                }
            }
            Event::Resize(w, h) => InputEventNapi {
                event_type: "resize".to_string(),
                key: None,
                char: None,
                key_event_type: None,
                modifiers: None,
                button: None,
                x: None,
                y: None,
                width: Some(w as i32),
                height: Some(h as i32),
                text: None,
                cursor: None,
            },
            Event::FocusGained => InputEventNapi {
                event_type: "focus".to_string(),
                key: Some("gained".to_string()),
                char: None,
                key_event_type: None,
                modifiers: None,
                button: None,
                x: None,
                y: None,
                width: None,
                height: None,
                text: None,
                cursor: None,
            },
            Event::FocusLost => InputEventNapi {
                event_type: "focus".to_string(),
                key: Some("lost".to_string()),
                char: None,
                key_event_type: None,
                modifiers: None,
                button: None,
                x: None,
                y: None,
                width: None,
                height: None,
                text: None,
                cursor: None,
            },
            Event::Paste(text) => InputEventNapi {
                event_type: "paste".to_string(),
                key: None,
                char: None,
                key_event_type: None,
                modifiers: None,
                button: None,
                x: None,
                y: None,
                width: None,
                height: None,
                text: Some(text.into()),
                cursor: None,
            },
        }
    }
}
