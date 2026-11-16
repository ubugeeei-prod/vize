//! Private transport for an already-evaluated host configuration.
#![expect(
    clippy::disallowed_types,
    reason = "the existing configuration loader API returns std String errors"
)]

use std::{io::Read, path::PathBuf};
use vize_l0::cstr;

fn read() -> Result<Vec<u8>, std::string::String> {
    let mut payload = Vec::new();
    std::io::stdin()
        .lock()
        .read_to_end(&mut payload)
        .map_err(|error| cstr!("failed to read config from stdin: {error}").into_string())?;
    Ok(payload)
}

fn source_path() -> PathBuf {
    std::env::temp_dir().join(".vize-vp-stdin.json")
}

pub(super) fn project() -> Result<crate::config::LoadedProjectConfig, std::string::String> {
    crate::config::try_load_project_config_from_serialized(&read()?, source_path())
}

#[cfg(feature = "glyph")]
pub(super) fn formatter() -> Result<crate::config::LoadedFormatterSnapshot, std::string::String> {
    crate::config::try_load_formatter_snapshot_from_serialized(&read()?, source_path())
}
