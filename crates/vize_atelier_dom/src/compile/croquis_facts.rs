//! The Croquis summary projected onto the S2 binding table (P3-17).
//!
//! The legacy transform lane consumes an attached Croquis summary in exactly
//! two places (`vize_atelier_core::lane::context`):
//!
//! - `get_reactive_kind` — the reactivity tracker's `lookup(name)` — decides
//!   the inline-mode `.value` / `_unref` reads in the identifier collector and
//!   the simple-identifier fallback. That is projected as
//!   [`ReactiveRead`] facts, name for name, onto the table the S2 transform
//!   rewrite reads.
//! - `is_component_registered` — Croquis `used_components` or a Croquis
//!   `SetupConst` binding — promotes a lowercase non-native tag to a
//!   component. The S2 lane resolves components from the binding metadata
//!   alone, so the projection is exact only when the summary registers
//!   nothing the metadata lacks; otherwise the compile stays on the legacy
//!   lane under the `croquis` reason.
//!
//! Nothing else reads the summary during a DOM compile (the scope-chain and
//! binding-type helpers on `TransformContext` that consult it have no callers
//! in the transform or codegen lanes).

use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_croquis::Croquis;
use vize_croquis::facts::{Bindings, BindingsTable, CroquisFacts, Demand, FactConsumer, FactGroup};
use vize_s1_to_s2::{BindingTable, ReactiveRead};

use crate::options::DomCompilerOptions;

/// Whether `options` carries a Croquis summary the S2 lane cannot
/// reproduce from binding metadata (P3-17). A projectable summary is
/// admitted: non-inline DOM reads the metadata table plus the summary's
/// reactivity facts, and the production-path oracle holds those compiles
/// to the legacy module byte for byte.
pub(in crate::compile) fn unprojectable_croquis(options: &DomCompilerOptions) -> bool {
    options
        .croquis
        .as_deref()
        .is_some_and(|croquis| !projectable(croquis, options.binding_metadata.as_ref()))
}

/// The binding table the S2 emitter reads: the script's binding metadata
/// plus, when a Croquis summary is attached, its reactivity facts.
pub(in crate::compile) fn s2_binding_table_for(
    options: &DomCompilerOptions,
) -> Option<BindingTable> {
    let table = super::stage_options::s2_binding_table(options.binding_metadata.as_ref())?;
    Some(match options.croquis.as_deref() {
        Some(croquis) => with_reactive_reads(table, croquis),
        None => table,
    })
}

/// DOM projection reads script bindings through the declared fact.
struct DomCroquisProjection;

impl FactConsumer for DomCroquisProjection {
    const NAME: &'static str = "dom/croquis-projection";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

/// Whether every legacy consumption of `croquis` is reproducible from
/// `metadata` plus the projected reactivity facts.
pub(super) fn projectable(croquis: &Croquis, metadata: Option<&BindingMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    if !vize_croquis::facts::used_components_empty(croquis) {
        return false;
    }
    let mut facts = CroquisFacts::new(croquis);
    let bindings = facts
        .prepare::<DomCroquisProjection>()
        .get::<Bindings>()
        .expect("declared demand");
    bindings
        .typed()
        .all(|(name, kind)| kind != BindingType::SetupConst || metadata.bindings.contains_key(name))
}

/// Attach each name's last lattice registration to `table`.
pub(super) fn with_reactive_reads(table: BindingTable, croquis: &Croquis) -> BindingTable {
    let sources = vize_croquis::facts::reactivity_sources(croquis);
    table.with_reactive_reads(sources.iter().map(|source| {
        let read = if source.kind.needs_value_access() {
            ReactiveRead::Value
        } else {
            ReactiveRead::Direct
        };
        (source.name.as_str(), read)
    }))
}

#[cfg(test)]
mod tests {
    use super::{projectable, with_reactive_reads};
    use vize_atelier_core::options::{BindingMetadata, BindingType};
    use vize_croquis::Croquis;
    use vize_croquis::reactivity::ReactiveKind;
    use vize_s1_to_s2::{BindingTable, ReactiveRead};

    #[test]
    fn reactive_reads_follow_the_trackers_last_registration() {
        let mut croquis = Croquis::default();
        croquis
            .reactivity
            .register("count".into(), ReactiveKind::Ref, 0);
        croquis
            .reactivity
            .register("state".into(), ReactiveKind::Reactive, 1);
        croquis
            .reactivity
            .register("count".into(), ReactiveKind::Readonly, 2);
        croquis
            .reactivity
            .register("total".into(), ReactiveKind::Computed, 3);
        let table = with_reactive_reads(BindingTable::new([], [], true), &croquis);
        assert_eq!(table.reactive_read("count"), Some(ReactiveRead::Direct));
        assert_eq!(table.reactive_read("state"), Some(ReactiveRead::Direct));
        assert_eq!(table.reactive_read("total"), Some(ReactiveRead::Value));
        assert_eq!(table.reactive_read("other"), None);
    }

    #[test]
    fn registration_beyond_the_metadata_is_not_projectable() {
        let mut metadata = BindingMetadata::default();
        metadata
            .bindings
            .insert("Child".into(), BindingType::SetupConst);
        let mut croquis = Croquis::default();
        croquis.bindings.add("Child", BindingType::SetupConst);
        assert!(projectable(&croquis, Some(&metadata)));
        assert!(!projectable(&croquis, None));

        croquis.bindings.add("Hidden", BindingType::SetupConst);
        assert!(!projectable(&croquis, Some(&metadata)));

        let mut drawn = Croquis::default();
        drawn.note_used_component("child");
        assert!(!projectable(&drawn, Some(&metadata)));
    }
}
