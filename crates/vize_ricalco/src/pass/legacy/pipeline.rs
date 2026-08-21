//! The per-dialect legacy pipeline and its runner (P2-9 series 7;
//! split from `pass.rs` under the source budget — and reached only
//! through the cfg-gated [`super`] module, so none of this exists
//! without the `_legacy` feature).

use vize_davinci::pass::{PassDesc, PassFailure, PassObserver, Pipeline, run_pipeline};

use super::super::{S2_STAGE, S2Facts, hoist, legacy, text, vfor, vif, vmodel, vslot};
use crate::lower::Lowered;

/// The per-dialect pipeline a **legacy** artifact runs (P2-9 series 7):
/// the plain [`TRANSFORM`] passes plus the legacy pass, appended last.
/// The pass set is dialect-determined exactly as the shipped lane's is
/// (`TransformOptions` arms `hoist_static` the same way); keeping the
/// plain pipeline untouched is what holds every `walks=6` pin — and the
/// zero-cost clause — byte-identical across both feature shapes.
///
/// [`TRANSFORM`]: super::super::TRANSFORM
pub const TRANSFORM_LEGACY_PASSES: &[PassDesc] = &[
    vif::DESC,
    vfor::DESC,
    vslot::DESC,
    text::DESC,
    vmodel::DESC,
    hoist::DESC,
    legacy::DESC,
];

/// The planned legacy pipeline over [`TRANSFORM_LEGACY_PASSES`].
pub const TRANSFORM_LEGACY: Pipeline = Pipeline::new(S2_STAGE, TRANSFORM_LEGACY_PASSES);

// The legacy pipeline's fusion shape, pinned: the six plain groups
// unchanged, plus the legacy barrier as a seventh lone group after the
// fusable singleton (a barrier neighbour, so the hoist group stays a
// singleton and fusion still buys nothing).
const _: () = assert!(TRANSFORM_LEGACY.group_count() == 7);
const _: () = assert!(TRANSFORM_LEGACY.is_fully_serialized());
const _: () = {
    let group = match TRANSFORM_LEGACY.group(6) {
        Some(group) => group,
        None => panic!("the seventh group exists"),
    };
    assert!(group.start == 6 && group.len == 1 && group.is_barrier);
};

/// Run the **legacy** S2 pipeline ([`TRANSFORM_LEGACY`]) over a
/// [`crate::lower::lower_legacy`]-lowered artifact —
/// [`run_transform`]'s per-dialect twin (P2-9 series 7).
///
/// # Panics
///
/// As [`run_transform`], plus the legacy pass's site laws.
///
/// [`run_transform`]: super::super::run_transform
pub fn run_transform_legacy<'a, O: PassObserver>(
    lowered: &mut Lowered<'a>,
    observer: &mut O,
) -> S2Facts {
    let mut facts = S2Facts::default();
    #[cfg(debug_assertions)]
    let mut verify = vize_disegno::verify::VerifyObserver::new();

    let outcome = run_pipeline(&TRANSFORM_LEGACY, observer, |event| {
        let name = event.desc().name;
        if name == vif::DESC.name {
            facts.if_facts = vif::run(lowered);
        } else if name == vfor::DESC.name {
            facts.for_facts = vfor::run(lowered);
        } else if name == vslot::DESC.name {
            facts.slot_facts = vslot::run(lowered);
        } else if name == text::DESC.name {
            facts.text_facts = text::run(lowered);
        } else if name == vmodel::DESC.name {
            facts.model_faults = vmodel::run(lowered);
        } else if name == hoist::DESC.name {
            facts.static_facts = hoist::run(lowered);
        } else if name == legacy::DESC.name {
            facts.legacy = legacy::run(lowered);
        } else {
            return Err(PassFailure::new("pipeline pass has no registered body"));
        }

        // P2-6: verifier between passes, debug builds only — the
        // `run_transform` wiring plus the two filter tables.
        #[cfg(debug_assertions)]
        {
            verify.note(event);
            let folio = vize_disegno::folio::DisegnoFolio::of(&lowered.root.ops);
            verify.check(event, &folio);
            verify.check_table(event, &folio, &lowered.scopes);
            verify.check_table(event, &folio, &lowered.texts);
            verify.check_table(event, &folio, &lowered.wrappers);
            verify.check_table(event, &folio, &lowered.filters);
            verify.check_table(event, &folio, &facts.if_facts);
            verify.check_table(event, &folio, &facts.for_facts);
            verify.check_table(event, &folio, &facts.slot_facts);
            verify.check_table(event, &folio, &facts.text_facts);
            verify.check_table(event, &folio, &facts.model_faults);
            verify.check_table(event, &folio, &facts.static_facts);
            verify.check_table(event, &folio, &facts.legacy.sites);
        }
        Ok(())
    });
    // The catalogue above is closed over the const pipeline, so a
    // failure here is a compiler bug, not an input property.
    if let Err(failure) = outcome {
        panic!("s2 legacy transform pipeline stopped: {}", failure.reason);
    }
    facts
}
