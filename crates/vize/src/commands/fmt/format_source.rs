//! Select the native formatter for each source file.
use super::data;
use super::{Allocator, FormatOptions, FormatResult, VueVersion};
use oxc_span::SourceType;
use std::path::Path;
use vize_glyph::{GlyphFormatter, format_script_with_sort_imports};
use vize_l0::{Lang, dialect::FileKind, profile};

pub(super) fn format_file_source(
    path: &Path,
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
    formatting: (VueVersion, Option<&vize_glyph::ImportSortOptions>, bool),
) -> Result<FormatResult, vize_glyph::FormatError> {
    let (vue_version, sort_imports, preserve_template_whitespace) = formatting;
    if let Some(source_type) = script_source_type_for_path(path) {
        let code = profile!(
            "cli.fmt.file.format_script",
            format_script_with_sort_imports(source, options, allocator, source_type, sort_imports)
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
        GlyphFormatter::new_with_vue_version(options, allocator, vue_version)
            .with_sort_imports(sort_imports)
            .with_template_whitespace_preserved(preserve_template_whitespace)
            .format(source)
    )
}

fn script_source_type_for_path(path: &Path) -> Option<SourceType> {
    let script = FileKind::of_path(path)?.script()?;
    let source_type = match (script.lang, script.jsx) {
        (Lang::Js, false) => SourceType::from_path("module.js").ok()?,
        (Lang::Ts, false) => SourceType::from_path(path).ok()?,
        (Lang::Js, true) => SourceType::jsx(),
        (Lang::Ts, true) => SourceType::tsx(),
    };
    Some(source_type.with_module(true))
}
