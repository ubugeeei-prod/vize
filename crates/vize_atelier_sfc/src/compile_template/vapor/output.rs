//! Output contract shared by the Vapor adapter and SFC inline assembly.
//!
//! Vapor has a target-specific IR and renderer, so this adapter does not force
//! it to pretend that its output has DOM render-body sections. The SFC boundary
//! only needs a flattened JavaScript module plus coarse byte ranges for the
//! plates that script-setup inline mode moves around: imports, hoists, the full
//! render function, and exports.

use vize_atelier_core::atelier_output::AtelierRange;
use vize_carton::{String, ToCompactString};

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
    let imports = AtelierRange::new(0, imports_len);
    let hoists = AtelierRange::new(imports.end, imports.end + hoists_len);
    let functions_start = hoists.end + separator_len;
    let functions = AtelierRange::new(functions_start, functions_start + functions_len);
    let exports = AtelierRange::empty(functions.end);

    AtelierModuleSections {
        imports,
        hoists,
        functions,
        exports,
    }
}

/// Rewrite Vapor runtime imports onto the adapter-selected runtime surface.
pub(super) fn rewrite_vapor_import(line: &str, runtime_module_name: &str) -> String {
    let (source, quote) = if line.contains("'vue/vapor'") {
        ("'vue/vapor'", '\'')
    } else if line.contains("\"vue/vapor\"") {
        ("\"vue/vapor\"", '"')
    } else if line.contains("'vue'") {
        ("'vue'", '\'')
    } else if line.contains("\"vue\"") {
        ("\"vue\"", '"')
    } else {
        return line.to_compact_string();
    };
    let replacement = vize_carton::cstr!("{quote}{runtime_module_name}{quote}");
    line.replace(source, replacement.as_str()).into()
}

/// Detect render signatures emitted by current and legacy Vapor codegen.
pub(super) fn is_render_signature(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("export function render(")
        || trimmed.starts_with("function render(")
        || trimmed.starts_with("export default")
}
