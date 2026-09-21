//! The plan rule's suite: the static table against the rule, every
//! plan against the landed pipeline's order, and one case per decline.

use super::{
    COMPLEXITY_ANALYSIS, LEGACY_SUGAR, MODEL_BINDINGS, PLAN_COUNT, PLANS, SELECTABLE,
    SLOT_CARRIERS, STATIC_ANALYSIS, mask_for, pipeline_for_profile, plan_for_mask,
};
use crate::lower::{LegacyCaps, LoweringFeatures, OpFamily};
use crate::pass::{TRANSFORM, TRANSFORM_PASSES, TransformProfile, cfg, hoist, legacy, vmodel};

/// The profile every emitter runs under: neither optional analysis.
const EMIT: TransformProfile = TransformProfile::DEFAULT
    .without_static_analysis()
    .without_complexity_analysis();
use vize_s0::config::VueVersion;

const EVERY: [OpFamily; 5] = [
    OpFamily::If,
    OpFamily::For,
    OpFamily::SlotCarrier,
    OpFamily::TextCompound,
    OpFamily::Model,
];

fn every_family() -> LoweringFeatures {
    EVERY
        .into_iter()
        .fold(LoweringFeatures::EMPTY, LoweringFeatures::observing)
}

fn only(family: OpFamily) -> LoweringFeatures {
    LoweringFeatures::EMPTY.observing(family)
}

fn default_pipeline_for(
    caps: LegacyCaps,
    features: LoweringFeatures,
) -> vize_davinci::pass::Pipeline {
    pipeline_for_profile(caps, features, TransformProfile::DEFAULT)
}

/// Assert a pipeline's pass names, in order. Spelled as a zip rather
/// than a collected `Vec` so this suite owns no storage — the file is a
/// separate module, so `davinci-storage-policy`'s `cfg(test)` masking
/// does not reach it.
#[track_caller]
fn assert_names(pipeline: &vize_davinci::pass::Pipeline, expected: &[&str]) {
    assert_eq!(
        pipeline.passes.len(),
        expected.len(),
        "pass count: {:?} vs {expected:?}",
        pipeline.passes.iter().map(|pass| pass.name),
    );
    for (index, (pass, name)) in pipeline.passes.iter().zip(expected).enumerate() {
        assert_eq!(pass.name, *name, "pass {index}");
    }
}

#[test]
fn the_static_table_agrees_with_the_rule_for_every_mask() {
    for (mask, plan) in PLANS.iter().enumerate() {
        let computed = plan_for_mask(u8::try_from(mask).expect("PLAN_COUNT fits u8"));
        assert_eq!(plan.len, computed.len, "mask {mask} length");
        assert_eq!(
            plan.passes[..plan.len],
            computed.passes[..computed.len],
            "mask {mask} passes"
        );
    }
}

/// Every plan is a subsequence of the landed table, never a
/// reordering of it: declining a pass may not move another one.
#[test]
fn every_plan_is_a_subsequence_of_the_full_pipeline() {
    for (mask, plan) in PLANS.iter().enumerate() {
        let selected = &plan.passes[..plan.len];
        // The Vue 2 sugar pass leads or is absent; it is not part of the
        // Vue 3 table, so the order check below skips it.
        assert!(
            selected
                .iter()
                .skip(1)
                .all(|pass| pass.name != legacy::DESC.name),
            "mask {mask}: sugar may only lead",
        );
        let mut next = 0;
        for pass in selected
            .iter()
            .filter(|pass| pass.name != legacy::DESC.name)
        {
            let found = TRANSFORM_PASSES[next..]
                .iter()
                .position(|full| full.name == pass.name)
                .unwrap_or_else(|| panic!("mask {mask}: {} out of pipeline order", pass.name));
            next += found + 1;
        }
    }
}

/// One walk per pass, except that the two optional analyses share one
/// when both are selected: declining either alone saves a pass, not a
/// walk; declining both saves the walk.
#[test]
fn a_plans_walk_count_is_its_pass_count_minus_the_fused_analyses() {
    let fused = STATIC_ANALYSIS | COMPLEXITY_ANALYSIS;
    for (mask, plan) in PLANS.iter().enumerate() {
        let pipeline = plan.pipeline();
        let bits = u8::try_from(mask).expect("PLAN_COUNT fits u8");
        let saved = usize::from(bits & fused == fused);
        assert_eq!(pipeline.passes.len(), plan.len, "mask {mask} length");
        assert_eq!(
            pipeline.group_count(),
            plan.len - saved,
            "mask {mask} groups"
        );
    }
}

#[test]
fn vue3_with_every_family_is_the_landed_four_pass_table() {
    let pipeline = default_pipeline_for(LegacyCaps::VUE3, every_family());
    assert_eq!(pipeline, TRANSFORM);
    assert_eq!(pipeline.passes.len(), 4);
    assert_eq!(pipeline.group_count(), 3);
}

#[test]
fn vue3_omits_the_model_pass_when_lowering_found_no_model_ops() {
    let features = every_family();
    let pipeline = default_pipeline_for(
        LegacyCaps::VUE3,
        only(OpFamily::If)
            .observing(OpFamily::For)
            .observing(OpFamily::SlotCarrier),
    );
    assert!(features.has_model_bindings());
    assert_names(&pipeline, &["v-slot", hoist::NAME, cfg::NAME]);
    assert!(!pipeline.passes.iter().any(|pass| pass.name == vmodel::NAME));
}

#[test]
fn vue3_keeps_the_model_pass_when_diagnostics_may_need_it() {
    let pipeline = default_pipeline_for(LegacyCaps::VUE3, every_family());
    assert_eq!(pipeline.passes[1], vmodel::DESC);
}

/// The headline of this installment: an artifact with none of the
/// remaining transform-owned structural family pays neither its walk nor
/// the model pass's; `v-if`, `v-for`, and compound text are already
/// lowering-published, so the fused optional analyses are the only
/// transform walk left.
#[test]
fn vue3_omits_every_structural_pass_when_no_family_was_lowered() {
    let pipeline = default_pipeline_for(LegacyCaps::VUE3, LoweringFeatures::EMPTY);
    assert_names(&pipeline, &[hoist::NAME, cfg::NAME]);
    assert_eq!(pipeline.group_count(), 1);
}

#[test]
fn each_remaining_structural_family_buys_back_exactly_its_own_pass() {
    let cases = [(only(OpFamily::SlotCarrier), "v-slot")];
    for (features, expected) in cases {
        let pipeline = default_pipeline_for(LegacyCaps::VUE3, features);
        assert_names(&pipeline, &[expected, hoist::NAME, cfg::NAME]);
    }
}

#[test]
fn vue3_does_not_plan_lowering_published_fact_walks() {
    for family in [OpFamily::If, OpFamily::For, OpFamily::TextCompound] {
        let pipeline = default_pipeline_for(LegacyCaps::VUE3, only(family));
        assert_names(&pipeline, &[hoist::NAME, cfg::NAME]);
    }
}

#[test]
fn vue3_omits_both_analyses_when_an_emitter_cannot_use_them() {
    let pipeline = pipeline_for_profile(LegacyCaps::VUE3, every_family(), EMIT);
    assert_eq!(pipeline.passes.len(), 2);
    assert!(!pipeline.passes.iter().any(|pass| pass.name == hoist::NAME));
    assert!(!pipeline.passes.iter().any(|pass| pass.name == cfg::NAME));
    assert_eq!(pipeline.passes[1], vmodel::DESC);

    let bare = pipeline_for_profile(LegacyCaps::VUE3, LoweringFeatures::EMPTY, EMIT);
    assert_names(&bare, &[]);
    assert_eq!(bare.group_count(), 0);
}

#[test]
fn each_analysis_declines_independently_and_the_survivor_keeps_its_walk() {
    let without_static = TransformProfile::DEFAULT.without_static_analysis();
    let bare = pipeline_for_profile(LegacyCaps::VUE3, LoweringFeatures::EMPTY, without_static);
    assert_names(&bare, &[cfg::NAME]);
    assert_eq!(bare.group_count(), 1);

    let without_complexity = TransformProfile::DEFAULT.without_complexity_analysis();
    let bare = pipeline_for_profile(
        LegacyCaps::VUE3,
        LoweringFeatures::EMPTY,
        without_complexity,
    );
    assert_names(&bare, &[hoist::NAME]);
    assert_eq!(bare.group_count(), 1);
    assert!(without_complexity.includes_static_analysis());
    assert!(!without_complexity.includes_complexity_analysis());
}

#[test]
fn vue2_legacy_sugar_still_prepends_the_selected_vue3_shape() {
    let caps = LegacyCaps::for_version(VueVersion::V2);
    let full = default_pipeline_for(caps, every_family());
    assert_eq!(full.passes.len(), 5);
    assert_eq!(full.group_count(), 4);
    assert_eq!(full.passes[0], legacy::DESC);

    let bare = default_pipeline_for(caps, LoweringFeatures::EMPTY);
    assert_names(&bare, &[legacy::DESC.name, hoist::NAME, cfg::NAME]);
}

#[test]
fn vue2_legacy_sugar_preserves_the_dom_emit_static_analysis_choice() {
    let caps = LegacyCaps::for_version(VueVersion::V2);

    let full = pipeline_for_profile(caps, every_family(), EMIT);
    assert_eq!(full.passes.len(), 3);
    assert!(!full.passes.iter().any(|pass| pass.name == hoist::NAME));

    let bare = pipeline_for_profile(caps, LoweringFeatures::EMPTY, EMIT);
    assert_names(&bare, &[legacy::DESC.name]);
}

#[test]
fn the_mask_reads_one_bit_per_selectable_pass() {
    assert_eq!(SELECTABLE.len(), 5);
    assert_eq!(PLAN_COUNT, 32);
    let bits = [
        LEGACY_SUGAR,
        SLOT_CARRIERS,
        MODEL_BINDINGS,
        STATIC_ANALYSIS,
        COMPLEXITY_ANALYSIS,
    ];
    for (index, (_, bit)) in SELECTABLE.iter().enumerate() {
        assert_eq!(*bit, bits[index], "SELECTABLE order must match the bits");
    }
    assert_eq!(
        mask_for(
            LegacyCaps::for_version(VueVersion::V2),
            every_family(),
            TransformProfile::DEFAULT
        ),
        u8::try_from(PLAN_COUNT - 1).expect("five bits"),
    );
    assert_eq!(mask_for(LegacyCaps::VUE3, LoweringFeatures::EMPTY, EMIT), 0);
}
