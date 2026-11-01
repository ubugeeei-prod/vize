//! All original outside inputs remain whole current attempts, including lazy.

use serde_json::{Value, json};
use vize_atelier_sfc::{
    compile_native_selected_setup_sfc_dom, compile_native_setup_ssr_sfc,
    compile_native_vapor_setup_sfc,
};
use vize_l0::Allocator;

pub(super) fn capture() -> (Vec<Value>, Vec<String>) {
    let mut rows = Vec::new();
    let mut failures = Vec::new();
    let pack: Value = match serde_json::from_str(include_str!(
        "../fixtures/native_original_input_model_ssr_contexts_vue_3_5_35.json"
    )) {
        Ok(pack) => pack,
        Err(error) => return (rows, vec![format!("original refusal inputs: {error}")]),
    };
    let Some(inputs) = pack["refusals"].as_array() else {
        return (rows, vec!["original refusal array".into()]);
    };
    if inputs.len() != 12 {
        return (rows, vec!["exact twelve original refusal inputs".into()]);
    }
    for target in ["dom", "ssr", "vapor"] {
        for input in inputs {
            let Some(source) = input["source"].as_str() else {
                failures.push("original refusal source".into());
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
            let current_lazy = input["id"] == "modifier"
                && source
                    == "<script setup>let draft='x'</script><template><input v-model.lazy='draft'/></template>";
            let expected = if current_lazy { "emitted" } else { "refused" };
            if attempt["state"] != expected {
                failures.push(format!(
                    "{target}/{} differs from current {expected}",
                    input["id"]
                ));
            }
            rows.push(json!({"id":input["id"],"source":source,"target":target,"attempt":attempt}));
        }
    }
    (rows, failures)
}
