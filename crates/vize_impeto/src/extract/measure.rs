//! The extraction cost model: three metrics over one placement plan.
//!
//! The S3 update model runs every effect unit when any key it reads changes,
//! and re-renders every op under an `if` or `for` when that op's keys change.
//! A key is one distinct reactive-read operand (kind plus source text), the
//! fact approximation extraction measures with: identical text is one
//! dependency, different text is conservatively another.
//!
//! - **emitted size**: operand payload bytes plus placement overhead bytes.
//! - **reactive edges**: the sum over effect units of the keys each reads.
//! - **update path**: the op executions the whole key set triggers, one change
//!   at a time: each unit costs its members plus every live op an `if` or
//!   `for` member re-renders, once per key the unit reads.

use alloc::vec::Vec;
use core::fmt;

use crate::op::OpKind;
use crate::placement::Placement;
use crate::placement::facts::{Index, contains, reads};

/// `_renderEffect(() => ` plus `)`: the wrapper one effect unit costs.
pub const EFFECT_UNIT_BYTES: u64 = 21;
/// `const _hoisted_1 = ` plus `_hoisted_1`: one hoisted root's declaration and use.
pub const HOIST_BYTES: u64 = 29;
/// `_cache[0] || (_cache[0] = ` plus `)`: one cached handler's guard.
pub const CACHE_BYTES: u64 = 27;

/// One measured metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    EmittedSize,
    ReactiveEdge,
    UpdatePath,
}

impl Metric {
    /// Constraints first, then the objective, as `[target.p3-10]` pins them.
    pub const CHECK_ORDER: [Self; 3] = [Self::ReactiveEdge, Self::UpdatePath, Self::EmittedSize];

    /// Stable `budgets.toml` spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmittedSize => "emitted-size",
            Self::ReactiveEdge => "reactive-edge",
            Self::UpdatePath => "update-path",
        }
    }
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The three metrics of one plan.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Metrics {
    pub emitted_size: u64,
    pub reactive_edges: u64,
    pub update_path: u64,
}

impl Metrics {
    /// One metric's value.
    #[must_use]
    pub const fn get(self, metric: Metric) -> u64 {
        match metric {
            Metric::EmittedSize => self.emitted_size,
            Metric::ReactiveEdge => self.reactive_edges,
            Metric::UpdatePath => self.update_path,
        }
    }
}

/// One op's placement inside a trial plan; `leader` is an op position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Choice {
    pub(crate) placement: Placement,
    pub(crate) leader: Option<usize>,
}

impl Choice {
    pub(crate) const INLINE: Self = Self {
        placement: Placement::Inline,
        leader: None,
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Footprint {
    Live,
    Root,
    Interior,
}

/// Plan-independent facts, computed once per extraction.
pub(crate) struct Model<'i, 'p, 'a> {
    index: &'i Index<'p, 'a>,
    payload: u64,
    /// `(op position, key id)`, sorted and unique.
    keys: Vec<(u32, u32)>,
    op_region: Vec<Option<usize>>,
    region_owner: Vec<Option<usize>>,
    region_parent: Vec<Option<usize>>,
    /// `(owner op position, region position)`, sorted.
    owned: Vec<(usize, usize)>,
}

impl<'i, 'p, 'a> Model<'i, 'p, 'a> {
    pub(crate) fn new(index: &'i Index<'p, 'a>) -> Self {
        let program = index.program;
        let mut texts: Vec<(u8, &'a str)> = Vec::new();
        let mut payload = 0u64;
        for operand in program.operands.iter() {
            if index.position(operand.op).is_none() {
                continue;
            }
            payload += (operand.value.text.len() + operand.name.map_or(0, str::len)) as u64;
            if reads(operand) {
                texts.push((operand.value.kind as u8, operand.value.text));
            }
        }
        texts.sort_unstable();
        texts.dedup();
        let mut keys: Vec<(u32, u32)> = program
            .operands
            .iter()
            .filter(|operand| reads(operand))
            .filter_map(|operand| {
                let position = index.position(operand.op)?;
                let key = texts
                    .binary_search(&(operand.value.kind as u8, operand.value.text))
                    .ok()?;
                Some((position as u32, key as u32))
            })
            .collect();
        keys.sort_unstable();
        keys.dedup();
        let op_region = program
            .ops
            .iter()
            .map(|op| index.region_position(op.region))
            .collect();
        let region_owner = program
            .regions
            .iter()
            .map(|region| region.owner.and_then(|owner| index.position(owner)))
            .collect();
        let region_parent = program
            .regions
            .iter()
            .map(|region| {
                region
                    .parent
                    .and_then(|parent| index.region_position(parent))
            })
            .collect();
        let mut owned: Vec<(usize, usize)> = program
            .regions
            .iter()
            .enumerate()
            .filter_map(|(position, region)| Some((index.position(region.owner?)?, position)))
            .collect();
        owned.sort_unstable();
        Self {
            index,
            payload,
            keys,
            op_region,
            region_owner,
            region_parent,
            owned,
        }
    }

    /// Measure `plan`, one choice per op position.
    pub(crate) fn measure(&self, plan: &[Choice]) -> Metrics {
        let ops = &self.index.program.ops;
        let footprint = self.footprint(plan);
        let mut weight = alloc::vec![0u64; self.region_parent.len()];
        for (position, state) in footprint.iter().enumerate() {
            if *state != Footprint::Interior {
                self.walk_up(self.op_region[position], |region| weight[region] += 1);
            }
        }
        let mut unit: Vec<Option<usize>> = Vec::with_capacity(ops.len());
        let mut cost = alloc::vec![0u64; ops.len()];
        let (mut units, mut roots, mut cached) = (0u64, 0u64, 0u64);
        for (position, op) in ops.iter().enumerate() {
            let choice = plan[position];
            roots += u64::from(footprint[position] == Footprint::Root);
            cached += u64::from(choice.placement == Placement::Cache);
            let own = op.effect.is_some()
                && footprint[position] == Footprint::Live
                && choice.placement != Placement::Cache;
            // A member joins its leader's unit only while the leader heads one.
            let head = match (own, choice.placement, choice.leader) {
                (false, _, _) => None,
                (true, Placement::Group, Some(leader))
                    if leader < position && unit[leader] == Some(leader) =>
                {
                    Some(leader)
                }
                (true, _, _) => Some(position),
            };
            unit.push(head);
            let Some(head) = head else {
                continue;
            };
            units += u64::from(head == position);
            cost[head] += 1;
            if matches!(op.kind, OpKind::If | OpKind::For) {
                let start = self.owned.partition_point(|entry| entry.0 < position);
                cost[head] += self.owned[start..]
                    .iter()
                    .take_while(|entry| entry.0 == position)
                    .map(|entry| weight[entry.1])
                    .sum::<u64>();
            }
        }
        let mut subscriptions: Vec<(usize, u32)> = self
            .keys
            .iter()
            .filter_map(|(position, key)| Some((unit[*position as usize]?, *key)))
            .collect();
        subscriptions.sort_unstable();
        subscriptions.dedup();
        Metrics {
            emitted_size: self.payload
                + EFFECT_UNIT_BYTES * units
                + HOIST_BYTES * roots
                + CACHE_BYTES * cached,
            reactive_edges: subscriptions.len() as u64,
            update_path: subscriptions.iter().map(|(head, _)| cost[*head]).sum(),
        }
    }

    /// Hoisted roots and every op their subtree covers.
    fn footprint(&self, plan: &[Choice]) -> Vec<Footprint> {
        let ops = &self.index.program.ops;
        let hoisted = |position: usize| plan[position].placement == Placement::Hoist;
        let mut footprint: Vec<Footprint> = (0..ops.len())
            .map(|position| {
                let mut covered = false;
                self.walk_up(self.op_region[position], |region| {
                    covered |= self.region_owner[region].is_some_and(hoisted);
                });
                match (covered, hoisted(position)) {
                    (true, _) => Footprint::Interior,
                    (false, true) => Footprint::Root,
                    (false, false) => Footprint::Live,
                }
            })
            .collect();
        for (root, op) in ops.iter().enumerate() {
            if footprint[root] != Footprint::Root {
                continue;
            }
            for (position, other) in ops.iter().enumerate() {
                if position != root && other.region == op.region && contains(op.span, other.span) {
                    footprint[position] = Footprint::Interior;
                }
            }
        }
        footprint
    }

    /// Visit `region` and each ancestor, bounded by the region table.
    fn walk_up(&self, region: Option<usize>, mut visit: impl FnMut(usize)) {
        let mut current = region;
        for _ in 0..self.region_parent.len() {
            let Some(region) = current else {
                return;
            };
            visit(region);
            current = self.region_parent[region];
        }
    }
}
