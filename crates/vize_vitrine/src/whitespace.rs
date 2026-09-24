//! Decode the Vue compiler's public whitespace option at FFI boundaries.

use vize_atelier_core::WhitespaceStrategy;

pub(crate) fn resolve_whitespace(value: Option<&str>) -> Result<WhitespaceStrategy, String> {
    match value {
        None | Some("condense") => Ok(WhitespaceStrategy::Condense),
        Some("preserve") => Ok(WhitespaceStrategy::Preserve),
        Some(value) => Err(format!(
            "Invalid whitespace `{value}`. Expected `condense` or `preserve`."
        )),
    }
}
