//! [`FactManager`] — computes and owns one artifact's facts.

use super::registry::FactRegistry;
use super::view::{Declared, Tables, count_production};
use super::{Demand, FactConsumer, FactError, FactView};

/// Computes and owns the facts of one artifact.
///
/// The manager does not hold the artifact: passes mutate it between
/// queries, so every computing call takes the artifact's **current** state.
/// Every group is produced at most once per artifact state, however many
/// consumers demand it; [`FactManager::after_pass`] (P4-1b) is the only way
/// a computed group goes away.
pub struct FactManager<'r, A: ?Sized + 'static> {
    pub(crate) registry: &'r FactRegistry<A>,
    pub(crate) tables: Tables,
}

impl<'r, A: ?Sized + 'static> FactManager<'r, A> {
    /// A manager with nothing computed yet.
    #[must_use]
    pub fn new(registry: &'r FactRegistry<A>) -> Self {
        Self {
            registry,
            tables: Tables::new(),
        }
    }

    /// The groups computed (and still valid) now.
    #[must_use]
    pub fn computed(&self) -> Demand {
        self.tables.computed
    }

    /// Compute exactly the transitive closure of `demand` over `artifact`,
    /// in stratum order, skipping every group already computed. Returns the
    /// groups this call produced.
    ///
    /// # Errors
    ///
    /// [`FactError::Unregistered`] (before anything runs) when the closure
    /// names a group no producer computes.
    pub fn compute(&mut self, artifact: &A, demand: Demand) -> Result<Demand, FactError> {
        let pending = self.pending(demand)?;
        compute_into(self.registry, &mut self.tables, artifact, pending, true);
        Ok(pending)
    }

    /// Compute `C`'s demand over `artifact` and return `C`'s view.
    ///
    /// # Errors
    ///
    /// As [`FactManager::compute`].
    pub fn prepare<C: FactConsumer>(&mut self, artifact: &A) -> Result<FactView<'_>, FactError> {
        self.compute(artifact, C::DEMAND)?;
        Ok(self.view::<C>())
    }

    /// `C`'s view of what is computed now, declared with `C`'s demand.
    #[must_use]
    pub fn view<C: FactConsumer>(&self) -> FactView<'_> {
        FactView::new(&self.tables, Declared::new(C::NAME, C::DEMAND))
    }

    /// The closure of `demand` minus what is already computed.
    fn pending(&self, demand: Demand) -> Result<Demand, FactError> {
        let closure = self.registry.closure(demand);
        match closure.minus(self.registry.registered()).iter().next() {
            Some(group) => Err(FactError::Unregistered { group }),
            None => Ok(closure.minus(self.tables.computed)),
        }
    }
}

/// Run every producer in `pending` into `tables`, in stratum order and
/// registration order inside a stratum. `pending` must be closed over
/// dependencies not already in `tables`; `count` records productions (a
/// verification recompute is not one).
pub(crate) fn compute_into<A: ?Sized + 'static>(
    registry: &FactRegistry<A>,
    tables: &mut Tables,
    artifact: &A,
    pending: Demand,
    count: bool,
) {
    if pending.is_empty() {
        return;
    }
    let producers = registry.producers();
    let top = producers
        .iter()
        .map(|entry| entry.desc.stratum)
        .max()
        .unwrap_or(0);
    for stratum in 0..=top {
        for entry in producers {
            let desc = entry.desc;
            if desc.stratum != stratum || !pending.contains(desc.id) {
                continue;
            }
            let inputs = FactView::new(tables, Declared::new(desc.name, desc.depends));
            let table = (entry.run)(artifact, &inputs);
            if count {
                count_production(desc.id);
            }
            tables.store(desc.id, table);
        }
    }
}
