//! Group descriptors, the producer registry, and the stratification check.

use alloc::boxed::Box;
use core::any::Any;

use super::manager::FactView;
use super::{Demand, FactProducer};
use crate::pass::{AnalysisId, MAX_ANALYSES};

/// One fact group as const data — what the stratification check reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupDesc {
    /// The group's identity.
    pub id: AnalysisId,
    /// The group's stable name.
    pub name: &'static str,
    /// The group's stratum.
    pub stratum: u8,
    /// The groups the group's producer reads.
    pub depends: Demand,
}

/// Why a set of group descriptors is not a valid registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrataError {
    /// Two registered groups share an id.
    DuplicateId { id: AnalysisId },
    /// A group depends on a group no registered producer computes.
    UnregisteredDependency {
        group: AnalysisId,
        dependency: AnalysisId,
    },
    /// A group depends on a group in its own or a higher stratum — the only
    /// way a demand cycle could be written, so it is rejected outright.
    NotStrictlyLower {
        group: AnalysisId,
        dependency: AnalysisId,
    },
}

/// The stratification check: every dependency is registered and sits in a
/// strictly lower stratum, and no id is registered twice.
///
/// A `const fn`, so [`FactRegistry::new`] runs it at compile time; exposed
/// on its own so the exact error of a rejected set can be asserted.
///
/// # Errors
///
/// The first violation in registration order.
pub const fn check_strata(groups: &[GroupDesc]) -> Result<(), StrataError> {
    let mut i = 0;
    while i < groups.len() {
        let group = groups[i];
        let mut j = 0;
        while j < i {
            if groups[j].id.index() == group.id.index() {
                return Err(StrataError::DuplicateId { id: group.id });
            }
            j += 1;
        }
        let mut index = 0;
        while index < MAX_ANALYSES {
            let dependency = AnalysisId::new(index);
            if group.depends.contains(dependency) {
                match stratum_of(groups, dependency) {
                    None => {
                        return Err(StrataError::UnregisteredDependency {
                            group: group.id,
                            dependency,
                        });
                    }
                    Some(stratum) if stratum >= group.stratum => {
                        return Err(StrataError::NotStrictlyLower {
                            group: group.id,
                            dependency,
                        });
                    }
                    Some(_) => {}
                }
            }
            index += 1;
        }
        i += 1;
    }
    Ok(())
}

const fn stratum_of(groups: &[GroupDesc], id: AnalysisId) -> Option<u8> {
    let mut i = 0;
    while i < groups.len() {
        if groups[i].id.index() == id.index() {
            return Some(groups[i].stratum);
        }
        i += 1;
    }
    None
}

/// A table with its group type erased, as the manager stores it.
pub(crate) type ErasedTable = Box<dyn Any + Send + Sync>;

/// A registered producer: its group's descriptor and how to run it over an
/// artifact of type `A`. Dispatch is one function pointer per group per
/// artifact — per pipeline, never per node (guardrail 1).
pub struct ProducerEntry<A: ?Sized> {
    /// The group this entry computes.
    pub desc: GroupDesc,
    pub(crate) run: fn(&A, &FactView<'_>) -> ErasedTable,
}

impl<A: ?Sized> Clone for ProducerEntry<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: ?Sized> Copy for ProducerEntry<A> {}

impl<A: ?Sized> core::fmt::Debug for ProducerEntry<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ProducerEntry")
            .field("desc", &self.desc)
            .finish_non_exhaustive()
    }
}

impl<A: ?Sized> ProducerEntry<A> {
    /// The entry for group `G`'s producer.
    #[must_use]
    pub const fn of<G: FactProducer<A>>() -> Self {
        Self {
            desc: G::DESC,
            run: run_erased::<A, G>,
        }
    }
}

fn run_erased<A: ?Sized, G: FactProducer<A>>(artifact: &A, inputs: &FactView<'_>) -> ErasedTable {
    Box::new(G::produce(artifact, inputs))
}

/// Every producer a manager may run over artifacts of type `A`, checked for
/// stratification when the registry is built.
///
/// Build it as a `const` item: [`FactRegistry::new`] panics on a violation,
/// and a panic in `const` evaluation is a compile error. A group that
/// depends on a group of its own stratum does not compile:
///
/// ```compile_fail,E0080
/// use vize_davinci::fact::{Demand, FactGroup, FactProducer, FactRegistry, FactTable, FactView, ProducerEntry};
/// use vize_davinci::pass::AnalysisId;
///
/// struct Scopes;
/// impl FactGroup for Scopes {
///     const ID: AnalysisId = AnalysisId::new(0);
///     const NAME: &'static str = "scopes";
///     const STRATUM: u8 = 1;
///     const DEPENDS: Demand = Demand::NONE;
///     type Key = u32;
///     type Value = u32;
/// }
/// struct Bindings;
/// impl FactGroup for Bindings {
///     const ID: AnalysisId = AnalysisId::new(1);
///     const NAME: &'static str = "bindings";
///     const STRATUM: u8 = 1; // same stratum as its dependency
///     const DEPENDS: Demand = Demand::NONE.with(Scopes::ID);
///     type Key = u32;
///     type Value = u32;
/// }
/// impl FactProducer<str> for Scopes {
///     fn produce(_: &str, _: &FactView<'_>) -> FactTable<Self> { FactTable::default() }
/// }
/// impl FactProducer<str> for Bindings {
///     fn produce(_: &str, _: &FactView<'_>) -> FactTable<Self> { FactTable::default() }
/// }
/// const REGISTRY: FactRegistry<str> =
///     FactRegistry::new(&[ProducerEntry::of::<Scopes>(), ProducerEntry::of::<Bindings>()]);
/// ```
///
/// Nor does the other half of a would-be cycle — a group reading a group in a
/// higher stratum:
///
/// ```compile_fail,E0080
/// use vize_davinci::fact::{Demand, FactGroup, FactProducer, FactRegistry, FactTable, FactView, ProducerEntry};
/// use vize_davinci::pass::AnalysisId;
///
/// struct Low;
/// impl FactGroup for Low {
///     const ID: AnalysisId = AnalysisId::new(0);
///     const NAME: &'static str = "low";
///     const STRATUM: u8 = 0;
///     const DEPENDS: Demand = Demand::NONE.with(AnalysisId::new(1)); // reads High
///     type Key = u32;
///     type Value = u32;
/// }
/// struct High;
/// impl FactGroup for High {
///     const ID: AnalysisId = AnalysisId::new(1);
///     const NAME: &'static str = "high";
///     const STRATUM: u8 = 1;
///     const DEPENDS: Demand = Demand::NONE.with(Low::ID);
///     type Key = u32;
///     type Value = u32;
/// }
/// impl FactProducer<str> for Low {
///     fn produce(_: &str, _: &FactView<'_>) -> FactTable<Self> { FactTable::default() }
/// }
/// impl FactProducer<str> for High {
///     fn produce(_: &str, _: &FactView<'_>) -> FactTable<Self> { FactTable::default() }
/// }
/// const REGISTRY: FactRegistry<str> =
///     FactRegistry::new(&[ProducerEntry::of::<Low>(), ProducerEntry::of::<High>()]);
/// ```
pub struct FactRegistry<A: ?Sized + 'static> {
    producers: &'static [ProducerEntry<A>],
}

impl<A: ?Sized + 'static> FactRegistry<A> {
    /// A registry over `producers`.
    ///
    /// # Panics
    ///
    /// On any [`StrataError`] — a compile error when evaluated in a `const`
    /// item, which is how registries are meant to be built.
    #[must_use]
    pub const fn new(producers: &'static [ProducerEntry<A>]) -> Self {
        assert!(
            producers.len() <= MAX_ANALYSES as usize,
            "a registry holds at most MAX_ANALYSES groups"
        );
        let mut descs = [GroupDesc {
            id: AnalysisId::new(0),
            name: "",
            stratum: 0,
            depends: Demand::NONE,
        }; MAX_ANALYSES as usize];
        let mut i = 0;
        while i < producers.len() {
            descs[i] = producers[i].desc;
            i += 1;
        }
        match check_strata(descs.split_at(producers.len()).0) {
            Ok(()) => {}
            Err(StrataError::DuplicateId { .. }) => {
                panic!("fact registry: two producers register the same group id")
            }
            Err(StrataError::UnregisteredDependency { .. }) => {
                panic!("fact registry: a group depends on a group no producer computes")
            }
            Err(StrataError::NotStrictlyLower { .. }) => panic!(
                "fact registry: a group depends on a group in its own or a higher stratum \
                 (a demand cycle is unrepresentable)"
            ),
        }
        Self { producers }
    }

    /// The registered producers, in registration order.
    #[must_use]
    pub const fn producers(&self) -> &'static [ProducerEntry<A>] {
        self.producers
    }

    /// The registered entry for `id`.
    #[must_use]
    pub fn entry(&self, id: AnalysisId) -> Option<&'static ProducerEntry<A>> {
        self.producers.iter().find(|entry| entry.desc.id == id)
    }

    /// Every registered group.
    #[must_use]
    pub fn registered(&self) -> Demand {
        self.producers
            .iter()
            .fold(Demand::NONE, |set, entry| set.with(entry.desc.id))
    }

    /// `demand` plus everything it transitively depends on. Groups outside
    /// the registry stay in the result so the caller can report them.
    #[must_use]
    pub fn closure(&self, demand: Demand) -> Demand {
        let mut closed = demand;
        loop {
            let next = closed.iter().fold(closed, |set, id| match self.entry(id) {
                Some(entry) => set.union(entry.desc.depends),
                None => set,
            });
            if next == closed {
                return closed;
            }
            closed = next;
        }
    }
}
