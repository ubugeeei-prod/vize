//! Formatting for non-SFC files: JSON/JSONC configs, YAML, and Markdown.

use std::path::Path;
use vize_glyph::{FormatOptions, FormatResult};
use vize_l0::profile;

mod plain;

/// Format `source` based on `path`'s extension.
///
/// `.yaml`/`.yml` and `.md`/`.markdown` get conservative, lossless
/// normalization (see [`plain`]). Every `.json` and `.jsonc` file uses JSONC
/// formatting so authored comments survive and trailing commas are accepted
/// on input. The public strict JSON formatter API retains its syntax policy.
/// Returns `None` for any other extension,
/// letting the caller fall through to the SFC formatter.
pub(super) fn format_data_file(
    path: &Path,
    source: &str,
    options: &FormatOptions,
) -> Option<Result<FormatResult, vize_glyph::FormatError>> {
    let extension = path.extension().and_then(|extension| extension.to_str())?;
    match extension {
        "yaml" | "yml" => return Some(Ok(plain::format_yaml(source))),
        "md" | "markdown" => return Some(Ok(plain::format_markdown(source))),
        _ => {}
    }
    if !matches!(extension, "json" | "jsonc") {
        return None;
    }
    let code = profile!(
        "cli.fmt.file.format_jsonc",
        vize_glyph::format_jsonc(source, options)
    );
    Some(code.map(|code| FormatResult {
        changed: code.as_str() != source,
        code,
    }))
}

#[cfg(test)]
mod tests {
    use super::format_data_file;
    use std::path::Path;
    use vize_glyph::FormatOptions;

    #[test]
    fn every_json_config_uses_the_complete_reported_jsonc_corpus() {
        let corpus: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../tests/_fixtures/differential/formatter-regressions/json-comments-7865/cases.json"
        )).unwrap();
        let cases = corpus["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 16);
        let options = FormatOptions::default();
        for case in cases {
            let path = Path::new(case["file"].as_str().unwrap());
            let source = case["source"].as_str().unwrap();
            let format = |source| format_data_file(path, source, &options).unwrap();
            if case["outcome"] == "error" {
                assert!(format(source).is_err(), "{path:?}");
                continue;
            }
            let expected = case["expected"].as_str().unwrap();
            let first = format(source).unwrap();
            assert_eq!(first.code.as_str(), expected, "{path:?}");
            assert_eq!(first.changed, source != expected, "{path:?}");
            for _ in 0..2 {
                let next = format(expected).unwrap();
                assert_eq!(next.code.as_str(), expected, "{path:?}");
                assert!(!next.changed, "{path:?}");
            }
            if case["strictRejected"] == true {
                assert!(
                    vize_glyph::format_json(source, &options).is_err(),
                    "strict API {path:?}"
                );
            }
        }
        for path in ["tsconfig.json.bak", "src/data/tsconfig.json/oops"] {
            assert!(format_data_file(Path::new(path), "{}", &options).is_none());
        }
    }
}
