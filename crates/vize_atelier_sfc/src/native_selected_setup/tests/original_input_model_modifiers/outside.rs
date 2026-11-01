//! Invoke each unchanged original observer, retaining its entire current array.

use serde_json::{Value, json};

pub(super) fn capture() -> (Value, Vec<String>) {
    let (dom, dom_failures) = super::super::original_input_model::refusal::capture();
    let mut failures: Vec<String> = dom_failures.into_iter().map(String::from).collect();
    let (ssr, ssr_failures) =
        crate::native_ssr_setup::tests::original_input_model::refusal::capture();
    let (vapor, vapor_failures) =
        crate::native_vapor_setup::tests::original_input_model::refusal::capture();
    failures.extend(ssr_failures.into_iter().map(String::from));
    failures.extend(vapor_failures.into_iter().map(String::from));
    if dom.len() != 24 || ssr.len() != 12 || vapor.len() != 12 {
        failures.push("all 48 original compatibility outside calls retained".into());
    }
    (json!({"dom":dom,"ssr":ssr,"vapor":vapor}), failures)
}
