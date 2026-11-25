//! Shared parse-error message rendering for the binding boundaries.

use vize_atelier_core::{CompilerError, CompilerErrorWithSource};

/// The `Parse errors: [...]` message both bindings raise when a template
/// parse fails, with each error's covered source text rendered from its span.
///
/// Recoverable diagnostics, such as the rewrite of a self-closing `<div />`
/// that SFC templates accept or a duplicate attribute, do not fail the parse,
/// as they do not fail `compile`; `None` when only those were reported.
#[expect(
    clippy::disallowed_types,
    reason = "N-API and wasm-bindgen errors carry std `String`s"
)]
pub(crate) fn failure(errors: &[CompilerError], source: &str) -> Option<String> {
    if errors.iter().all(CompilerError::is_recoverable) {
        return None;
    }
    let fatal: Vec<CompilerError> = errors
        .iter()
        .filter(|error| !error.is_recoverable())
        .cloned()
        .collect();
    Some(
        vize_l0::cstr!(
            "Parse errors: {:?}",
            CompilerErrorWithSource::list(&fatal, source)
        )
        .into(),
    )
}
