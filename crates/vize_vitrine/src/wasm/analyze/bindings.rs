//! The `analyzeSfc` payload's bindings, read through a declared fact demand
//! (Davinci P4-3a).

use vize_croquis::Croquis;
use vize_croquis::facts::{Bindings, BindingsTable, CroquisFacts, Demand, FactConsumer, FactGroup};

use super::super::source_offsets::ScriptOffsetMapper;

/// The playground's analysis payload reads script bindings.
struct AnalyzePayload;

impl FactConsumer for AnalyzePayload {
    const NAME: &'static str = "vitrine/analyze-sfc";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

/// Every typed binding as JSON (name order, UTF-16 SFC spans), and whether
/// the bindings come from `<script setup>`.
pub(super) fn bindings_json(
    summary: &Croquis,
    source: &str,
    script_offset_mapper: ScriptOffsetMapper,
) -> (Vec<serde_json::Value>, bool) {
    let mut facts = CroquisFacts::new(summary);
    let view = facts.prepare::<AnalyzePayload>();
    let binding_facts = view.get::<Bindings>().expect("declared demand");
    let bindings = binding_facts
        .typed()
        .map(|(name, binding_type)| {
            let (start, end) = binding_facts
                .span(name)
                .map(|(start, end)| script_offset_mapper.to_utf16_range(source, start, end))
                .unwrap_or((0, 0));
            serde_json::json!({
                "name": name,
                "type": format!("{binding_type:?}"),
                "start": start,
                "end": end,
            })
        })
        .collect();
    (bindings, binding_facts.is_script_setup())
}
