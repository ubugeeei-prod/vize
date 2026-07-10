//! Output contract shared by the Vapor adapter and SFC inline assembly.
//!
//! Vapor has a target-specific IR and renderer, so this adapter does not force
//! it to pretend that its output has DOM render-body sections. The SFC boundary
//! only needs a flattened JavaScript module plus coarse byte ranges for the
//! plates that script-setup inline mode moves around: imports, hoists, the full
//! render function, and exports.

use vize_carton::{String, ToCompactString};
use vize_rendu::RenduRange;

use crate::compile::output_module::AtelierModuleSections;

/// Vapor template output after adapter normalization.
///
/// `code` is byte-equivalent JavaScript after runtime import rewriting, scope
/// attribute injection, and render signature normalization. `module_sections`
/// describes the coarse plates in that same string so callers can slice the
/// output directly rather than rediscovering structure with the legacy scanner.
pub(super) struct VaporTemplateModule {
    pub(super) code: String,
    pub(super) module_sections: AtelierModuleSections,
}

/// Build coarse module sections for normalized Vapor output.
///
/// The blank separator before the render function is intentionally outside the
/// hoist range. Inline assembly moves hoists as declarations, while the
/// separator remains a formatting boundary in the flattened output.
pub(super) fn vapor_module_sections(
    imports_len: usize,
    hoists_len: usize,
    separator_len: usize,
    functions_len: usize,
) -> AtelierModuleSections {
    let imports = RenduRange::new(0, imports_len);
    let hoists = RenduRange::new(imports.end, imports.end + hoists_len);
    let functions_start = hoists.end + separator_len;
    let functions = RenduRange::new(functions_start, functions_start + functions_len);
    let exports = RenduRange::empty(functions.end);

    AtelierModuleSections {
        imports,
        hoists,
        functions,
        exports,
    }
}

/// Rewrite Vapor runtime imports onto the public Vue runtime surface.
pub(super) fn rewrite_vapor_import(line: &str) -> String {
    if line.contains("'vue/vapor'") {
        line.replace("'vue/vapor'", "'vue'").into()
    } else if line.contains("\"vue/vapor\"") {
        line.replace("\"vue/vapor\"", "\"vue\"").into()
    } else {
        line.to_compact_string()
    }
}

/// Detect render signatures emitted by current and legacy Vapor codegen.
pub(super) fn is_render_signature(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("export function render(")
        || trimmed.starts_with("function render(")
        || trimmed.starts_with("export default")
}
