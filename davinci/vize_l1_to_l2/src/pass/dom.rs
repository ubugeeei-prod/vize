//! DOM-emitter transform product builder.
//!
//! The public pass-manager path remains in `pass.rs`; this module holds the
//! DOM-specific shortcut that folds preserving products before codegen.

use vize_l0::pass::{PassDesc, PassFailure, PassObserver, Pipeline, run_pipeline};

use crate::lower::Lowered;
use vize_l2::provenance::ProvenanceSink;

use vize_l2::analysis::pass_facts::L2Facts;

use super::{L2_STAGE, TransformProfile, hoist, legacy, vmodel, vslot};

const DOM_LEGACY_PASSES: &[PassDesc] = &[legacy::DESC];
const DOM_LEGACY_TRANSFORM: Pipeline = Pipeline::new(L2_STAGE, DOM_LEGACY_PASSES);

/// Build the transform products consumed by the DOM emitter.
///
/// The full `run_transform_with_profile` path stays the pass-manager review
/// surface. DOM emission has already selected a single source-map-free L2
/// product path, so the preserving fact products are folded before the
/// code-producing walk instead of spending observer-visible transform walks.
/// Vue 2 legacy sugar is the exception: it mutates the L2 surface and still
/// runs through the pass manager so the observer records that compatibility
/// walk.
pub fn run_dom_transform_with_profile<'a, O: PassObserver>(
    lowered: &mut Lowered<'a, impl ProvenanceSink>,
    observer: &mut O,
    profile: TransformProfile,
) -> L2Facts {
    let mut facts = L2Facts::default();

    if lowered.caps.needs_sugar() {
        run_dom_legacy_transform(lowered, observer, &mut facts);
    }

    if lowered.artifact.features.has_slot_carriers() {
        facts.slot_facts = vslot::run(lowered);
    }
    if lowered.artifact.features.has_model_bindings() {
        facts.model_faults = vmodel::run(lowered);
    }
    if profile.includes_static_analysis() {
        facts.static_facts = hoist::run(lowered);
    }

    facts
}

fn run_dom_legacy_transform<'a, O: PassObserver>(
    lowered: &mut Lowered<'a, impl ProvenanceSink>,
    observer: &mut O,
    facts: &mut L2Facts,
) {
    #[cfg(debug_assertions)]
    let mut verify = vize_l2::verify::VerifyObserver::new();

    let outcome = run_pipeline(&DOM_LEGACY_TRANSFORM, observer, |event| {
        if event.desc().name != legacy::DESC.name {
            return Err(PassFailure::new("pipeline pass has no registered body"));
        }
        facts.filters = legacy::run(lowered);

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

        Ok(())
    });
    if let Err(failure) = outcome {
        lowered.diagnostics.push(crate::exemptions::lowering(
            vize_l0::Span::new(0, 0),
            failure.reason,
        ));
    }
}
