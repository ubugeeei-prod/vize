//! A returned SFC result can carry compilation failures alongside partial code.

use vize_atelier_sfc::{SfcCompileResult, SfcError};

pub(in crate::commands::build::runner) fn check_compile_result(
    result: SfcCompileResult,
) -> Result<SfcCompileResult, SfcError> {
    let Some(mut error) = result.errors.first().cloned() else {
        return Ok(result);
    };
    error.message = result
        .errors
        .iter()
        .map(|error| error.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .into();
    Err(error)
}
