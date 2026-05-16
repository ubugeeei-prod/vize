//! Virtual TypeScript generation for Vue SFC type checking.
//!
//! This module delegates to the canonical `virtual_ts` module in vize_canon.
//! The canonical implementation provides proper scope hierarchy and structured
//! source mappings (VizeMapping).

use vize_carton::String;

/// Generate virtual TypeScript using croquis scope information.
///
/// Delegates to `crate::virtual_ts::generate_virtual_ts_with_offsets` which
/// is the canonical implementation used by the CLI and all other consumers.
pub fn generate_virtual_ts_with_scopes(
    summary: &vize_croquis::Croquis,
    script_content: Option<&str>,
    script_offset: u32,
    template_ast: Option<&vize_relief::ast::RootNode<'_>>,
    template_offset: u32,
) -> String {
    let output = crate::virtual_ts::generate_virtual_ts_with_offsets(
        summary,
        script_content,
        template_ast,
        script_offset,
        template_offset,
        &crate::virtual_ts::VirtualTsOptions::default(),
    );
    output.code
}
