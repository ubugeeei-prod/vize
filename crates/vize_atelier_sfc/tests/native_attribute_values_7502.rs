//! Private complete-output capture; an unfrozen artifact is always a failure.
use vize_l0::{String, ToCompactString};
mod native_attribute_values_7502 {
    use super::{Test, check};

    pub(super) mod capture;
    pub(super) mod custody;
    pub(super) mod envelopes;
    pub(super) mod inputs;
    pub(super) mod refusals;
    pub(super) mod successor;
}
use native_attribute_values_7502::{capture, envelopes, inputs, refusals, successor};

type Test<T = ()> = Result<T, String>;

fn check(condition: bool, context: &str) -> Test {
    if condition {
        Ok(())
    } else {
        Err(String::from(context))
    }
}

#[test]
fn complete_original_attribute_value_outputs_require_independent_review() -> Test {
    let inputs = inputs::load()?;
    let first = capture::packet(&inputs);
    // Retain every actual outcome before either repeat or frozen-output gates.
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ATTRIBUTE_VALUES_7502_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&first).map_err(|error| error.to_compact_string())?,
        )
        .map_err(|error| error.to_compact_string())?;
    }
    let second = capture::packet(&inputs);
    check(first == second, "complete fresh captures differ")?;
    refusals::validate(&first)?;
    successor::unchanged(&first)?;
    let reviewed: serde_json::Value = serde_json::from_slice(include_bytes!(
        "fixtures/native_attribute_values_7502/reviewed_output_v2.json"
    ))
    .map_err(|error| error.to_compact_string())?;
    check(
        reviewed.as_object().is_some_and(|object| {
            object.len() == 4
                && object.get("schema").and_then(serde_json::Value::as_str)
                    == Some("vize.native-attribute-values-7502.reviewed-output")
                && object.get("version").and_then(serde_json::Value::as_u64) == Some(2)
                && object.get("state").and_then(serde_json::Value::as_str) == Some("reviewed")
                && object
                    .get("capture")
                    .is_some_and(serde_json::Value::is_object)
        }),
        "complete output is absent or unfrozen; independent paired review is required",
    )?;
    check(
        reviewed.get("capture") == Some(&first),
        "complete reviewed output differs",
    )
}

#[test]
fn independent_script_and_style_carriers_preserve_original_refusals() -> Test {
    let inputs = inputs::load()?;
    let first = envelopes::packet(&inputs);
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ATTRIBUTE_VALUES_7502_CAPTURE") {
        let mut envelope_path = path;
        envelope_path.push(".envelopes.json");
        std::fs::write(
            envelope_path,
            serde_json::to_vec_pretty(&first).map_err(|error| error.to_compact_string())?,
        )
        .map_err(|error| error.to_compact_string())?;
    }
    check(
        first == envelopes::packet(&inputs),
        "fresh carrier refusals differ",
    )?;
    envelopes::validate(&first)
}
