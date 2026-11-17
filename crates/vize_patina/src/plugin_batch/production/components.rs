//! Exact owned component-use records from the registered production table.
#![expect(clippy::disallowed_methods, reason = "JSON uses std strings")]
use serde_json::{Value, json};
use vize_croquis::facts::{ComponentIdentity, GroupedComponentUse};

pub fn project((key, sites): (&ComponentIdentity, &Vec<GroupedComponentUse>)) -> Value {
    let sites: Vec<Value> = sites.iter().map(|site| {
        let usage = site.usage.as_ref().map(|usage| {
            let props: Vec<Value> = usage.props.iter().map(|prop| json!({
                "name": prop.name.as_str(), "nameIsDynamic": prop.name_is_dynamic,
                "value": prop.value.as_deref(), "start": prop.start, "end": prop.end, "isDynamic": prop.is_dynamic,
            })).collect();
            let events: Vec<Value> = usage.events.iter().map(|event| json!({
                "name": event.name.as_str(), "nameIsDynamic": event.name_is_dynamic,
                "handler": event.handler.as_deref(), "modifiers": event.modifiers.iter().map(|value| value.as_str()).collect::<Vec<_>>(),
                "start": event.start, "end": event.end,
            })).collect();
            let slots: Vec<Value> = usage.slots.iter().map(|slot| json!({
                "name": slot.name.as_str(), "nameIsDynamic": slot.name_is_dynamic,
                "scopeVars": slot.scope_vars.iter().map(|value| value.as_str()).collect::<Vec<_>>(),
                "start": slot.start, "end": slot.end, "hasScope": slot.has_scope,
            })).collect();
            let spreads: Vec<Value> = usage.spread_props.iter().map(|spread| json!({
                "expression": spread.expression.as_str(), "start": spread.start, "end": spread.end,
            })).collect();
            json!({"name": usage.name.as_str(), "start": usage.start, "end": usage.end,
                "props": props, "events": events, "slots": slots, "hasSpreadAttrs": usage.has_spread_attrs,
                "spreadProps": spreads, "scopeId": usage.scope_id.as_u32(), "vifGuard": usage.vif_guard.as_deref()})
        });
        json!({"tag": site.tag.as_str(), "usage": usage, "usageIndex": site.usage_index, "nameOrdinal": site.name_ordinal})
    }).collect();
    let identity = json!([key.module.as_deref(), key.export_name.as_str()]).to_string();
    json!([identity, {"module": key.module.as_deref(), "export": key.export_name.as_str(), "sites": sites}])
}
