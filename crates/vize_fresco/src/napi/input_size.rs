use super::types::{FlexStyleNapi, RenderNodeNapi};
use crate::text::TextWidth;

const DEFAULT_INPUT_WIDTH: usize = 30;

pub(super) fn input_intrinsic_size(node: &RenderNodeNapi) -> (f32, f32) {
    let value = node.value.as_deref().unwrap_or("");
    let placeholder = node.placeholder.as_deref().unwrap_or("");
    let content = if value.is_empty() { placeholder } else { value };
    let input_width = input_wrap_width(node.style.as_ref());
    let height = wrapped_line_count(TextWidth::width(content), input_width);

    (input_width as f32, height as f32)
}

pub(super) fn wrapped_line_count(content_width: usize, wrap_width: usize) -> usize {
    content_width.div_ceil(wrap_width.max(1)).max(1)
}

fn input_wrap_width(style: Option<&FlexStyleNapi>) -> usize {
    style
        .and_then(|style| style.width.as_deref())
        .and_then(parse_positive_point_width)
        .unwrap_or(DEFAULT_INPUT_WIDTH)
}

fn parse_positive_point_width(value: &str) -> Option<usize> {
    if value == "auto" || value.ends_with('%') {
        return None;
    }

    let value = value.parse::<f32>().ok()?;
    if value.is_finite() && value > 0.0 {
        Some(value.ceil() as usize)
    } else {
        None
    }
}
