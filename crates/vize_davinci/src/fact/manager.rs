//! [`FactManager`], [`FactView`], [`FactError`] and the process-global
//! counters that pin "each group at most once per artifact" and "zero
//! undeclared accesses".

use core::any::type_name;
use core::sync::atomic::{AtomicU64, Ordering};

use super::registry::{ErasedTable, FactRegistry};
use super::{Demand, FactConsumer, FactGroup, FactTable};
use crate::pass::{AnalysisId, MAX_ANALYSES};

const SLOTS: usize = MAX_ANALYSES as usize;

/// How many times each group's producer has run, process-wide.
static PRODUCED: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];

/// How many undeclared accesses the debug detector has refused, process-wide.
static UNDECLARED: AtomicU64 = AtomicU64::new(0);

/// How many times `group`'s producer has run in this process. A manager runs
/// a producer at most once per artifact, so across `n` artifacts this grows
/// by at most `n`.
#[must_use]
pub fn produced_count(group: AnalysisId) -> u64 {
    PRODUCED[group.index() as usize].load(Ordering::Relaxed)
}

/// How many undeclared accesses the detector has refused in this process —
/// TS-35's "zero undeclared accesses" reads this. Always 0 in release
/// builds, where the detector does not exist.
#[must_use]
pub fn undeclared_accesses() -> u64 {
    UNDECLARED.load(Ordering::Relaxed)
}

/// Why a fact query or a run failed. Every variant names the group, so a
/// test asserts the exact value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactError {
    /// A consumer (or producer) read a group outside its declared demand.
    /// Reported by the debug detector only.
    Undeclared {
        consumer: &'static str,
        group: AnalysisId,
    },
    /// A demanded group has no registered producer.
    Unregistered { group: AnalysisId },
    /// The group was not computed for this artifact: no run demanded it.
    NotComputed { group: AnalysisId },
    /// The stored table belongs to another group type with the same id.
    TypeMismatch {
        group: AnalysisId,
        expected: &'static str,
    },
}

/// The tables computed for one artifact.
pub(crate) struct Tables {
    slots: [Option<ErasedTable>; SLOTS],
    computed: Demand,
}

/// What the detector knows about the reader: its name and declared demand
/// in debug builds, nothing at all in release builds.
#[derive(Debug, Clone, Copy)]
struct Declared {
    #[cfg(debug_assertions)]
    consumer: &'static str,
    #[cfg(debug_assertions)]
    demand: Demand,
}

// The release shape carries nothing: the detector costs zero bytes and zero
// instructions outside debug builds.
#[cfg(not(debug_assertions))]
const _: () = assert!(size_of::<Declared>() == 0);

impl Declared {
    #[cfg_attr(not(debug_assertions), allow(unused_variables))]
    const fn new(consumer: &'static str, demand: Demand) -> Self {
        Self {
            #[cfg(debug_assertions)]
            consumer,
            #[cfg(debug_assertions)]
            demand,
        }
    }

    #[cfg_attr(not(debug_assertions), allow(unused_variables, clippy::unused_self))]
    #[inline]
    fn check(self, group: AnalysisId) -> Result<(), FactError> {
        #[cfg(debug_assertions)]
        if !self.demand.contains(group) {
            UNDECLARED.fetch_add(1, Ordering::Relaxed);
            return Err(FactError::Undeclared {
                consumer: self.consumer,
                group,
            });
        }
        Ok(())
    }
}

/// A read-only window onto one artifact's facts, scoped to one consumer.
///
/// In debug builds it carries the consumer's declared demand and refuses
/// everything outside it; in release builds it is one pointer.
#[derive(Clone, Copy)]
pub struct FactView<'m> {
    tables: &'m Tables,
    declared: Declared,
}

impl<'m> FactView<'m> {
    /// Group `G`'s table.
    ///
    /// # Errors
    ///
    /// [`FactError::Undeclared`] (debug builds) when `G` is outside the
    /// reader's declared demand; [`FactError::NotComputed`] when no run
    /// demanded `G`; [`FactError::TypeMismatch`] when another group type
    /// owns `G`'s id.
    pub fn get<G: FactGroup>(&self) -> Result<&'m FactTable<G>, FactError> {
        self.declared.check(G::ID)?;
        let slot = self.tables.slots[G::ID.index() as usize].as_deref();
        let table = slot.ok_or(FactError::NotComputed { group: G::ID })?;
        table
            .downcast_ref::<FactTable<G>>()
            .ok_or(FactError::TypeMismatch {
                group: G::ID,
                expected: type_name::<G>(),
            })
    }
}

/// Computes and owns the facts of one artifact.
///
/// One manager per artifact: every group is produced at most once for it,
/// however many consumers demand it (P4-1b adds invalidation after a pass).
pub struct FactManager<'r, 'x, A: ?Sized + 'static> {
    registry: &'r FactRegistry<A>,
    artifact: &'x A,
    tables: Tables,
}

impl<'r, 'x, A: ?Sized + 'static> FactManager<'r, 'x, A> {
    /// A manager for `artifact` with nothing computed yet.
    #[must_use]
    pub fn new(registry: &'r FactRegistry<A>, artifact: &'x A) -> Self {
        Self {
            registry,
            artifact,
            tables: Tables {
                slots: [const { None }; SLOTS],
                computed: Demand::NONE,
            },
        }
    }

    /// The groups computed so far.
    #[must_use]
    pub fn computed(&self) -> Demand {
        self.tables.computed
    }

    /// Compute exactly the transitive closure of `demand`, in stratum order,
    /// skipping every group already computed for this artifact. Returns the
    /// groups this call produced.
    ///
    /// # Errors
    ///
    /// [`FactError::Unregistered`] (before anything runs) when the closure
    /// names a group no producer computes; otherwise the first error a
    /// producer's view reported is impossible by construction — producers
    /// read only their declared, lower-stratum inputs.
    pub fn compute(&mut self, demand: Demand) -> Result<Demand, FactError> {
        let closure = self.registry.closure(demand);
        if let Some(group) = closure.minus(self.registry.registered()).iter().next() {
            return Err(FactError::Unregistered { group });
        }
        let pending = closure.minus(self.tables.computed);
        let producers = self.registry.producers();
        let Some(top) = producers.iter().map(|entry| entry.desc.stratum).max() else {
            return Ok(Demand::NONE);
        };
        for stratum in 0..=top {
            for entry in producers {
                let desc = entry.desc;
                if desc.stratum != stratum || !pending.contains(desc.id) {
                    continue;
                }
                let inputs = FactView {
                    tables: &self.tables,
                    declared: Declared::new(desc.name, desc.depends),
                };
                let table = (entry.run)(self.artifact, &inputs);
                PRODUCED[desc.id.index() as usize].fetch_add(1, Ordering::Relaxed);
                self.tables.slots[desc.id.index() as usize] = Some(table);
                self.tables.computed = self.tables.computed.with(desc.id);
            }
        }
        Ok(pending)
    }

    /// Compute `C`'s demand and return `C`'s view.
    ///
    /// # Errors
    ///
    /// As [`FactManager::compute`].
    pub fn prepare<C: FactConsumer>(&mut self) -> Result<FactView<'_>, FactError> {
        self.compute(C::DEMAND)?;
        Ok(self.view::<C>())
    }

    /// `C`'s view of what is computed now, declared with `C`'s demand.
    #[must_use]
    pub fn view<C: FactConsumer>(&self) -> FactView<'_> {
        FactView {
            tables: &self.tables,
            declared: Declared::new(C::NAME, C::DEMAND),
        }
    }
}
