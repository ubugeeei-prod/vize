//! The L2 transform passes over lowered Vue output — the P2-9 series
//! substrate.
//!
//! # Ownership
//!
//! Vue dialect legalization and lowering adapters stay in this conversion
//! crate. Backend-neutral analyses, including static-subtree and template
//! complexity, execute in `vize_l2` over its canonical regions. The current
//! transform plan still selects them and publishes their facts beside the
//! transitional lowering result.
//!
//! # Driving a run
//!
//! [`run_transform`] executes the artifact-selected L2 plan through the
//! P2-2 pass manager (`vize_l0::pass::run_pipeline`) with a
//! caller-supplied observer, and wires the P2-6 [`VerifyObserver`] between
//! passes in debug builds exactly as its module documents: `note` then
//! `check` / `check_table` after every pass, so a broken invariant names
//! the pass that broke it. Release builds make zero verifier calls
//! (guardrail 5). Passes that make optimization decisions explain them
//! through the same observer (P3-13 remarks, `hoist-static` first); an
//! observer that consumes none compiles the explanation away.

use vize_l0::pass::{
    PassDesc, PassEvent, PassFailure, PassObserver, Pipeline, run_pipeline_remarked,
};
use vize_l2::analysis::pass_facts::L2Facts;

use crate::lower::Lowered;

pub mod cfg;
mod dom;
pub mod hoist;
pub mod legacy;
mod plan;
pub mod vfor;
pub mod vif;
pub mod vmodel;
pub mod vslot;
pub(crate) mod walk;

pub use cfg::{ComplexityFacts, Contribution, DecisionKind};
pub use dom::run_dom_transform_with_profile;
pub use hoist::{StaticFacts, StaticLevel};
pub use plan::TransformProfile;
pub use vfor::{ForFacts, ForName};
pub use vif::{BranchKey, BranchKeyKind, IfFacts};
pub use vmodel::{ModelFacts, ModelFault};
pub use vslot::{SlotCarrier, SlotFacts, SlotGroup, SlotName, SlotParams};

/// The stage name L2 transform pipelines print and parse under.
// Schema-1 attribution remains stable across physical layer renames.
pub const L2_STAGE: &str = "s2";

/// The L2 transform pipeline as the series has built it so far.
///
/// Four passes ([`vslot::DESC`], [`vmodel::DESC`], [`hoist::DESC`], then
/// [`cfg::DESC`]). Text, `v-for`, and `v-if` facts are now
/// lowering-published on the canonical artifact (no L2 walk and no
/// copy), so the transform table only contains passes
/// that still need one. Later installments append here, and the `const`
/// pins below are the grouping regression guard (the P2-2 convention: a
/// fusion-plan change is a compile error, not a surprise).
pub const TRANSFORM_PASSES: &[PassDesc] = &[vslot::DESC, vmodel::DESC, hoist::DESC, cfg::DESC];

/// The planned pipeline over [`TRANSFORM_PASSES`].
pub const TRANSFORM: Pipeline = Pipeline::new(L2_STAGE, TRANSFORM_PASSES);

// The plan's fusion shape, pinned: two mandatory barriers plus one
// NON-BARRIER group holding both `Optional`/`Fusable` analyses —
// `hoist-static` (installment 6) and `template-complexity` (P4-9a), the
// pipeline's first real fusion: four passes, three walks. Text, `v-for`,
// and `v-if` facts no longer cost a transform group.
const _: () = assert!(TRANSFORM.group_count() == 3);
const _: () = assert!(!TRANSFORM.is_fully_serialized());
const _: () = {
    let group = match TRANSFORM.group(2) {
        Some(group) => group,
        None => panic!("the third group exists"),
    };
    assert!(group.start == 2 && group.len == 2 && !group.is_barrier);
};

/// Run the artifact-selected L2 transform pipeline over `lowered`, firing
/// `observer`'s hooks around each pass.
///
/// Pass diagnostics and provenance append to `lowered`'s own channels —
/// the unified-channel design; there is no second diagnostics stream.
/// In debug builds the L2 verifier runs between passes ([`VerifyObserver`]
/// wiring per its docs); a violated invariant panics naming the pass.
///
/// # Panics
///
/// Panics only on a compiler bug: a pipeline pass with no registered
/// body, or (debug builds) a verifier violation.
///
/// [`VerifyObserver`]: vize_l2::verify::VerifyObserver
pub fn run_transform<'a, O: PassObserver>(lowered: &mut Lowered<'a>, observer: &mut O) -> L2Facts {
    run_transform_with_profile(lowered, observer, TransformProfile::DEFAULT)
}

/// [`run_transform`] under a product-selected optional-pass profile.
pub fn run_transform_with_profile<'a, O: PassObserver>(
    lowered: &mut Lowered<'a>,
    observer: &mut O,
    profile: TransformProfile,
) -> L2Facts {
    run_transform_with_pass_hook(lowered, observer, profile, |_, _| {})
}

/// [`run_transform_with_profile`] with an artifact hook after every pass.
///
/// Observer hooks carry no artifact (`DumpObserver`'s contract: whoever
/// holds the artifact prints it), so a consumer that pages the L2 tree
/// per pass - a `Collector`, the stage ladder - receives each
/// [`PassEvent`] here together with the post-pass lowering, after the pass
/// body and (debug builds) the between-pass verifier ran. The hook only
/// observes: the plan, the pass bodies and the returned facts are those of
/// [`run_transform_with_profile`], which is this function with a no-op hook.
pub fn run_transform_with_pass_hook<'a, O, H>(
    lowered: &mut Lowered<'a>,
    observer: &mut O,
    profile: TransformProfile,
    mut after_pass: H,
) -> L2Facts
where
    O: PassObserver,
    H: FnMut(&PassEvent<'_>, &Lowered<'a>),
{
    let mut facts = L2Facts::default();
    #[cfg(debug_assertions)]
    let mut verify = vize_l2::verify::VerifyObserver::new();

    let pipeline = plan::pipeline_for_profile(lowered.caps, lowered.artifact.features, profile);
    let outcome = run_pipeline_remarked(&pipeline, observer, |event, remarks| {
        let name = event.desc().name;
        if name == legacy::DESC.name {
            facts.filters = legacy::run(lowered);
        } else if name == vslot::DESC.name {
            facts.slot_facts = vslot::run(lowered);
        } else if name == vmodel::DESC.name {
            facts.model_faults = vmodel::run(lowered);
        } else if name == hoist::DESC.name {
            facts.static_facts = hoist::run_remarked(lowered, remarks);
        } else if name == cfg::DESC.name {
            facts.complexity = Some(cfg::run(lowered));
        } else {
            return Err(PassFailure::new("pipeline pass has no registered body"));
        }

        // P2-6: verifier between passes, debug builds only. `note` first
        // (rigor escalates after a mandatory-lowering pass), then the
        // tree checks and one `check_table` per side table that exists.
        #[cfg(debug_assertions)]
        {
            verify.note(event);
            let folio = vize_l2::dump::Page::of(&lowered.artifact.root.ops);
            verify.check(event, &folio);
            verify.check_table(event, &folio, &lowered.artifact.scopes);
            verify.check_table(event, &folio, &lowered.artifact.texts);
            verify.check_table(event, &folio, &lowered.artifact.if_facts);
            verify.check_table(event, &folio, &lowered.artifact.for_facts);
            verify.check_table(event, &folio, &lowered.artifact.branch_fragments);
            verify.check_table(event, &folio, &lowered.artifact.loop_fragments);
            verify.check_table(event, &folio, &facts.slot_facts);
            verify.check_table(event, &folio, &facts.model_faults);
            verify.check_table(event, &folio, &facts.static_facts);
        }
        after_pass(event, lowered);
        Ok(())
    });
    // The catalogue above is closed over the const pipeline, so a failure
    // here is a compiler bug, not an input property. It surfaces as a
    // lowering diagnostic so the caller sees the failure instead of a crash.
    if let Err(failure) = outcome {
        lowered.diagnostics.push(crate::exemptions::lowering(
            vize_l0::Span::new(0, 0),
            failure.reason,
        ));
    }
    facts
}

#[cfg(test)]
mod tests {
    use super::{L2_STAGE, TRANSFORM, TRANSFORM_PASSES, cfg, hoist, vif, vmodel, vslot};
    use vize_l0::pass::{Fusability, PassKind, Preserved};

    #[test]
    fn the_pipeline_holds_exactly_the_landed_passes() {
        assert_eq!(TRANSFORM.stage, L2_STAGE);
        assert_eq!(TRANSFORM_PASSES.len(), 4);
        assert_eq!(TRANSFORM_PASSES[0], vslot::DESC);
        assert_eq!(TRANSFORM_PASSES[1], vmodel::DESC);
        assert_eq!(TRANSFORM_PASSES[2], hoist::DESC);
        assert_eq!(TRANSFORM_PASSES[3], cfg::DESC);
    }

    #[test]
    fn the_vif_mirror_keeps_the_recorded_name() {
        assert_eq!(vif::NAME, "v-if");
    }

    #[test]
    fn the_vslot_classification_is_pinned() {
        // The review-point classification, pinned so a drive-by re-kind
        // is a loud diff: see `vslot::DESC`'s docs for the reasoning
        // (again a *preserving* mandatory pass — installment 2's
        // recorded taxonomy tension, in the milder diagnosing form).
        assert_eq!(vslot::DESC.name, "v-slot");
        assert_eq!(vslot::DESC.kind, PassKind::MandatoryLowering);
        assert_eq!(vslot::DESC.fusability, Fusability::Barrier);
    }

    #[test]
    fn the_vmodel_classification_is_pinned() {
        // The review-point classification, pinned so a drive-by re-kind
        // is a loud diff: see `vmodel::DESC`'s docs for the reasoning —
        // the series' FIRST `MandatoryDiagnostic` (the pass preserves
        // everything and its whole product is diagnostics plus the
        // fault record; nothing canonicalizes).
        assert_eq!(vmodel::DESC.name, "v-model");
        assert_eq!(vmodel::DESC.kind, PassKind::MandatoryDiagnostic);
        assert_eq!(vmodel::DESC.fusability, Fusability::Barrier);
    }

    #[test]
    fn the_hoist_classification_is_pinned() {
        // The review-point classification, pinned so a drive-by re-kind
        // is a loud diff: see `hoist::DESC`'s docs for the reasoning —
        // the series' FIRST `Optional` (skipping loses optimization
        // facts only; the shipped lane's own `hoist_static: false`
        // default is the proof) and FIRST `Fusable` (a synthesized-
        // attribute analysis, single-visit and local).
        assert_eq!(hoist::DESC.name, "hoist-static");
        assert_eq!(hoist::DESC.kind, PassKind::Optional);
        assert_eq!(hoist::DESC.fusability, Fusability::Fusable);
        assert_eq!(hoist::DESC.preserved, Preserved::ALL);
    }

    #[test]
    fn the_complexity_classification_is_pinned() {
        // See `cfg::DESC`'s docs: a pure analysis over a shared borrow
        // that no emitter reads, one pre-order visit.
        assert_eq!(cfg::DESC.name, "template-complexity");
        assert_eq!(cfg::DESC.kind, PassKind::Optional);
        assert_eq!(cfg::DESC.fusability, Fusability::Fusable);
        assert_eq!(cfg::DESC.preserved, Preserved::ALL);
    }

    #[test]
    fn the_fusion_plan_is_two_lone_barriers_plus_one_fused_analysis_group() {
        // The review point's fusion question, answered as data: the
        // two mandatory passes fuse with nothing (law 1), and the two
        // fusable analyses share the first NON-barrier group — the
        // plan's first real fusion (four passes, three walks). Text,
        // `v-for`, and `v-if` facts are no longer transform passes at
        // all.
        for index in 0..2 {
            let group = TRANSFORM.group(index).expect("group exists");
            assert!(group.is_barrier && group.len == 1);
        }
        let fusable = TRANSFORM.group(2).expect("the third group exists");
        assert!(!fusable.is_barrier);
        assert_eq!((fusable.start, fusable.len), (2, 2));
        assert!(fusable.preserved == Preserved::ALL);
        assert_eq!(TRANSFORM.group(3), None);
        assert_eq!(TRANSFORM.group_count(), 3);
        assert!(!TRANSFORM.is_fully_serialized());
    }
}
