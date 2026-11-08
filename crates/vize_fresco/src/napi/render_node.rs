//! Conversion from N-API render payloads to native render nodes.

use crate::layout::Dimension;
use crate::render::{InputContent, NodeKind, RawContent, RenderNode, TextContent};
use crate::text::TextWidth;

use super::input_size::input_intrinsic_size;
use super::render_payload::{RenderNodeKindNapi, parse_wrap_mode};
use super::types::RenderNodeNapi;

pub(super) fn create_render_node(
    node: &RenderNodeNapi,
    node_type: RenderNodeKindNapi,
) -> RenderNode {
    let text = node.text.clone().unwrap_or_default();
    let kind = match node_type {
        RenderNodeKindNapi::Text => NodeKind::Text(TextContent {
            text: text.clone().into(),
            wrap: node.wrap.unwrap_or(false),
            wrap_mode: parse_wrap_mode(node.wrap_mode.as_deref(), node.wrap),
        }),
        RenderNodeKindNapi::Input => NodeKind::Input(InputContent {
            value: node.value.clone().unwrap_or_default().into(),
            placeholder: node.placeholder.clone().unwrap_or_default().into(),
            cursor: node.cursor.unwrap_or(0) as usize,
            focused: node.focused.unwrap_or(false),
            mask: node.mask.unwrap_or(false),
            selection_start: node.selection_start.unwrap_or(0).max(0) as usize,
            selection_end: node.selection_end.unwrap_or(0).max(0) as usize,
            mask_char: node
                .mask_char
                .as_deref()
                .and_then(|value| value.chars().next())
                .unwrap_or('*'),
        }),
        RenderNodeKindNapi::Raw => {
            NodeKind::Raw(RawContent::new(node.lines.clone().unwrap_or_default()))
        }
        RenderNodeKindNapi::Root | RenderNodeKindNapi::Box => NodeKind::Box,
    };
    let mut render_node = RenderNode::new(node.id as u64, kind);

    match node_type {
        RenderNodeKindNapi::Text if !text.is_empty() => {
            render_node.style.width = Dimension::Points(TextWidth::width(&text) as f32);
            render_node.style.height = Dimension::Points(text.lines().count().max(1) as f32);
        }
        RenderNodeKindNapi::Input => {
            let (width, height) = input_intrinsic_size(node);
            render_node.style.width = Dimension::Points(width);
            render_node.style.height = Dimension::Points(height);
        }
        RenderNodeKindNapi::Raw => {
            let lines = node.lines.as_deref().unwrap_or_default();
            let width = lines
                .iter()
                .map(|line| TextWidth::width(line))
                .max()
                .unwrap_or_default();
            render_node.style.width = Dimension::Points(width as f32);
            render_node.style.height = Dimension::Points(lines.len().max(1) as f32);
        }
        _ => {}
    }

    render_node
}
