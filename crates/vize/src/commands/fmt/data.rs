//! Formatting for non-SFC data files (currently JSON and JSONC).

use std::path::Path;
use vize_carton::profile;
use vize_glyph::{FormatOptions, FormatResult};

/// Format `source` as JSON or JSONC based on `path`'s extension.
///
/// Returns `None` when `path` is neither `.json` nor `.jsonc`, letting the
/// caller fall through to the SFC formatter.
pub(super) fn format_data_file(
    path: &Path,
    source: &str,
    options: &FormatOptions,
) -> Option<Result<FormatResult, vize_glyph::FormatError>> {
    let code = match path.extension().and_then(|extension| extension.to_str())? {
        "json" => profile!(
            "cli.fmt.file.format_json",
            vize_glyph::format_json(source, options)
        ),
        "jsonc" => {
            profile!(
                "cli.fmt.file.format_jsonc",
                vize_glyph::format_jsonc(source, options)
            )
        }
        _ => return None,
    };
    Some(code.map(|code| FormatResult {
        changed: code.as_str() != source,
        code,
    }))
}
