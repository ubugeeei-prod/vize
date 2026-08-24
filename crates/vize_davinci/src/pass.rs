//! The pass manager: pipelines as const data, classified and fused at build
//! time.
//!
//! # Performance guardrail 1, which shapes everything here
//!
//! `architecture.md`'s first guardrail is that dispatch is resolved **per
//! pipeline, never per node**. That rules out the obvious design — a registry
//! of `Box<dyn Pass>` looked up while walking — and it is why a pass is
//! described by [`PassDesc`], a plain `Copy` struct of `&'static str` and
//! enums, and a pipeline is a `&'static [PassDesc]`. Nothing here allocates,
//! nothing here is dynamically dispatched, and every planning question a
//! caller can ask — how many walks, which passes fuse, what a group preserves
//! — is answered by a `const fn`.
//!
//! That last property is the one worth stating twice: a fusion plan can be
//! pinned in a `const` item, so **a grouping regression is a compile error**
//! rather than a runtime surprise nobody notices.
//!
//! # The three laws
//!
//! 1. **Mandatory passes are unfusable barriers.** Enforced in
//!    [`PassDesc::new`], which is a `const fn` with an assertion — so a
//!    `const DESC` that breaks it does not compile.
//! 2. **Only a mandatory pass performs the raw → canonical transition.**
//!    Enforced at the type level in [`canonical`].
//! 3. **A fused group preserves what every member preserves.** The
//!    intersection is a `const fn` ([`Preserved::intersect`]).
//!
//! # Module layout
//!
//! Split into files rather than one module because the repository's
//! source-length budget is 350 lines
//! (`tools/moon/cmd/source_file_lengths`); the crate has no `mod.rs` anywhere,
//! per the workspace convention.
//!
//! - [`kind`] — [`PassKind`] and [`Fusability`]
//! - [`preserved`] — [`AnalysisId`] and [`Preserved`]
//! - [`canonical`] — [`Raw`] and [`Canonical`]
//! - [`fusion`] — [`Pipeline`] and [`FusionGroup`]
//! - [`observer`] — [`PassObserver`], the run driver, and the four in-tree
//!   observers
//! - [`pipeline`] — the textual pipeline syntax `davinci-opt --pipeline` reads
//!
//! # What this module does not do yet
//!
//! It plans, and it drives a plan; it does not supply pass bodies. There is no
//! S2 to run over until P2-5a and no catalogue binding a name to an
//! implementation until P2-9, so [`Pass`] carries a description and nothing
//! else, and [`run_pipeline`] takes the body as a callback. Optimization tiers
//! that scale budgets rather than pass sets are P3-10's.

pub mod canonical;
pub mod fusion;
pub mod kind;
pub mod observer;
pub mod pipeline;
pub mod preserved;

pub use canonical::{Canonical, Raw};
pub use fusion::{FusionGroup, Pipeline};
pub use kind::{Fusability, PassKind};
pub use observer::{
    AnalysisEvent, BudgetObserver, FailEvent, NoObserver, Pair, PassEvent, PassFailure,
    PassObserver, TimingObserver, run_pipeline,
};
pub use pipeline::{PipelineSyntaxError, parse_pipelines, print_pipelines};
pub use preserved::{AnalysisId, MAX_ANALYSES, Preserved};

/// Everything the pass manager needs to know about a pass, as const data.
///
/// A `Copy` struct of `&'static str` and enums: 8-byte preserved mask, two
/// one-byte tags, one string. No allocation, no trait object, no registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassDesc {
    /// The pass's identity in pipeline strings and folio pages. Lowercase
    /// kebab-case, because [`pipeline`]'s grammar accepts exactly that.
    pub name: &'static str,
    /// What the pass is for, and therefore when it runs.
    pub kind: PassKind,
    /// Whether the pass may share a walk.
    pub fusability: Fusability,
    /// The analyses the pass leaves valid.
    pub preserved: Preserved,
}

impl PassDesc {
    /// Describe a pass.
    ///
    /// # Panics
    ///
    /// Panics if `kind` is mandatory and `fusability` is
    /// [`Fusability::Fusable`]. Descriptions are declared as `const DESC`
    /// items, so that panic is a **compile error** — which is the whole
    /// enforcement of law 1. A mandatory pass either needs whole-tree facts or
    /// establishes an invariant the next pass reads, and fusion breaks both.
    ///
    /// Also panics on an empty name: an unnamed pass cannot appear in a
    /// pipeline string, so it cannot be selected, dumped or gated.
    #[must_use]
    pub const fn new(
        name: &'static str,
        kind: PassKind,
        fusability: Fusability,
        preserved: Preserved,
    ) -> Self {
        assert!(!name.is_empty(), "a pass must have a name");
        assert!(
            !(kind.is_mandatory() && fusability.is_fusable()),
            "a mandatory pass must be a barrier: it either needs whole-tree facts \
             or establishes an invariant the next pass reads, and fusion breaks both"
        );
        Self {
            name,
            kind,
            fusability,
            preserved,
        }
    }

    /// Whether this pass runs at every optimization level.
    #[inline]
    #[must_use]
    pub const fn is_mandatory(&self) -> bool {
        self.kind.is_mandatory()
    }
}

/// `PassDesc` is copied into every planning query; keep it one cache line's
/// worth. 16 bytes of `&str`, 8 of mask, 2 of tags, padded to 32.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<PassDesc>() == 32);

/// A pass, identified by its description.
///
/// Deliberately carries no `run` method: there is no stage artifact to run
/// over until P2-5a lands S2, and inventing the signature now would fix a
/// contract against a type that does not exist. The trait exists in P2-2 so
/// the type-level canonicalization check in [`canonical`] has something to be
/// generic over.
pub trait Pass {
    /// This pass's const description.
    const DESC: PassDesc;
}

#[cfg(test)]
mod tests {
    use super::{Fusability, Pass, PassDesc, PassKind, Pipeline, Preserved};

    const SCOPES: super::AnalysisId = super::AnalysisId::new(0);
    const CONSTNESS: super::AnalysisId = super::AnalysisId::new(1);

    const NORMALIZE: PassDesc = PassDesc::new(
        "normalize",
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::ALL,
    );
    const FOLD: PassDesc = PassDesc::new(
        "fold",
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::NONE.with(SCOPES).with(CONSTNESS),
    );
    const HOIST: PassDesc = PassDesc::new(
        "hoist",
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::NONE.with(SCOPES),
    );
    const CHECK: PassDesc = PassDesc::new(
        "check",
        PassKind::MandatoryDiagnostic,
        Fusability::Barrier,
        Preserved::ALL,
    );

    struct Normalizer;
    impl Pass for Normalizer {
        const DESC: PassDesc = NORMALIZE;
    }

    #[test]
    fn a_description_keeps_what_it_was_given() {
        assert_eq!(NORMALIZE.name, "normalize");
        assert_eq!(NORMALIZE.kind, PassKind::Optional);
        assert_eq!(NORMALIZE.fusability, Fusability::Fusable);
        assert!(!NORMALIZE.is_mandatory());
        assert!(CHECK.is_mandatory());
    }

    #[test]
    fn a_pass_reaches_its_description_through_the_trait() {
        assert_eq!(Normalizer::DESC, NORMALIZE);
    }

    /// Law 1 in `const` form. `PassDesc::new` asserts, so the illegal
    /// combination cannot be written as a `const` item at all — which is why
    /// the negative case is a comment rather than a test:
    /// `PassDesc::new("x", PassKind::MandatoryLowering, Fusability::Fusable,
    /// Preserved::NONE)` fails to compile with "a mandatory pass must be a
    /// barrier". What is testable is that every mandatory description in the
    /// tree is in fact a barrier.
    #[test]
    fn every_mandatory_pass_is_a_barrier() {
        for desc in [NORMALIZE, FOLD, HOIST, CHECK] {
            if desc.is_mandatory() {
                assert_eq!(
                    desc.fusability,
                    Fusability::Barrier,
                    "{} is mandatory and must be a barrier",
                    desc.name
                );
            }
        }
    }

    /// The fusion plan of a fixture pipeline, pinned in `const` items so a
    /// grouping regression is a compile error rather than a test failure.
    #[test]
    fn the_fixture_pipeline_groups_are_const_pinned() {
        const PASSES: &[PassDesc] = &[NORMALIZE, FOLD, CHECK, HOIST];
        const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

        const _: () = assert!(PIPELINE.group_count() == 3);
        // normalize + fold fuse into one walk.
        const FIRST: super::FusionGroup = match PIPELINE.group(0) {
            Some(group) => group,
            None => panic!("group 0 exists"),
        };
        const _: () = assert!(FIRST.start == 0 && FIRST.len == 2 && !FIRST.is_barrier);
        // The fused group preserves the intersection: fold drops everything
        // but scopes and constness, so the group cannot preserve more.
        const _: () = assert!(FIRST.preserved.preserves(SCOPES));
        const _: () = assert!(FIRST.preserved.preserves(CONSTNESS));
        // check is a mandatory barrier and sits alone.
        const SECOND: super::FusionGroup = match PIPELINE.group(1) {
            Some(group) => group,
            None => panic!("group 1 exists"),
        };
        const _: () = assert!(SECOND.start == 2 && SECOND.len == 1 && SECOND.is_barrier);
        // hoist starts a fresh group because a barrier preceded it.
        const THIRD: super::FusionGroup = match PIPELINE.group(2) {
            Some(group) => group,
            None => panic!("group 2 exists"),
        };
        const _: () = assert!(THIRD.start == 3 && THIRD.len == 1 && !THIRD.is_barrier);
        const _: () = assert!(PIPELINE.group(3).is_none());

        assert_eq!(PIPELINE.group_count(), 3);
    }

    /// The canary the acceptance names: a mandatory pass never joins a fusion
    /// group. Proven over every arrangement of the fixture passes rather than
    /// one, so a rule that only holds in the middle of a pipeline fails here.
    #[test]
    fn a_mandatory_pass_never_joins_a_fusion_group() {
        const ARRANGEMENTS: [&[PassDesc]; 5] = [
            &[CHECK, NORMALIZE, FOLD],
            &[NORMALIZE, CHECK, FOLD],
            &[NORMALIZE, FOLD, CHECK],
            &[CHECK, CHECK],
            &[CHECK],
        ];
        for passes in ARRANGEMENTS {
            let pipeline = Pipeline::new("s2", passes);
            for group in pipeline.groups() {
                let members = pipeline.passes_of(group);
                let mandatory = members.iter().filter(|desc| desc.is_mandatory()).count();
                if mandatory > 0 {
                    assert_eq!(
                        members.len(),
                        1,
                        "a group holding a mandatory pass must hold nothing else"
                    );
                    assert!(group.is_barrier);
                }
            }
        }
    }

    #[test]
    fn a_pipeline_of_only_barriers_is_fully_serialized() {
        const PASSES: &[PassDesc] = &[CHECK, CHECK, CHECK];
        const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);
        const _: () = assert!(PIPELINE.is_fully_serialized());
        assert_eq!(PIPELINE.group_count(), 3);
    }

    #[test]
    fn an_empty_pipeline_costs_no_walks() {
        const PIPELINE: Pipeline = Pipeline::new("s2", &[]);
        const _: () = assert!(PIPELINE.group_count() == 0);
        assert!(PIPELINE.group(0).is_none());
        assert!(PIPELINE.group_of_pass(0).is_none());
    }

    #[test]
    fn every_pass_reports_the_group_it_landed_in() {
        const PASSES: &[PassDesc] = &[NORMALIZE, FOLD, CHECK, HOIST];
        const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);
        assert_eq!(PIPELINE.group_of_pass(0), Some(0));
        assert_eq!(PIPELINE.group_of_pass(1), Some(0));
        assert_eq!(PIPELINE.group_of_pass(2), Some(1));
        assert_eq!(PIPELINE.group_of_pass(3), Some(2));
        assert_eq!(PIPELINE.group_of_pass(4), None);
    }
}
