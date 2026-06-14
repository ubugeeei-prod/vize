//! Section slicing for compiled template output.

use vize_carton::String;

use crate::compile::output_module::{AtelierModuleSections, AtelierOutputSections, OutputRange};

#[cfg(test)]
mod tests;

/// Slice the structural sections out of compiled template code using
/// emission-recorded byte offsets.
///
/// The codegen pipeline already knows where each section starts and ends, so
/// this is slicing plus a trim pass over the tiny asset-resolution region.
pub(crate) fn slice_template_parts(
    template_code: &str,
    sections: &AtelierOutputSections,
) -> (String, String, String, String, &'static str) {
    let slice = |range: OutputRange| {
        template_code
            .get(range.start..range.end)
            .unwrap_or_default()
    };

    let imports = String::new(slice(sections.imports));
    let hoisted = String::new(slice(sections.hoisted));

    // Asset-resolution statements carry the render function's indentation;
    // the inline assembly expects them trimmed, one per line. The region also
    // ends with the blank separator line codegen emits before `return`.
    let assets_raw = slice(sections.assets);
    let mut preamble = String::with_capacity(assets_raw.len());
    for line in assets_raw.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            preamble.push_str(trimmed);
            preamble.push('\n');
        }
    }

    // Inline assembly wants the returned expression without codegen's optional
    // trailing semicolon.
    let mut body = slice(sections.return_expr);
    body = body.trim_end_matches([' ', '\t', '\n', '\r']);
    if let Some(stripped) = body.strip_suffix(';') {
        body = stripped;
    }
    let render_body = String::new(body);

    (imports, hoisted, preamble, render_body, "render")
}

/// Slice full-module template parts from coarse SFC Atelier output chunks.
///
/// SSR and Vapor register imports, hoists, and the full render function as
/// separate chunks. Inline script assembly expects a trailing newline after the
/// render function, so the slice is normalized to that shape without scanning
/// the generated JavaScript.
pub(crate) fn slice_template_parts_full(
    template_code: &str,
    sections: &AtelierModuleSections,
    render_fn_name: &'static str,
) -> (String, String, String, &'static str) {
    let slice = |range: OutputRange| {
        template_code
            .get(range.start..range.end)
            .unwrap_or_default()
    };

    let imports = String::new(slice(sections.imports));
    let hoisted = String::new(slice(sections.hoists));
    let mut render_fn = String::new(slice(sections.functions));
    if !render_fn.is_empty() && !render_fn.ends_with('\n') {
        render_fn.push('\n');
    }

    (imports, hoisted, render_fn, render_fn_name)
}
