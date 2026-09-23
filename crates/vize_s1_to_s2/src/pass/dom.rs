//! DOM-emitter transform product builder.
//!
//! The public pass-manager path remains in `pass.rs`; this module holds the
//! DOM-specific shortcut that folds preserving products before codegen.

use vize_davinci::pass::{PassDesc, PassFailure, PassObserver, Pipeline, run_pipeline};

use crate::lower::Lowered;

use super::{S2_STAGE, S2Facts, TransformProfile, hoist, legacy, text, vfor, vif, vmodel, vslot};

const DOM_LEGACY_PASSES: &[PassDesc] = &[legacy::DESC];
const DOM_LEGACY_TRANSFORM: Pipeline = Pipeline::new(S2_STAGE, DOM_LEGACY_PASSES);

/// Build the transform products consumed by the DOM emitter.
///
/// The full `run_transform_with_profile` path stays the pass-manager review
/// surface. DOM emission has already selected a single source-map-free S2
/// product path, so the preserving fact products are folded before the
/// code-producing walk instead of spending observer-visible transform walks.
/// Vue 2 legacy sugar is the exception: it mutates the S2 surface and still
/// runs through the pass manager so the observer records that compatibility
/// walk.
pub fn run_dom_transform_with_profile<'a, O: PassObserver>(
    lowered: &mut Lowered<'a>,
    observer: &mut O,
    profile: TransformProfile,
) -> S2Facts {
    let mut facts = S2Facts {
        if_facts: vif::facts_from_lowering(lowered),
        for_facts: vfor::facts_from_lowering(lowered),
        text_facts: text::facts_from_lowering(lowered),
        ..S2Facts::default()
    };

    if lowered.caps.needs_sugar() {
        run_dom_legacy_transform(lowered, observer, &mut facts);
    }

    if lowered.features.has_slot_carriers() {
        facts.slot_facts = vslot::run(lowered);
    }
    if lowered.features.has_model_bindings() {
        facts.model_faults = vmodel::run(lowered);
    }
    if profile.includes_static_analysis() {
        facts.static_facts = hoist::run(lowered);
    }

    facts
}

fn run_dom_legacy_transform<'a, O: PassObserver>(
    lowered: &mut Lowered<'a>,
    observer: &mut O,
    facts: &mut S2Facts,
) {
    #[cfg(debug_assertions)]
    let mut verify = vize_s2::verify::VerifyObserver::new();

    let outcome = run_pipeline(&DOM_LEGACY_TRANSFORM, observer, |event| {
        if event.desc().name != legacy::DESC.name {
            return Err(PassFailure::new("pipeline pass has no registered body"));
        }
        facts.legacy = legacy::run(lowered);
        facts.if_facts = vif::facts_from_lowering(lowered);
        facts.for_facts = vfor::facts_from_lowering(lowered);
        facts.text_facts = text::facts_from_lowering(lowered);

        #[cfg(debug_assertions)]
        {
            verify.note(event);
            let folio = vize_s2::folio::S2Folio::of(&lowered.root.ops);
            verify.check(event, &folio);
            verify.check_table(event, &folio, &lowered.scopes);
            verify.check_table(event, &folio, &lowered.texts);
            verify.check_table(event, &folio, &lowered.if_facts);
            verify.check_table(event, &folio, &lowered.for_facts);
            verify.check_table(event, &folio, &lowered.wrappers);
            verify.check_table(event, &folio, &lowered.for_wrappers);
            verify.check_table(event, &folio, &facts.if_facts);
            verify.check_table(event, &folio, &facts.for_facts);
            verify.check_table(event, &folio, &facts.slot_facts);
            verify.check_table(event, &folio, &facts.text_facts);
            verify.check_table(event, &folio, &facts.model_faults);
            verify.check_table(event, &folio, &facts.static_facts);
        }

        Ok(())
    });
    if let Err(failure) = outcome {
        lowered.diagnostics.push(crate::exemptions::lowering(
            vize_s0::Span::new(0, 0),
            failure.reason,
        ));
    }
}
