//! Exact generated successors for the generated-name spelling: every
//! `__vize_*` value, key, marker and projection-local type of the
//! authenticated predecessor is spelled `$name` (`vize_l0::generated_name`),
//! and nothing else changes.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/generated-names-generated.json";
const RECORD_SHA: &str = "393aba365fe6ec77b1265509d1954f0d11b72edef5015863f0b7dacca2a2baf4";

/// Whole old names and their spellings.
const STATIC: &[(&str, &str)] = &[
    ("__vize_resolved_runtime_default", "$runtimeDefault"),
    ("__vize_strict_template_context", "$strictCtx"),
    ("__vize_keyed_template_context", "$keyedCtx"),
    ("__vize_jsx_component_spread__", "$jsxSpread"),
    ("__vize_plain_script_exports", "$scriptExports"),
    ("__vize_jsx_component_slot__", "$jsxSlot"),
    ("__vize_with_defaults_props", "$defaultedProps"),
    ("__vize_emit_closed_factory", "$EmitClosedFactory"),
    ("__vize_typed_route_global", "$typedRoute"),
    ("__vize_template_instance", "$inst"),
    ("__vize_css_module_global", "$cssModule"),
    ("__vize_emit_native_input", "$EmitNativeGate"),
    ("__vize_incomplete_member", "$incomplete"),
    ("__vize_unknown_reference", "$unknown"),
    ("__vize_instance_globals", "$globals"),
    ("__vize_options_instance", "$this"),
    ("__vize_template_handler", "$handlerHost"),
    ("__vize_public_instance", "$public"),
    ("__vize_jsx_component__", "$jsxComponent"),
    ("__vize_template_slots", "$templateSlots"),
    ("__vize_emit_calls_nav", "$emitCalls"),
    ("__vize_model_defaults", "$modelDefaults"),
    ("__vize_template_hover", "$hover"),
    ("__vize_options_props", "$optionsProps"),
    ("__vize_typed_exposed", "$typedExposed"),
    ("__vize_template_emit", "$templateEmit"),
    ("__vize_emit_resolver", "$EmitResolver"),
    ("__vize_async_setup__", "$asyncSetup"),
    ("__vize_emit_options", "$emitOptions"),
    ("__vize_template_use", "$use"),
    ("__vize_setup_props", "$setupProps"),
    ("__vize_typed_slots", "$typedSlots"),
    ("__vize_typed_emits", "$typedEmits"),
    ("__vize_model_value", "$value"),
    ("__vize_emit_closed", "$EmitClosed"),
    ("__vize_model_event", "$modelEvent"),
    ("__vize_jsx_expr__", "$jsx"),
    ("__vize_emit_input", "$EmitGate"),
    ("__vize_plain_host", "$plainHost"),
    ("__vize_defaults__", "$defaults"),
    ("__vize_forwarded", "$forwarded"),
    ("__vize_handler__", "$handlerBody"),
    ("__vize_default__", "$defaultValue"),
    ("__vize_template", "$template"),
    ("__vize_listener", "$listener"),
    ("__vize_root_el", "$rootEl"),
    ("__vize_exposed", "$exposed"),
    ("__vize_default", "$default"),
    ("__vize_handler", "$handlerBody"),
    ("__vize_generic", "$typeParams"),
    ("__vize_runtime", "$runtime"),
    ("__vize_spread", "$spread"),
    ("__vize_pragma", "$pragma"),
    ("__vize_props", "$propsObj"),
    ("__vize_emits", "$emits"),
    ("__vize_event", "$evt"),
    ("__vize_scope", "$scope"),
    ("__vize_value", "$iterable"),
    ("__vize_refs", "$refMap"),
    ("__vize_args", "$args"),
    ("__vize_src", "$src"),
    ("__vize_cb", "$cb"),
    ("__vize_v", "$item"),
];

/// Old family prefixes and their stems; a stem ending in `_` keeps the tail
/// as written, any other stem joins a numeric tail directly.
const FAMILY: &[(&str, &str)] = &[
    ("__vize_model_events_completion_", "$modelCompletion"),
    ("__vize_component_ref_check_", "$refCheck"),
    ("__vize_native_prop_check_", "$nativeProp"),
    ("__vize_unknown_directive_", "$unknownDirective"),
    ("__vize_patina_reactivity_", "$reactivity"),
    ("__vize_kebab_events_nav_", "$kebabNav"),
    ("__vize_model_events_nav_", "$modelNav"),
    ("__vize_global_component_", "$globalComponent"),
    ("__vize_events_resolved_", "$resolvedEvents"),
    ("__vize_directive_check_", "$directive"),
    ("__vize_slot_props_host_", "$slotPropsHost"),
    ("__vize_shipped_default_", "$shippedDefault"),
    ("__vize_events_resolver_", "$EventsResolver"),
    ("__vize_forwarded_props_", "$forwardedProps"),
    ("__vize_forwarded_emits_", "$forwardedEmits"),
    ("__vize_required_slots_", "$requiredSlots"),
    ("__vize_inline_handler_", "$inlineHandler"),
    ("__vize_boolean_target_", "$booleanTarget_"),
    ("__vize_patina_promise_", "$promise"),
    ("__vize_slot_children_", "$slotChildren"),
    ("__vize_fragment_prop_", "$fragmentProp"),
    ("__vize_exported_type_", "$exportedType_"),
    ("__vize_slot_payload_", "$slotPayload"),
    ("__vize_v_for_source_", "$forSource"),
    ("__vize_emit_factory_", "$emitFactory"),
    ("__vize_setup_import_", "$import"),
    ("__vize_legacy_props_", "$LegacyProps"),
    ("__vize_patina_prop_", "$propShape"),
    ("__vize_patina_emit_", "$emitCheck"),
    ("__vize_legacy_prop_", "$LegacyProp"),
    ("__vize_events_nav_", "$eventsNav"),
    ("__vize_event_keys_", "$eventKeys"),
    ("__vize_prop_check_", "$prop"),
    ("__vize_slot_props_", "$slotProps"),
    ("__vize_slot_scope_", "$slotScope"),
    ("__vize_emit_props_", "$emitProps"),
    ("__vize_native_key_", "$nativeKey"),
    ("__vize_dynamic_is_", "$is"),
    ("__vize_props_nav_", "$propsNav"),
    ("__vize_slots_nav_", "$slotsNav"),
    ("__vize_slot_host_", "$slotHost"),
    ("__vize_slot_name_", "$slotName"),
    ("__vize_slot_nav_", "$slotNav"),
    ("__vize_computed_", "$computed_"),
    ("__vize_handler_", "$handler"),
    ("__vize_generic_", "$generic"),
    ("__vize_ambient_", "$ambient"),
    ("__vize_method_", "$method_"),
    ("__vize_match_", "$match"),
    ("__vize_ref_", "$ref"),
];

fn record() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-generated-name-spelling-successor");
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn spell(token: &str) -> String {
    if let Some((_, new)) = STATIC.iter().find(|(old, _)| *old == token) {
        return (*new).to_owned();
    }
    for (old, new) in FAMILY {
        if let Some(tail) = token.strip_prefix(old) {
            return if new.ends_with('_') || tail.is_empty() || tail.as_bytes()[0].is_ascii_digit() {
                format!("{new}{tail}")
            } else {
                format!("{new}_{tail}")
            };
        }
    }
    token.to_owned()
}

/// The predecessor with every old generated name respelled.
pub(super) fn respell(current: &str) -> String {
    let bytes = current.as_bytes();
    let mut out = String::with_capacity(current.len());
    let mut at = 0;
    while let Some(found) = current[at..].find("__vize_") {
        let start = at + found;
        if start > 0 && is_word(bytes[start - 1]) {
            out.push_str(&current[at..start + 7]);
            at = start + 7;
            continue;
        }
        let mut end = start + 7;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }
        out.push_str(&current[at..start]);
        out.push_str(&spell(&current[start..end]));
        at = end;
    }
    out.push_str(&current[at..]);
    out
}

fn successor_body(current: &str, entry: &Value) -> String {
    assert_eq!(json!(current.len()), entry["before"][1]);
    assert_eq!(support::hash(current.as_bytes()), entry["before"][2]);
    let successor = respell(current);
    assert_ne!(successor, current);
    assert_eq!(
        (
            json!(successor.len()),
            json!(support::hash(successor.as_bytes()))
        ),
        (entry["after"][1].clone(), entry["after"][2].clone()),
        "{}",
        entry["after"][0]
    );
    successor
}

pub(super) fn apply(manifest: &mut Value, members: &mut [Value]) {
    let record = record();
    let entries = record["entries"].as_array().unwrap();
    // B.vue.ts spells no renamed name, so only A.vue.ts has a successor.
    assert_eq!(entries.len(), 1);
    for (index, entry) in entries.iter().enumerate() {
        let member_index = index + 74;
        assert_eq!(entry["entry"], member_index);
        assert_eq!(entry["after"][0], entry["before"][0]);
        let member = members.get_mut(member_index).unwrap();
        assert_eq!(*member, entry["before"]);
        *member = entry["after"].clone();
    }
    manifest["provenance"]["currentGeneratedNamesGenerated"] = record;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_generated_successors_respell_the_authenticated_predecessors() {
        let record = record();
        let bodies = super::super::fallthrough_source_generated::current_bodies();
        let entries = record["entries"].as_array().unwrap();
        assert_eq!(bodies.len(), entries.len() + 1);
        let unchanged = bodies.last().unwrap();
        assert_eq!(
            &respell(unchanged),
            unchanged,
            "B.vue.ts has no renamed name"
        );
        for (body, entry) in bodies.iter().zip(entries) {
            successor_body(body, entry);
            assert!(
                std::panic::catch_unwind(|| successor_body(&format!("{body}\n"), entry)).is_err()
            );
        }
    }
}
