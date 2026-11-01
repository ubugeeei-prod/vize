//! New derived number boundaries; no historical refusal denominator is changed.

use crate::{
    compile_native_selected_setup_sfc_dom, compile_native_setup_ssr_sfc,
    compile_native_vapor_setup_sfc,
};
use serde_json::{Value, json};
use vize_l0::Allocator;

pub(super) const INPUTS: &str = include_str!(
    "../../../../tests/fixtures/native_original_input_model_number_refusal_inputs.json"
);

pub(super) fn capture() -> (Vec<Value>, Vec<String>) {
    let mut rows = Vec::new();
    let mut failures = Vec::new();
    let pack: Value = match serde_json::from_str(INPUTS) {
        Ok(value) => value,
        Err(error) => {
            return (
                rows,
                vec![format!("number derived outside inputs: {error}")],
            );
        }
    };
    let Some(inputs) = pack["cases"].as_array() else {
        return (rows, vec!["number outside inputs".into()]);
    };
    if inputs.len() != 12 {
        return (rows, vec!["exact twelve number outside inputs".into()]);
    }
    for target in ["dom", "ssr", "vapor"] {
        for input in inputs {
            let Some(source) = input["source"].as_str() else {
                failures.push("number outside source".into());
                continue;
            };
            let arena = Allocator::default();
            let attempt = match target {
                "dom" => super::output::observe(
                    &compile_native_selected_setup_sfc_dom(&arena, source, Default::default())
                        .result(),
                ),
                "ssr" => super::output::observe(
                    &compile_native_setup_ssr_sfc(&arena, source, Default::default()).result(),
                ),
                "vapor" => super::output::observe(
                    &compile_native_vapor_setup_sfc(&arena, source, Default::default()).result(),
                ),
                _ => {
                    failures.push("closed target set".into());
                    continue;
                }
            };
            if attempt["state"] != "refused" {
                failures.push(format!("{target}/{} must remain refused", input["id"]));
            }
            rows.push(json!({"id":input["id"],"source":source,"target":target,"attempt":attempt}));
        }
    }
    (rows, failures)
}
