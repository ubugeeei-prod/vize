//! Listeners that compile to one `onX` prop.

use vize_carton::{String, camelize, capitalize, cstr};
use vize_croquis::croquis::{ComponentUsage, EventListener};

use super::super::context::ComponentPropsContext;
use super::super::event_handler::event_name_source_range;
use crate::virtual_ts::helpers::push_ts_string_literal;
use crate::virtual_ts::types::VizeMapping;

/// `@foo-bar` and `@fooBar` compile to the same `onFooBar` listener prop, so
/// binding both on one usage is a collision Vue resolves silently (the last
/// one wins) and `vue-tsc` reports as a duplicate object key. Listeners are
/// grouped by that prop name plus their modifiers: `@foo-bar.up` next to
/// `@fooBar.down` is two distinct listeners and stays legal. Each key of a
/// colliding group maps to its own authored event name so the duplicate-key
/// error lands on the binding that repeats it.
pub(super) fn append_duplicate_listener_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_, '_>,
    usage: &ComponentUsage,
) {
    let mut groups: Vec<(String, Vec<&EventListener>)> = Vec::new();
    for event in &usage.events {
        if event.name_is_dynamic || event.name.is_empty() {
            continue;
        }
        // `v-model` records its `update:` listener like an authored one; the
        // authored `@update:model-value` beside it is the documented way to
        // observe the model, not a collision (upstream keys by directive).
        if is_v_model_listener(ctx.template_source, event) {
            continue;
        }
        let mut key = cstr!("on{}", capitalize(&camelize(event.name.as_str())));
        for modifier in &event.modifiers {
            key.push('+');
            key.push_str(modifier.as_str());
        }
        match groups.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, events)) => events.push(event),
            None => groups.push((key, vec![event])),
        }
    }
    for (key, events) in groups {
        if events.len() < 2 {
            continue;
        }
        let prop_name = key.split('+').next().unwrap_or(key.as_str());
        ts.push_str("  void { ");
        for event in events {
            let key_start = ts.len();
            push_ts_string_literal(ts, prop_name);
            let key_end = ts.len();
            ts.push_str(": 0, ");
            if let Some(src_range) = event_name_source_range(
                ctx.template_source,
                ctx.template_offset,
                event.start..event.end,
                event.name.as_str(),
            ) {
                mappings.push(VizeMapping {
                    gen_range: key_start..key_end,
                    src_range,
                    sub_spans: Vec::new(),
                });
            }
        }
        ts.push_str("};  // listeners that compile to one prop\n");
    }
}

/// Whether Croquis derived this listener from a `v-model` directive rather
/// than from an authored `@`/`v-on` binding.
fn is_v_model_listener(template_source: Option<&str>, event: &EventListener) -> bool {
    template_source
        .and_then(|source| source.get(event.start as usize..event.end as usize))
        .is_some_and(|directive| directive.starts_with("v-model"))
}
