//! Compilation requires a component block; descriptor parsing stays tolerant.

use crate::types::{SfcDescriptor, SfcError};
use vize_l0::String;

pub(super) fn validate_component(descriptor: &SfcDescriptor<'_>) -> Result<(), SfcError> {
    if descriptor.template.is_none()
        && descriptor.script.is_none()
        && descriptor.script_setup.is_none()
    {
        return Err(missing_block());
    }
    Ok(())
}

pub(super) fn missing_block() -> SfcError {
    SfcError {
        message: String::from(
            "At least one <template> or <script> is required in a single file component.",
        ),
        code: Some("SFC_MISSING_BLOCK".into()),
        loc: None,
    }
}
