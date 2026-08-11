use napi_derive::napi;

/// Input event for NAPI.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct InputEventNapi {
    /// Event type: "key" | "mouse" | "resize" | "focus" | "paste"
    #[napi(ts_type = "\"key\" | \"mouse\" | \"resize\" | \"focus\" | \"paste\" | (string & {})")]
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
                    crate::input::MouseEventKind::Down(button)
                    | crate::input::MouseEventKind::Up(button)
                    | crate::input::MouseEventKind::Drag(button) => match button {
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
            Event::Resize(width, height) => InputEventNapi {
                event_type: "resize".to_string(),
                key: None,
                char: None,
                key_event_type: None,
                modifiers: None,
                button: None,
                x: None,
                y: None,
                width: Some(width as i32),
                height: Some(height as i32),
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
