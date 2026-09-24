//! Shared parse-error message rendering for the binding boundaries.

use vize_atelier_core::{CompilerError, CompilerErrorWithSource};

/// The `Parse errors: [...]` message both bindings raise when a template
/// parse fails, with each error's covered source text rendered from its span.
#[expect(
    clippy::disallowed_types,
    reason = "N-API and wasm-bindgen errors carry std `String`s"
)]
pub(crate) fn message(errors: &[CompilerError], source: &str) -> String {
    vize_s0::cstr!(
        "Parse errors: {:?}",
        CompilerErrorWithSource::list(errors, source)
    )
    .into()
}
