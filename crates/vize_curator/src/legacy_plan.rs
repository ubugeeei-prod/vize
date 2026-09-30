//! Historical template-traversal plans for inspected legacy compiles.
//!
//! The DOM, SSR and Vapor plans record the template walks pinned by the
//! backends' `davinci_walk_baseline` laws. Each backend reads this inspection
//! metadata through a path-only dev dependency. The CLI uses the same plans
//! to describe its selected legacy compile when writing a crash report.
//!
//! The declarations describe traversals; the real pass bodies run in their
//! backend owners. These plans retain the historical two-barrier baseline and
//! do not attribute native per-pass execution. Vapor generation walks Vapor
//! IR and Croquis walks the script AST, so both remain outside this template
//! traversal baseline. See `docs/davinci/plan/walk-baseline.md`.

use vize_l0::pass::{Fusability, PassDesc, PassKind, Pipeline, Preserved};

/// The template stage every plan here runs over.
///
/// Named for what it is today rather than for L2: these passes read and mutate
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
