//! Shared NAPI type definitions for rendering, layout, and input.

use napi_derive::napi;
use serde::{Deserialize, Serialize};

mod input;

pub use input::{ImeStateNapi, InputEventNapi, ModifiersNapi};

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
    /// Node type accepted by the native renderer.
    #[napi(ts_type = "\"root\" | \"box\" | \"text\" | \"input\"")]
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
