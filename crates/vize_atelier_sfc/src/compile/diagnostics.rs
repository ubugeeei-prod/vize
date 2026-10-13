//! Warning constructors emitted by the SFC compile pipeline.

use vize_l0::{String, ToCompactString};

use crate::types::SfcError;

pub(super) fn create_v_model_reactive_const_warning(
    script_setup: &crate::types::SfcScriptBlock<'_>,
    binding_name: &str,
) -> SfcError {
    let mut message = String::from("`v-model` cannot update the const reactive binding `");
    message.push_str(binding_name);
    message.push_str("`. The compiler transformed it to `let` so the update can work.");

    SfcError {
        message,
        code: Some("V_MODEL_CONST_REACTIVE_DEMOTED".to_compact_string()),
        loc: Some(script_setup.loc.clone()),
    }
}
