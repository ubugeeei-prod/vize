//! Explicit current outside successor; complete older positive packs stay frozen.
use serde_json::Value;
use sha2::{Digest, Sha256};
use vize_l0::{String, cstr};

pub(crate) fn expected(family: &str) -> Result<Value, String> {
    let expected_len = match family {
        "dom" => 24,
        "ssr" | "vapor" => 12,
        "trim" => 36,
        _ => return Err("closed original observer family".into()),
    };
    let expected: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_input_model_current_outside.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let digest = Sha256::digest(include_bytes!(
        "../../../tests/fixtures/native_original_input_model_modifier_inputs.json"
    ));
    let mut input_hash = String::with_capacity(64);
    for nibble in digest.iter().flat_map(|byte| [byte >> 4, byte & 15]) {
        input_hash.push(char::from_digit(u32::from(nibble), 16).ok_or("SHA256 nibble")?);
    }
    if expected["schema"] != "vize.native-sfc.input-model-current-outside"
        || expected["inputHash"] != input_hash.as_str()
        || expected["provenance"]["independentlyReviewedBeforeFreeze"] != true
        || expected["arrays"][family]
            .as_array()
            .is_none_or(|rows| rows.len() != expected_len)
    {
        return Err("whole current outside authority/denominator mismatch".into());
    }
    Ok(expected["arrays"][family].clone())
}
