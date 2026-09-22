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
use vize_s1_to_s2::{BindingTable, ReactiveRead};

use crate::options::DomCompilerOptions;

/// Whether `options` carries a Croquis summary that keeps the compile on the
/// legacy lane under the `croquis` reason (P3-17). The projection is
/// byte-exact on the production-path oracle, but the S2 lane is measured
/// slower than the legacy lane on these compiles, so production keeps
/// refusing until that closes (charter #22); the differential lanes arm
/// the projection to hold its parity.
pub(in crate::compile) fn unprojectable_croquis(options: &DomCompilerOptions) -> bool {
    options.croquis.as_deref().is_some_and(|croquis| {
        !(croquis_projection_armed() && projectable(croquis, options.binding_metadata.as_ref()))
    })
}

#[cfg(feature = "davinci-differential")]
fn croquis_projection_armed() -> bool {
    super::selection::differential::croquis_projection()
}

#[cfg(not(feature = "davinci-differential"))]
const fn croquis_projection_armed() -> bool {
    false
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

/// Whether every legacy consumption of `croquis` is reproducible from
/// `metadata` plus the projected reactivity facts.
pub(super) fn projectable(croquis: &Croquis, metadata: Option<&BindingMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    croquis.used_components.is_empty()
        && croquis.bindings.iter().all(|(name, kind)| {
            kind != BindingType::SetupConst || metadata.bindings.contains_key(name)
        })
}

/// Attach the reactivity tracker's `lookup(name)` answers to `table`.
pub(super) fn with_reactive_reads(table: BindingTable, croquis: &Croquis) -> BindingTable {
    let tracker = &croquis.reactivity;
    table.with_reactive_reads(tracker.reactive_names().iter().filter_map(|name| {
        tracker.lookup(name).map(|source| {
            let read = if source.kind.needs_value_access() {
                ReactiveRead::Value
            } else {
                ReactiveRead::Direct
            };
            (name.as_str(), read)
        })
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
        drawn.used_components.insert("child".into());
        assert!(!projectable(&drawn, Some(&metadata)));
    }
}
