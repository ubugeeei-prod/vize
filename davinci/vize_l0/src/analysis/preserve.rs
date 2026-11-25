//! Facts across passes (P4-1b): post-hoc `Preserved` sets, named
//! preservation groups, and the recompute-and-compare verify mode.
//!
//! The LLVM new-pass-manager import: a pass does not name what it breaks,
//! it names what it **preserves**, after the fact, and the manager drops
//! everything else. A pass that preserves nothing (the
//! [`Preserved::NONE`] default) costs a recomputation, never a stale fact.
//!
//! The failure mode that design leaves open is a pass that *claims* too
//! much. [`AnalysisVerifyObserver`] closes it: after a pass it recomputes every
//! group the pass kept, from scratch on the pass's output, and compares
//! with exact equality — a mismatch is the exact
//! [`AnalysisError::StalePreserved`]. The check is chosen by type (P2-3's
//! static dispatch): [`NoAnalysisVerify`] and a release-build
//! [`AnalysisVerifyObserver`] compile to nothing, and both are zero-sized.

use super::ids::{
    BINDINGS, COMPONENT_USAGES, REACTIVITY, RENDER_TREE, UNDEFINED_REFS, UNUSED_BINDINGS,
};
use super::manager::{AnalysisManager, compute_into};
use super::view::Tables;
use super::{AnalysisError, Demand};
use crate::pass::{AnalysisId, PassDesc, Preserved};

/// Facts over element, component and region structure: a pass that
/// rewrites only expressions (constant folding, expression normalization)
/// preserves them.
pub const PRESERVE_STRUCTURE: Preserved = Preserved::NONE.with(COMPONENT_USAGES).with(RENDER_TREE);

/// Facts over script bindings and the expressions that read them: a pass
/// that moves or hoists template structure without touching an expression
/// preserves them.
pub const PRESERVE_BINDINGS: Preserved = Preserved::NONE
    .with(BINDINGS)
    .with(UNDEFINED_REFS)
    .with(UNUSED_BINDINGS)
    .with(REACTIVITY);

/// Whether [`AnalysisManager::after_pass`] re-checks what a pass preserved.
/// Implemented by zero-sized policy types, so the choice is static dispatch
/// and the disabled shape costs nothing.
pub trait AnalysisVerify {
    /// Recompute and compare every kept group after every pass.
    const ENABLED: bool;
}

/// Verify mode: in debug builds, recompute every preserved group after a
/// pass and compare it with the kept table by exact equality. In release
/// builds it is [`NoAnalysisVerify`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AnalysisVerifyObserver;

impl AnalysisVerify for AnalysisVerifyObserver {
    const ENABLED: bool = cfg!(debug_assertions);
}

/// No verification: trust every pass's `Preserved` claim.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoAnalysisVerify;

impl AnalysisVerify for NoAnalysisVerify {
    const ENABLED: bool = false;
}

const _: () = assert!(size_of::<AnalysisVerifyObserver>() == 0);
const _: () = assert!(size_of::<NoAnalysisVerify>() == 0);

impl<A: ?Sized + 'static> AnalysisManager<'_, A> {
    /// Account for a pass that just ran over `artifact`: drop every computed
    /// group outside `desc.preserved`, then — when `V` verifies — recompute
    /// every kept group on `artifact` and compare. Returns the dropped set.
    ///
    /// # Errors
    ///
    /// [`AnalysisError::StalePreserved`] naming the first kept group (stratum
    /// order, then registration order) whose recomputation differs. Every
    /// stale group is dropped as well, so a later query recomputes it
    /// instead of reading the stale table.
    pub fn after_pass<V: AnalysisVerify>(
        &mut self,
        artifact: &A,
        desc: &PassDesc,
    ) -> Result<Demand, AnalysisError> {
        let kept = self.tables.computed.surviving(desc.preserved);
        let dropped = self.tables.computed.minus(kept);
        self.tables.drop_groups(dropped);
        if V::ENABLED
            && let Some((first, stale)) = self.stale(artifact, kept)
        {
            self.tables.drop_groups(stale);
            return Err(AnalysisError::StalePreserved {
                pass: desc.name,
                group: first,
            });
        }
        Ok(dropped)
    }

    /// The groups in `kept` whose tables differ from a from-scratch
    /// recomputation on `artifact`, with the first of them in stratum order.
    fn stale(&self, artifact: &A, kept: Demand) -> Option<(AnalysisId, Demand)> {
        if kept.is_empty() {
            return None;
        }
        let mut fresh = Tables::new();
        compute_into(
            self.registry,
            &mut fresh,
            artifact,
            self.registry.closure(kept),
            false,
        );
        let producers = self.registry.producers();
        let top = producers.iter().map(|entry| entry.desc.stratum).max()?;
        let mut first = None;
        let mut stale = Demand::NONE;
        for stratum in 0..=top {
            for entry in producers {
                let id = entry.desc.id;
                if entry.desc.stratum != stratum || !kept.contains(id) {
                    continue;
                }
                let same = match (self.tables.slot(id), fresh.slot(id)) {
                    (Some(old), Some(new)) => (entry.eq)(old, new),
                    _ => false,
                };
                if !same {
                    first.get_or_insert(id);
                    stale = stale.with(id);
                }
            }
        }
        first.map(|first| (first, stale))
    }
}
