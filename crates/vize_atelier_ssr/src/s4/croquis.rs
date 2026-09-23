//! Whether a Croquis summary is reproducible on the non-inline SSR plan.
//!
//! The legacy transform reads the summary in two places
//! (`vize_atelier_core::lane::context`): reactivity facts, which change
//! `.value` / `_unref` only for an inline render closure, and component
//! registration. Non-inline SSR refuses inline up front, so the plan keeps
//! the metadata binding table and admits the summary only when registration
//! adds nothing that table lacks. This is the same registration check the
//! DOM lane uses (`vize_atelier_dom::compile::croquis_facts`).

use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_croquis::Croquis;
use vize_croquis::facts::{Bindings, BindingsTable, CroquisFacts, Demand, FactConsumer, FactGroup};

/// SSR projection reads script bindings through the declared fact.
struct SsrCroquisProjection;

impl FactConsumer for SsrCroquisProjection {
    const NAME: &'static str = "ssr/croquis-projection";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

pub(super) fn projectable(croquis: &Croquis, metadata: Option<&BindingMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    if !croquis.used_components.is_empty() {
        return false;
    }
    let mut facts = CroquisFacts::new(croquis);
    let bindings = facts
        .prepare::<SsrCroquisProjection>()
        .get::<Bindings>()
        .expect("declared demand");
    bindings
        .typed()
        .all(|(name, kind)| kind != BindingType::SetupConst || metadata.bindings.contains_key(name))
}
