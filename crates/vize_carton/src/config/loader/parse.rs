//! Config format dispatch.
//!
//! Discovery feeds candidate paths into this module. It picks the concrete
//! reader from the file extension and implements the fallback rule used during
//! auto-discovery: malformed configs warn, while an unavailable PKL runtime can
//! continue to the next lower-priority config format.

use std::path::Path;

use crate::config::model::RawVizeConfig;

use super::{js::parse_js_config, pkl};

/// Parse a single config file path without discovery fallback.
pub(super) fn parse_raw_config_file(
    path: &Path,
) -> Result<RawVizeConfig, Box<dyn std::error::Error>> {
    let config = match path.extension().and_then(|ext| ext.to_str()) {
        Some("pkl") => pkl::parse_pkl_config(path)?,
        Some("ts" | "js" | "mjs") => parse_js_config(path)?,
        Some("json") => {
            let content = std::fs::read_to_string(path)?;
            // Share the same config deserializer as JS and PKL evaluation.
            // Keeping a separate str reader instantiates the entire model twice.
            serde_json::from_slice::<RawVizeConfig>(content.as_bytes())?
        }
        _ => return Ok(RawVizeConfig::default()),
    };

    Ok(config)
}

/// Parse a discovered candidate and map recoverable failures to `None`.
pub(super) fn try_parse_raw_candidate(path: &Path) -> Option<RawVizeConfig> {
    match parse_raw_config_file(path) {
        Ok(config) => Some(config),
        Err(error) => {
            let should_try_next = should_try_next_config(path, error.as_ref());
            eprintln!(
                "\x1b[33mWarning:\x1b[0m Failed to parse {}: {}",
                path.display(),
                error
            );
            if should_try_next {
                None
            } else {
                Some(RawVizeConfig::default())
            }
        }
    }
}

fn should_try_next_config(path: &Path, error: &(dyn std::error::Error + 'static)) -> bool {
    if path.extension().and_then(|ext| ext.to_str()) != Some("pkl") {
        return true;
    }

    pkl::is_process_error_box(error)
}

#[cfg(test)]
mod tests {
    use super::parse_raw_config_file;

    #[test]
    fn json_config_preserves_unicode_and_escaped_strings() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("vize.config.json");
        std::fs::write(
            &path,
            r#"{
                "basePath": "日本語/🎨",
                "files": ["src/\u65e5\u672c.vue"],
                "formatter": {
                    "attributeGroups": [["é", "data-\"quoted\""]]
                }
            }"#,
        )
        .unwrap();

        let config = parse_raw_config_file(&path).unwrap();
        assert_eq!(config.base_path.as_deref(), Some("日本語/🎨"));
        assert_eq!(config.files.unwrap()[0], "src/日本.vue");
        let groups = config.formatter.attribute_groups.unwrap();
        assert_eq!(groups[0][0], "é");
        assert_eq!(groups[0][1], "data-\"quoted\"");
    }

    #[test]
    fn json_config_rejects_invalid_utf8_before_deserialization() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("vize.config.json");
        std::fs::write(&path, b"{\"basePath\":\"\xff\"}").unwrap();

        let error = parse_raw_config_file(&path).unwrap_err();
        let error = error.downcast_ref::<std::io::Error>().unwrap();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn json_config_preserves_trailing_input_error_location() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("vize.config.json");
        std::fs::write(&path, "{}\r\n\r\n[]").unwrap();

        let error = parse_raw_config_file(&path).unwrap_err();
        let error = error.downcast_ref::<serde_json::Error>().unwrap();
        assert!(error.is_syntax());
        assert_eq!((error.line(), error.column()), (3, 1));
        assert_eq!(
            crate::cstr!("{error}"),
            "trailing characters at line 3 column 1"
        );
    }
}
