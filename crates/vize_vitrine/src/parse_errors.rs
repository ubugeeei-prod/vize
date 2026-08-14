//! Shared parse-error message rendering for the binding boundaries.

use vize_atelier_core::{CompilerError, CompilerErrorWithSource};

/// The `Parse errors: [...]` message both bindings raise when a template
/// parse fails, with each error's covered source text rendered from its span.
pub(crate) fn message(errors: &[CompilerError], source: &str) -> String {
    format!(
        "Parse errors: {:?}",
        CompilerErrorWithSource::list(errors, source)
    )
}
