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
            serde_json::from_str::<RawVizeConfig>(&content)?
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
