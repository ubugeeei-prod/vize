use std::path::Path;

use super::load_raw_config_with_source;

/// Load configured SFC Vapor mode; stable `compiler.vapor` wins over experimentals.
pub fn load_compiler_vapor(path: Option<&Path>) -> Option<bool> {
    let loaded = load_raw_config_with_source(path);
    loaded
        .config
        .compiler
        .vapor
        .or_else(|| loaded.config.experimentals.vapor_enabled().then_some(true))
}
