//! Utility modules for vize_maestro.

pub mod position;

pub use position::{
    internal_to_lsp_position, line_range, make_range, offset_to_position, offset_to_position_str,
    position_to_offset, position_to_offset_str, source_location_to_range,
};

/// Returns true for standalone HTML files that should be linted outside the SFC pipeline.
#[inline]
pub fn is_standalone_html_path(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    path.ends_with(".html") || path.ends_with(".htm")
}

/// Returns true for JSX/TSX files that should be compiled outside the SFC
/// pipeline. These surface JSX compiler/lowering diagnostics only — no virtual
/// TypeScript document is generated for them (see #1497).
#[inline]
pub fn is_jsx_path(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    path.ends_with(".jsx") || path.ends_with(".tsx")
}
