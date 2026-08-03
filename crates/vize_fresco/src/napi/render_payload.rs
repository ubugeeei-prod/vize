//! Render payload parsing for the render NAPI bindings.

use napi::bindgen_prelude::*;

use super::types::RenderNodeNapi;
use crate::text::WrapMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenderNodeKindNapi {
    Root,
    Box,
    Text,
    Input,
}

pub(super) fn parse_render_node_kind(value: &str) -> Option<RenderNodeKindNapi> {
    match value {
        "root" => Some(RenderNodeKindNapi::Root),
        "box" => Some(RenderNodeKindNapi::Box),
        "text" => Some(RenderNodeKindNapi::Text),
        "input" => Some(RenderNodeKindNapi::Input),
        _ => None,
    }
}

/// Resolve every node kind in a render payload, reporting the first unsupported
/// `nodeType` so the batch can be rejected before the backend is borrowed.
pub(super) fn validate_render_node_kinds(
    nodes: &[RenderNodeNapi],
) -> std::result::Result<Vec<RenderNodeKindNapi>, &str> {
    nodes
        .iter()
        .map(|node| parse_render_node_kind(&node.node_type).ok_or(node.node_type.as_str()))
        .collect()
}

pub(super) fn unsupported_render_node_kind(node_type: &str) -> Error {
    Error::new(
        Status::InvalidArg,
        format!(
            "Unsupported render node type `{node_type}`; expected one of: root, box, text, input"
        ),
    )
}

pub(super) fn parse_wrap_mode(mode: Option<&str>, wrap: Option<bool>) -> WrapMode {
    match mode {
        Some("wrap") => WrapMode::Word,
        Some("hard") => WrapMode::Char,
        Some("truncate") | Some("truncate-end") => WrapMode::TruncateEnd,
        Some("truncate-start") => WrapMode::TruncateStart,
        Some("truncate-middle") => WrapMode::TruncateMiddle,
        Some("false") | Some("none") => WrapMode::NoWrap,
        _ if wrap.unwrap_or(false) => WrapMode::Word,
        _ => WrapMode::NoWrap,
    }
}
