//! Utility modules for vize_maestro.

pub mod position;

pub use position::{
    line_range, make_range, offset_to_position, offset_to_position_str, position_to_offset,
    position_to_offset_str,
};

fn has_ascii_extension(path: &str, extensions: &[&str]) -> bool {
    path.rsplit_once('.').is_some_and(|(_, extension)| {
        extensions
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    })
}

/// Returns true for standalone HTML files that should be linted outside the SFC pipeline.
#[inline]
pub fn is_standalone_html_path(path: &str) -> bool {
    has_ascii_extension(path, &["html", "htm"])
}

/// Routes JSX and TSX files to the standalone compiler diagnostics (see #1497).
#[inline]
pub fn is_jsx_path(path: &str) -> bool {
    has_ascii_extension(path, &["jsx", "tsx"])
}

/// A TSRX module (`.tsrx`), routed by its file kind.
#[inline]
pub(crate) fn is_tsrx_path(path: &str) -> bool {
    vize_l0::dialect::FileKind::of_path(std::path::Path::new(path))
        == Some(vize_l0::dialect::FileKind::Tsrx)
}

/// Authored non-JSX scripts use their existing native script diagnostic path.
#[inline]
pub(crate) fn is_plain_script_path(path: &str) -> bool {
    matches!(
        path.rsplit_once('.').map(|(_, extension)| extension),
        Some("js" | "mjs" | "cjs" | "ts" | "mts" | "cts")
    )
}
