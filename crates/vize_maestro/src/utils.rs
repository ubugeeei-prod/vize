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

/// Returns true when a standalone HTML document appears to use petite-vue.
#[inline]
pub fn is_petite_vue_document(content: &str) -> bool {
    content.contains("petite-vue")
        || content.contains("PetiteVue")
        || content.contains("v-scope")
        || content.contains("v-effect")
}
