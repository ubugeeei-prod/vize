//! The pipelines Davinci is replacing, described in Davinci's own terms.
//!
//! Until this module existed, `Pipeline` described nothing that runs: the type
//! was exercised only by fixtures it defined itself. Here the three shipped
//! backends' template traversals are declared as [`Pipeline`] const data, so
//! the pass manager is the authority on what the compiler walks **today** —
//! before any pass body has moved onto S2, and therefore before there is
//! anything to compare a migration against.
//!
//! # Why this lives in `vize_davinci` and not in `vize_atelier_core`
//!
//! It was written there first, as a normal dependency, and the release gate
//! rejected it: `vize_atelier_core` is published to crates.io and
//! `vize_davinci` is `publish = false`, so a published crate would have
//! carried an unresolvable dependency
//! (`tests/tooling/moonbit-publish-crates.test.ts`). The constraint is real
//! and it is not a packaging detail — **no published crate can consume any
//! Davinci crate until the program decides to publish them**, which is a
//! question P2-11 has to answer before the DOM backend can actually run on
//! S2, not after.
//!
//! So the plans live here and the backends read them from their
//! **dev-dependencies**, which is the same shape `davinci_harness` already
//! uses to instrument published crates. Davinci is exercised against the real
//! pipeline without entering its release graph.
//!
//! # What is declared, and what it is measured against
//!
//! Exactly the walks `budgets.toml [traversal]` gates: traversals of the
//! **template tree**. `vize_atelier_vapor::generate` walks Vapor IR rather
//! than the template tree and `vize_croquis` walks the script AST; neither is
//! a template traversal, so neither appears here, and
//! `davinci-road/plan/walk-baseline.md` records the same exclusion for the
//! probe that measures these plans.
//!
//! The tie is a law, not a comment: each backend's
//! `tests/davinci_walk_baseline.rs` asserts that the walks
//! `vize_atelier_core::walk_probe` counted equal
//! [`Pipeline::group_count`] of the plan below. A plan that drifts from the
//! pipeline it describes fails there, which is the whole point of declaring
//! it — a description nothing checks is a comment.
//!
//! # Why every pass here is a mandatory barrier
//!
//! Because that is what the pipeline is today, and the plan has to say so.
//! Each stage owns its own full traversal: P2-12a measured `walks = 2` on
//! every backend and every ladder fixture, and the transform column identical
//! across all three. Declaring these fusable would make the plan claim one
//! walk where the compiler makes two, and `PassDesc::new` would reject it
//! anyway — mandatory passes are unfusable barriers by construction.
//!
//! Phase 2's DOM target (`budgets.toml`'s `[target.phase-2]`,
//! `dom_walks_max = 1`) is exactly the claim that [`DOM`] below becomes one
//! group once P2-11 moves the DOM backend onto S2. That is measurable against
//! this declaration rather than against prose.

use crate::pass::{Fusability, PassDesc, PassKind, Pipeline, Preserved};

/// The template stage every plan here runs over.
///
/// Named for what it is today rather than for S2: these passes read and mutate
/// the `vize_relief` template AST, and calling that `s2` would make the folio
/// pages and profile attributions lie about which IR was walked.
pub const TEMPLATE_STAGE: &str = "template";

/// The transform lane (`vize_atelier_core::lane`): one full traversal with enter/exit
/// callbacks, structural directive handling and static hoisting.
///
/// Mandatory because skipping it does not produce slower output, it produces
/// wrong output — and a barrier because its enter/exit sibling mutation (the
/// `v-else` merge on the parent's child list) is precisely the re-visit source
/// a region-owning `ui.if` op will remove.
pub const TRANSFORM: PassDesc = PassDesc::new(
    "transform",
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    Preserved::NONE,
);

/// DOM / base code generation (`vize_atelier_core::codegen`): a second full traversal of
/// the transformed tree.
pub const CODEGEN: PassDesc = PassDesc::new(
    "codegen",
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    Preserved::NONE,
);

/// SSR code generation (`vize_atelier_ssr::codegen`).
pub const SSR_CODEGEN: PassDesc = PassDesc::new(
    "ssr-codegen",
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    Preserved::NONE,
);

/// Vapor IR lowering (`vize_atelier_vapor::lower`).
///
/// Vapor *generate* is not here: it walks Vapor IR, not the template tree.
pub const VAPOR_LOWER: PassDesc = PassDesc::new(
    "vapor-lower",
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    Preserved::NONE,
);

const DOM_PASSES: &[PassDesc] = &[TRANSFORM, CODEGEN];
const SSR_PASSES: &[PassDesc] = &[TRANSFORM, SSR_CODEGEN];
const VAPOR_PASSES: &[PassDesc] = &[TRANSFORM, VAPOR_LOWER];

/// The DOM backend's template traversals. Phase 2's strangler target (P2-11).
pub const DOM: Pipeline = Pipeline::new(TEMPLATE_STAGE, DOM_PASSES);

/// The SSR backend's template traversals.
pub const SSR: Pipeline = Pipeline::new(TEMPLATE_STAGE, SSR_PASSES);

/// The Vapor backend's template traversals.
pub const VAPOR: Pipeline = Pipeline::new(TEMPLATE_STAGE, VAPOR_PASSES);

/// Today every stage owns its walk, so each plan is fully serialized and the
/// group count is the pass count. Pinned in `const` items rather than a test,
/// because the day this stops holding is the day fusion started working and
/// the change must be deliberate.
const _: () = assert!(DOM.group_count() == 2);
const _: () = assert!(SSR.group_count() == 2);
const _: () = assert!(VAPOR.group_count() == 2);
const _: () = assert!(DOM.is_fully_serialized());
const _: () = assert!(SSR.is_fully_serialized());
const _: () = assert!(VAPOR.is_fully_serialized());

#[cfg(test)]
mod tests {
    use super::{CODEGEN, DOM, SSR, SSR_CODEGEN, TRANSFORM, VAPOR, VAPOR_LOWER};

    #[test]
    fn every_backend_starts_from_the_same_transform_lane() {
        // The measured baseline's transform column is identical across the
        // three backends because they run the same lane; the plans say so.
        assert_eq!(DOM.passes[0], TRANSFORM);
        assert_eq!(SSR.passes[0], TRANSFORM);
        assert_eq!(VAPOR.passes[0], TRANSFORM);
    }

    #[test]
    fn each_backend_adds_exactly_one_traversal_of_its_own() {
        assert_eq!(DOM.passes[1], CODEGEN);
        assert_eq!(SSR.passes[1], SSR_CODEGEN);
        assert_eq!(VAPOR.passes[1], VAPOR_LOWER);
        for pipeline in [DOM, SSR, VAPOR] {
            assert_eq!(pipeline.passes.len(), 2);
        }
    }

    #[test]
    fn nothing_fuses_today_so_every_pass_owns_its_walk() {
        for pipeline in [DOM, SSR, VAPOR] {
            assert_eq!(pipeline.group_count(), pipeline.passes.len());
            for group in pipeline.groups() {
                assert_eq!(group.len, 1);
                assert!(group.is_barrier);
            }
        }
    }
}
