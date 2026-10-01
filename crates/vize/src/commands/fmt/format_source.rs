//! Select the native formatter for each source file.
use super::data;
use super::{Allocator, FormatOptions, FormatResult, VueVersion};
use oxc_span::SourceType;
use std::path::Path;
use vize_glyph::{format_script_with_source_type, format_sfc_with_allocator_and_vue_version};
use vize_l0::profile;

pub(super) fn format_file_source(
    path: &Path,
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
    vue_version: VueVersion,
) -> Result<FormatResult, vize_glyph::FormatError> {
    if let Some(source_type) = script_source_type_for_path(path) {
        let code = profile!(
            "cli.fmt.file.format_script",
            format_script_with_source_type(source, options, allocator, source_type)
        )?;
        return Ok(FormatResult {
            changed: code.as_str() != source,
            code,
        });
    }

    if let Some(result) = data::format_data_file(path, source, options) {
        return result;
    }
    profile!(
        "cli.fmt.file.format_sfc",
        format_sfc_with_allocator_and_vue_version(source, options, allocator, vue_version)
    )
}

fn script_source_type_for_path(path: &Path) -> Option<SourceType> {
    let extension = path.extension().and_then(|extension| extension.to_str())?;
    match extension {
        "js" | "mjs" | "cjs" => Some(SourceType::from_path("module.js").ok()?.with_module(true)),
        "ts" | "mts" | "cts" => Some(SourceType::from_path(path).ok()?.with_module(true)),
        "jsx" => Some(SourceType::jsx().with_module(true)),
        "tsx" => Some(SourceType::tsx().with_module(true)),
        _ => None,
    }
}
