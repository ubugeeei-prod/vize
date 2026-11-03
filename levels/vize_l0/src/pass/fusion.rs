//! Pipelines as const data, and the fusion grouping computed from them.
//!
//! A [`Pipeline`] is a `&'static [PassDesc]` and a name — no registry, no
//! trait objects, nothing allocated. Everything this module computes is a
//! `const fn`, which is the point: a pipeline's fusion plan can be pinned in a
//! `const` item, so a grouping regression is a **compile error** rather than a
//! runtime surprise nobody notices.
//!
//! # The grouping rule
//!
//! Walk the passes in order and start a new group when either the current pass
//! or the previous one is a [`Barrier`](super::Fusability::Barrier). A barrier therefore always
//! sits alone: it neither joins the run before it nor admits the run after it.
//! Adjacent [`Fusable`](super::Fusability::Fusable) passes collect into one
//! group, which is one walk.
//!
//! Since a mandatory pass is a barrier by construction
//! ([`PassDesc::new`](super::PassDesc::new)), "a mandatory pass never joins a
//! fusion group" is a consequence of the rule rather than a second check that
//! could disagree with it.
//!
//! A group's preserved set is the **intersection** of its members'
//! ([`Preserved::intersect`]): fusing passes cannot preserve more than the
//! least-preserving member does.

use super::{PassDesc, Preserved};

/// A run of passes that share one walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FusionGroup {
    /// Index of the group's first pass in the pipeline.
    pub start: usize,
    /// How many passes the group holds. Always at least 1.
    pub len: usize,
    /// What every pass in the group preserves.
    pub preserved: Preserved,
    /// Whether the group is a single barrier pass.
    pub is_barrier: bool,
}

impl FusionGroup {
    /// Index one past the group's last pass.
    #[inline]
    #[must_use]
    pub const fn end(&self) -> usize {
        self.start + self.len
    }
}

/// A pipeline of passes over one stage, as const data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pipeline {
    /// The stage this pipeline runs over, e.g. `s2` or `s2-to-s3`.
    pub stage: &'static str,
    /// The passes, in execution order.
    pub passes: &'static [PassDesc],
}

impl Pipeline {
    /// A pipeline of `passes` over `stage`.
    #[inline]
    #[must_use]
    pub const fn new(stage: &'static str, passes: &'static [PassDesc]) -> Self {
        Self { stage, passes }
    }

    /// How many walks running this pipeline costs.
    ///
    /// This is the number the budget observer (P2-3) reports and
    /// `budgets.toml [traversal]` gates, so it is deliberately derivable
    /// without running anything.
    #[must_use]
    pub const fn group_count(&self) -> usize {
        let mut groups = 0;
        let mut rest = self.passes;
        let mut previous_was_barrier = true;
        while let [pass, tail @ ..] = rest {
            let barrier = !pass.fusability.is_fusable();
            if barrier || previous_was_barrier {
                groups += 1;
            }
            previous_was_barrier = barrier;
            rest = tail;
        }
        groups
    }

    /// The `index`-th fusion group, or `None` past the end.
    #[must_use]
    pub const fn group(&self, index: usize) -> Option<FusionGroup> {
        let mut seen = 0;
        let mut start = 0;
        while let Some((_, [first, ..])) = self.passes.split_at_checked(start) {
            let len = self.group_len_at(start);
            if seen == index {
                return Some(FusionGroup {
                    start,
                    len,
                    preserved: self.preserved_from(start, len),
                    is_barrier: !first.fusability.is_fusable(),
                });
            }
            seen += 1;
            start += len;
        }
        None
    }

    /// How many passes the group starting at `start` holds (at least one,
    /// so a walk over the groups always advances).
    const fn group_len_at(&self, start: usize) -> usize {
        let Some((_, [first, rest @ ..])) = self.passes.split_at_checked(start) else {
            return 1;
        };
        if !first.fusability.is_fusable() {
            return 1;
        }
        let mut len = 1;
        let mut tail = rest;
        while let [pass, more @ ..] = tail {
            if !pass.fusability.is_fusable() {
                break;
            }
            len += 1;
            tail = more;
        }
        len
    }

    /// The intersection of the preserved sets over `passes[start..start + len]`.
    /// An empty range preserves nothing.
    const fn preserved_from(&self, start: usize, len: usize) -> Preserved {
        let Some((_, from)) = self.passes.split_at_checked(start) else {
            return Preserved::NONE;
        };
        let group = match from.split_at_checked(len) {
            Some((group, _)) => group,
            None => from,
        };
        let [first, rest @ ..] = group else {
            return Preserved::NONE;
        };
        let mut preserved = first.preserved;
        let mut tail = rest;
        while let [pass, more @ ..] = tail {
            preserved = preserved.intersect(pass.preserved);
            tail = more;
        }
        preserved
    }

    /// The group `passes[index]` belongs to, or `None` if `index` is past the
    /// end.
    #[must_use]
    pub const fn group_of_pass(&self, index: usize) -> Option<usize> {
        if index >= self.passes.len() {
            return None;
        }
        let mut group = 0;
        let mut start = 0;
        while start < self.passes.len() {
            let len = self.group_len_at(start);
            if index < start + len {
                return Some(group);
            }
            group += 1;
            start += len;
        }
        None
    }

    /// Whether every pass this pipeline runs is a barrier — i.e. fusion buys
    /// nothing here.
    #[must_use]
    pub const fn is_fully_serialized(&self) -> bool {
        self.group_count() == self.passes.len()
    }
}

/// Iterating groups without `const` evaluation, for renderers and tests.
impl Pipeline {
    /// The groups, in order.
    pub fn groups(&self) -> impl Iterator<Item = FusionGroup> + '_ {
        let count = self.group_count();
        // A group index below `group_count` always resolves.
        (0..count).filter_map(move |index| self.group(index))
    }

    /// The passes of `group`, in order.
    #[must_use]
    pub fn passes_of(&self, group: FusionGroup) -> &'static [PassDesc] {
        self.passes
            .get(group.start..group.end())
            .unwrap_or_default()
    }
}
