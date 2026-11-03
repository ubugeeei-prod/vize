//! Artifact-scoped L2 transform plans.
//!
//! The static transform table stays the review unit, but the build path
//! should not pay a diagnostic pass for an op family the lowering did not
//! produce. Feature bits come from the lowering construction sites, not a
//! second L2 walk.
//!
//! # Why the plan is a mask, not a match arm
//!
//! Each selectable pass is declined independently — its own op family is
//! absent, or the DOM emit declared it cannot consume the pass's product —
//! so the reachable plans are the *subsets* of [`SELECTABLE`], not a short
//! list of shapes. Enumerating them as named `const` pipelines costs one
//! item per combination and doubles with every pass that learns to
//! decline; at four selectable passes it is already 16 names for one
//! rule. `v-for`, like compound text, is lowering-published and no
//! longer participates in this mask. [`plan_for_mask`] states the rule
//! once instead, and [`PLANS`]
//! holds its answer for every mask so a [`Pipeline`]'s
//! `&'static [PassDesc]` still points at data nobody built at run time.
//!
//! The `const fn` is what keeps the compile-time pins available: the
//! `const _: () = assert!(…)` items below evaluate [`plan_for_mask`]
//! directly, so a plan-shape regression is a **compile error** exactly as
//! it was when each shape had a name (the P2-2 convention).
//!
//! # What a decline may cost
//!
//! Nothing the artifact can observe. A declined pass is one whose op
//! family the lowering never built, so its walk would publish an empty
//! fact table and raise no diagnostic — see each remaining pass's
//! `run` for the `if let Op::…` that is its whole body, and
//! `tests/lowering_features.rs` for the behavioural pin. `v-if` joins
//! the lowering-published families, so its op bit is ignored like
//! `v-for` and compound text. `hoist` and `template-complexity` are the
//! declines made for a different reason: both are `Optional`, the DOM
//! emit declines hoist facts outright under `hoist_static: false`, and
//! no emitter reads complexity facts at all.

use vize_l0::pass::{PassDesc, Pipeline};

use crate::lower::{LegacyCaps, LoweringFeatures};

use super::{L2_STAGE, cfg, hoist, legacy, vmodel, vslot};

/// Product-facing selection for optional L2 transform work.
///
/// The transform catalogue stays the full review surface. One-shot DOM
/// emission can still decline facts it cannot consume: with
/// `hoist_static: false`, `hoist-static` would only spend an L2 walk to
/// produce facts that the emitter is required to ignore, and no emitter
/// reads `template-complexity`'s facts at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformProfile {
    include_static_analysis: bool,
    include_complexity_analysis: bool,
}

impl TransformProfile {
    /// The full transform plan.
    pub const DEFAULT: Self = Self {
        include_static_analysis: true,
        include_complexity_analysis: true,
    };

    /// Drop the optional `hoist-static` analysis pass.
    #[must_use]
    pub const fn without_static_analysis(self) -> Self {
        Self {
            include_static_analysis: false,
            ..self
        }
    }

    /// Drop the optional `template-complexity` analysis pass (every
    /// emitter: none reads it).
    #[must_use]
    pub const fn without_complexity_analysis(self) -> Self {
        Self {
            include_complexity_analysis: false,
            ..self
        }
    }

    #[must_use]
    pub const fn includes_static_analysis(self) -> bool {
        self.include_static_analysis
    }

    #[must_use]
    pub const fn includes_complexity_analysis(self) -> bool {
        self.include_complexity_analysis
    }
}

/// One selectable pass and the mask bit that keeps it.
///
/// Order here is the pipeline's execution order, so a plan is this table
/// filtered — never reordered. The passes touch disjoint op families and
/// carry no semantic dependency on one another (`super::TRANSFORM`'s
/// docs), which is what makes an arbitrary subset a legal pipeline.
const SELECTABLE: [(PassDesc, u8); 5] = [
    (legacy::DESC, LEGACY_SUGAR),
    (vslot::DESC, SLOT_CARRIERS),
    (vmodel::DESC, MODEL_BINDINGS),
    (hoist::DESC, STATIC_ANALYSIS),
    (cfg::DESC, COMPLEXITY_ANALYSIS),
];

const LEGACY_SUGAR: u8 = 1 << 0;
const SLOT_CARRIERS: u8 = 1 << 1;
const MODEL_BINDINGS: u8 = 1 << 2;
const STATIC_ANALYSIS: u8 = 1 << 3;
const COMPLEXITY_ANALYSIS: u8 = 1 << 4;

/// Every mask, hence every plan.
const PLAN_COUNT: usize = 1 << SELECTABLE.len();

/// The passes one mask selects, in execution order.
///
/// `passes` is fixed-capacity because a [`Pipeline`] borrows
/// `&'static [PassDesc]`; slots past `len` are filler and never exposed —
/// [`Plan::pipeline`] is the only reader and it slices to `len`.
#[derive(Debug, Clone, Copy)]
struct Plan {
    passes: [PassDesc; SELECTABLE.len()],
    len: usize,
}

impl Plan {
    const EMPTY: Self = Self {
        passes: [hoist::DESC; SELECTABLE.len()],
        len: 0,
    };

    fn pipeline(&'static self) -> Pipeline {
        Pipeline::new(L2_STAGE, self.passes.get(..self.len).unwrap_or(&[]))
    }
}

/// The plan `mask` selects: [`SELECTABLE`] filtered in landed order.
const fn plan_for_mask(mask: u8) -> Plan {
    let mut plan = Plan::EMPTY;
    let mut free: &mut [PassDesc] = &mut plan.passes;
    let mut rest: &[(PassDesc, u8)] = &SELECTABLE;
    while let [(desc, bit), tail @ ..] = rest {
        if mask & *bit != 0
            && let [slot, free_tail @ ..] = free
        {
            *slot = *desc;
            free = free_tail;
            plan.len += 1;
        }
        rest = tail;
    }
    plan
}

const fn all_plans() -> [Plan; PLAN_COUNT] {
    let mut plans = [Plan::EMPTY; PLAN_COUNT];
    let mut rest: &mut [Plan] = &mut plans;
    // Plan `i` sits at index `i`; `PLAN_COUNT` is `1 << 5`, so every mask
    // fits a `u8`.
    let mut mask: u8 = 0;
    while let [plan, tail @ ..] = rest {
        *plan = plan_for_mask(mask);
        mask = mask.wrapping_add(1);
        rest = tail;
    }
    plans
}

/// Every plan, so a selected [`Pipeline`] borrows rather than builds.
static PLANS: [Plan; PLAN_COUNT] = all_plans();

// The full plan is the landed selectable table, and the empty plan is now
// truly empty: a plan-shape regression is a compile error, not a test failure.
const _: () = assert!(plan_for_mask(u8::MAX).len == 5);
const _: () = assert!(plan_for_mask(0).len == 0);
const _: () = assert!(plan_for_mask(STATIC_ANALYSIS).len == 1);
const _: () = assert!(plan_for_mask(STATIC_ANALYSIS | COMPLEXITY_ANALYSIS).len == 2);

fn mask_for(caps: LegacyCaps, features: LoweringFeatures, profile: TransformProfile) -> u8 {
    let mut mask = 0;
    if caps.needs_sugar() {
        mask |= LEGACY_SUGAR;
    }
    let _if_facts_are_lowering_published = features.has_if_ops();
    let _for_facts_are_lowering_published = features.has_for_ops();
    if features.has_slot_carriers() {
        mask |= SLOT_CARRIERS;
    }
    if features.has_model_bindings() {
        mask |= MODEL_BINDINGS;
    }
    if profile.includes_static_analysis() {
        mask |= STATIC_ANALYSIS;
    }
    if profile.includes_complexity_analysis() {
        mask |= COMPLEXITY_ANALYSIS;
    }
    mask
}

pub(super) fn pipeline_for_profile(
    caps: LegacyCaps,
    features: LoweringFeatures,
    profile: TransformProfile,
) -> Pipeline {
    match PLANS.get(usize::from(mask_for(caps, features, profile))) {
        Some(plan) => plan.pipeline(),
        // Every mask is built from the five selectable bits, so it names a
        // plan; an out-of-table mask would select no optional pass.
        None => Pipeline::new(L2_STAGE, &[]),
    }
}

#[cfg(test)]
mod tests {
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
    use vize_l0::config::VueVersion;

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
    ) -> vize_l0::pass::Pipeline {
        pipeline_for_profile(caps, features, TransformProfile::DEFAULT)
    }

    /// Assert a pipeline's pass names, in order. Spelled as a zip rather
    /// than a collected `Vec` so this suite owns no storage — the file is a
    /// separate module, so `davinci-storage-policy`'s `cfg(test)` masking
    /// does not reach it.
    #[track_caller]
    fn assert_names(pipeline: &vize_l0::pass::Pipeline, expected: &[&str]) {
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
}
