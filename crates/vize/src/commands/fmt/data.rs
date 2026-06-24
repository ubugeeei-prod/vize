//! Formatting for non-SFC data files (currently JSON and JSONC).

use std::path::Path;
use vize_carton::profile;
use vize_glyph::{FormatOptions, FormatResult};

/// Format `source` as JSON or JSONC based on `path`.
///
/// `.jsonc` files and comment-bearing config files (`tsconfig*.json`,
/// `jsconfig*.json`, `.vscode/*.json`, `*.code-workspace`) are formatted as
/// JSONC so their comments and trailing commas survive; every other `.json`
/// file uses strict JSON. Returns `None` when `path` is neither `.json` nor
/// `.jsonc`, letting the caller fall through to the SFC formatter.
pub(super) fn format_data_file(
    path: &Path,
    source: &str,
    options: &FormatOptions,
) -> Option<Result<FormatResult, vize_glyph::FormatError>> {
    let jsonc = match path.extension().and_then(|extension| extension.to_str())? {
        "jsonc" => true,
        "json" => is_jsonc_config_file(path),
        _ => return None,
    };
    let code = if jsonc {
        profile!(
            "cli.fmt.file.format_jsonc",
            vize_glyph::format_jsonc(source, options)
        )
    } else {
        profile!(
            "cli.fmt.file.format_json",
            vize_glyph::format_json(source, options)
        )
    };
    Some(code.map(|code| FormatResult {
        changed: code.as_str() != source,
        code,
    }))
}

/// Whether a `.json` file conventionally allows comments and trailing commas
/// and should be formatted as JSONC, matching Prettier's `jsonc` parser
/// defaults for TypeScript and editor config files.
fn is_jsonc_config_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if name.ends_with(".code-workspace") {
        return true;
    }
    if (name.starts_with("tsconfig.") || name.starts_with("jsconfig.")) && name.ends_with(".json") {
        return true;
    }
    path.parent()
        .and_then(Path::file_name)
        .and_then(|parent| parent.to_str())
        == Some(".vscode")
}

#[cfg(test)]
mod tests {
    use super::is_jsonc_config_file;
    use std::path::Path;

    #[test]
    fn treats_typescript_and_editor_config_json_as_jsonc() {
        for path in [
            "tsconfig.json",
            "tsconfig.app.json",
            "packages/ui/tsconfig.build.json",
            "jsconfig.json",
            "jsconfig.node.json",
            "acme.code-workspace",
            ".vscode/settings.json",
            "repo/.vscode/launch.json",
        ] {
            assert!(
                is_jsonc_config_file(Path::new(path)),
                "expected JSONC: {path}"
            );
        }
    }

    #[test]
    fn keeps_plain_json_strict() {
        for path in [
            "package.json",
            "tsconfig.json.bak",
            "src/data/tsconfig.json/oops",
            "settings.json",
            "config/eslintrc.json",
        ] {
            assert!(
                !is_jsonc_config_file(Path::new(path)),
                "expected strict JSON: {path}"
            );
        }
    }
}
