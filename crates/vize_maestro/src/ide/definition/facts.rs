//! Go-to-definition's declared fact demands (Davinci P4-3a): definition
//! spans come from the `Bindings` group, never from `Croquis` fields.

use vize_croquis::Croquis;
use vize_croquis::facts::{Bindings, BindingsTable, CroquisFacts, Demand, FactConsumer, FactGroup};

/// Script go-to-definition on an analyzed (class / Options API) binding.
struct ScriptBindingDefinition;

impl FactConsumer for ScriptBindingDefinition {
    const NAME: &'static str = "maestro/script-definition";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

/// Template go-to-definition on a component prop declared in its script.
struct TemplatePropDefinition;

impl FactConsumer for TemplatePropDefinition {
    const NAME: &'static str = "maestro/template-prop-definition";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

fn span_of<C: FactConsumer>(croquis: &Croquis, name: &str) -> Option<(u32, u32)> {
    let mut facts = CroquisFacts::new(croquis);
    let view = facts.prepare::<C>();
    view.get::<Bindings>().expect("declared demand").span(name)
}

/// The definition span of script binding `name`.
pub(super) fn binding_span(croquis: &Croquis, name: &str) -> Option<(u32, u32)> {
    span_of::<ScriptBindingDefinition>(croquis, name)
}

/// The definition span of the prop binding `name` in a component's script.
pub(super) fn prop_span(croquis: &Croquis, name: &str) -> Option<(u32, u32)> {
    span_of::<TemplatePropDefinition>(croquis, name)
}
