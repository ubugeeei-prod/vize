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

pub(super) fn projectable(croquis: &Croquis, metadata: Option<&BindingMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    croquis.used_components.is_empty()
        && croquis.bindings.iter().all(|(name, kind)| {
            kind != BindingType::SetupConst || metadata.bindings.contains_key(name)
        })
}
