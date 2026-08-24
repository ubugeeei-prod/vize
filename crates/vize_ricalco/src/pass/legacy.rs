//! Vue 2 template-sugar legalization (P2-9 installment 7).
//!
//! The lowering admits `.sync` / `slot-scope` / pipe filters as dialect
//! payloads. This pass rewrites them into the Vue 3 surface the rest of
//! the pipeline already understands: `vue.sync` → `ui.bind` + `ui.on`,
//! `vue.slot-scope` → `ui.slot-content` (1:1, same introduction site),
//! `.native`/numeric keyCodes on `ui.on`, and `vue.filter` →
//! `_filter_*(...)`. Vue 3 never enters ([`pipeline_for`]): the 6-pass
//! table is unchanged, a single `needs_sugar()` read.

use vize_davinci::pass::{Fusability, PassDesc, PassKind, Pipeline, Preserved};

use crate::lower::{LegacyCaps, Lowered};

mod filter;
mod ids;
mod on;
mod slot;
mod sync;
mod tree;

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "legacy-sugar";

/// **`MandatoryLowering`, barrier** — skipping it leaves Vue 2 sugar as
/// dialect ops the rest of the lane does not consume; inserting `ui.on`
/// for `.sync` is a structural rewrite, so it is the lowering kind.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    // Inserted listeners shift every later page-order id; nothing that
    // ran before this pass (nothing does — we are first) is preserved
    // as a numbered fact.
    Preserved::NONE,
);

/// Vue 2's pipeline: this pass, then the Vue 3 table.
pub const LEGACY_PASSES: &[PassDesc] = &[
    DESC,
    super::vif::DESC,
    super::vfor::DESC,
    super::vslot::DESC,
    super::text::DESC,
    super::vmodel::DESC,
    super::hoist::DESC,
];

/// The planned pipeline over [`LEGACY_PASSES`].
pub const LEGACY: Pipeline = Pipeline::new(super::S2_STAGE, LEGACY_PASSES);

const _: () = assert!(LEGACY.group_count() == 7);
const _: () = assert!(LEGACY.is_fully_serialized());

/// Vue 3 is the 6-pass table; every legacy dialect prepends this pass.
#[must_use]
pub const fn pipeline_for(caps: LegacyCaps) -> Pipeline {
    if caps.needs_sugar() {
        LEGACY
    } else {
        super::TRANSFORM
    }
}

/// Legalize Vue 2 sugar in place, then recount and rekey.
pub fn run(lowered: &mut Lowered<'_>) {
    let allocator = lowered.allocator;
    let sync_ids = ids::collect_sync_ids(&lowered.root.ops);
    tree::map_binding_lists(&mut lowered.root.ops, &mut |bindings| {
        sync::expand(allocator, bindings);
        slot::convert(allocator, bindings);
        if lowered.caps.v2_event_sugar {
            on::rewrite(allocator, bindings);
        }
    });
    if lowered.caps.supports_filters {
        filter::rewrite(allocator, &mut lowered.root.ops);
    }
    ids::rekey(lowered, &sync_ids);
    ids::recount(lowered);
}

#[cfg(test)]
mod tests {
    use super::{DESC, LEGACY, LEGACY_PASSES, pipeline_for};
    use crate::lower::LegacyCaps;
    use crate::pass::{TRANSFORM, TRANSFORM_PASSES};
    use vize_carton::config::VueVersion;
    use vize_davinci::pass::{Fusability, PassKind, Preserved};

    #[test]
    fn vue3_keeps_the_six_pass_table() {
        assert_eq!(pipeline_for(LegacyCaps::VUE3), TRANSFORM);
        assert_eq!(TRANSFORM_PASSES.len(), 6);
    }

    #[test]
    fn vue2_prepends_the_legalizing_pass() {
        let caps = LegacyCaps::for_version(VueVersion::V2);
        assert_eq!(pipeline_for(caps), LEGACY);
        assert_eq!(LEGACY_PASSES.len(), 7);
        assert_eq!(LEGACY_PASSES[0], DESC);
        assert_eq!(DESC.name, "legacy-sugar");
        assert_eq!(DESC.kind, PassKind::MandatoryLowering);
        assert_eq!(DESC.fusability, Fusability::Barrier);
        assert_eq!(DESC.preserved, Preserved::NONE);
        assert_eq!(LEGACY.group_count(), 7);
    }
}
