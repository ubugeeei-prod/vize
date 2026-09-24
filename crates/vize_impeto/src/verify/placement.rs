//! S3V010: placement alternatives and committed choices.
//!
//! Each record is re-derived from the canonical graph rather than trusted:
//! an alternative is accepted only where it preserves meaning, and a
//! committed choice only where the plan it forms keeps effect order.

use alloc::vec::Vec;
use vize_s0::{Span, String, cstr};

use super::{Violation, ViolationCode};
use crate::op::{Op, OpId, OpKind, Program};
use crate::placement::facts::{Index, contains};
use crate::placement::{Placement, PlacementRecord};

pub(super) fn check(program: &Program<'_>, out: &mut Vec<Violation>) {
    if program.placements.is_empty() {
        return;
    }
    let index = Index::new(program);
    let mut records: Vec<(OpId, u32)> = program
        .placements
        .iter()
        .enumerate()
        .map(|(position, record)| (record.op, position as u32))
        .collect();
    records.sort_by_key(|entry| entry.0);
    let plan = Plan {
        index: &index,
        records: &records,
    };
    let mut previous = None;
    for record in &program.placements {
        let (Some(op), Some(position)) = (index.op(record.op), index.position(record.op)) else {
            push(
                out,
                Span::new(0, 0),
                cstr!("placement for {} does not resolve", record.op),
            );
            continue;
        };
        if previous.is_some_and(|previous| previous >= position) {
            push(
                out,
                op.span,
                cstr!("placement for {} is not in op order", record.op),
            );
        }
        previous = Some(position);
        // A choice is judged only once its alternatives are legal, so one
        // defect yields one violation.
        let message = plan
            .alternatives(record, op, position)
            .or_else(|| plan.choice(record, op));
        if let Some(message) = message {
            push(out, op.span, message);
        }
    }
}

fn push(out: &mut Vec<Violation>, span: Span, message: String) {
    out.push(Violation {
        code: ViolationCode::Placement,
        span,
        message,
    });
}

struct Plan<'i, 'p, 'a> {
    index: &'i Index<'p, 'a>,
    records: &'i [(OpId, u32)],
}

impl Plan<'_, '_, '_> {
    fn record(&self, op: OpId) -> Option<&PlacementRecord> {
        let start = self.records.partition_point(|entry| entry.0 < op);
        let (id, position) = *self.records.get(start)?;
        (id == op)
            .then(|| self.index.program.placements.get(position as usize))
            .flatten()
    }

    fn alternatives(&self, record: &PlacementRecord, op: &Op, position: usize) -> Option<String> {
        let set = record.alternatives;
        if !set.contains(Placement::Inline) || set.len() < 2 {
            return Some(cstr!(
                "placement for {} must list inline and another alternative",
                op.id
            ));
        }
        if !set.contains(record.chosen) {
            return Some(cstr!(
                "placement for {} chooses {} outside its alternatives",
                op.id,
                record.chosen
            ));
        }
        match (record.leader, set.contains(Placement::Group)) {
            (Some(leader), false) => {
                return Some(cstr!(
                    "placement for {} names leader {} without the group alternative",
                    op.id,
                    leader
                ));
            }
            (None, true) => {
                return Some(cstr!(
                    "placement for {} lists group without a leader",
                    op.id
                ));
            }
            _ => {}
        }
        if set.contains(Placement::Hoist)
            && let Some(message) = self.hoist(op, position)
        {
            return Some(message);
        }
        if set.contains(Placement::Cache)
            && let Some(message) = self.cache(op)
        {
            return Some(message);
        }
        record
            .leader
            .and_then(|leader| self.group(op, position, leader))
    }

    fn hoist(&self, op: &Op, position: usize) -> Option<String> {
        let index = self.index;
        if op.kind != OpKind::InsertNode {
            return Some(cstr!("hoist for {} requires an insert-node", op.id));
        }
        if !index.shape_static(op) {
            return Some(cstr!(
                "hoist for {} requires a literal node without instance attributes",
                op.id
            ));
        }
        if index.controller(op.region).is_none() {
            return Some(cstr!("hoist for {} is outside every control region", op.id));
        }
        let covered = index
            .program
            .ops
            .iter()
            .enumerate()
            .find(|(other_position, other)| {
                *other_position != position
                    && ((other.region == op.region && contains(op.span, other.span))
                        || index.under(other.region, op.id))
                    && !index.shape_static(other)
            });
        covered.map(|(_, other)| cstr!("hoist for {} covers non-static {}", op.id, other.id))
    }

    fn cache(&self, op: &Op) -> Option<String> {
        let index = self.index;
        if op.kind != OpKind::SetEvent {
            return Some(cstr!("cache for {} requires a set-event", op.id));
        }
        if op.effect.is_none() {
            return Some(cstr!("cache for {} is already static", op.id));
        }
        if !index.plain_handler(op) {
            return Some(cstr!(
                "cache for {} requires one plain js handler under a static event name",
                op.id
            ));
        }
        if !index.scope_free(op.region) {
            return Some(cstr!(
                "cache for {} may read for, slot, or component scope",
                op.id
            ));
        }
        None
    }

    fn group(&self, op: &Op, position: usize, leader: OpId) -> Option<String> {
        let index = self.index;
        let Some(reference) = index.reference(op) else {
            return Some(cstr!(
                "group for {} requires a dynamic direct-reference leaf update",
                op.id
            ));
        };
        let Some(head) = index.op(leader) else {
            return Some(cstr!(
                "group leader {} for {} does not resolve",
                leader,
                op.id
            ));
        };
        let Some(head_reference) = index.reference(head) else {
            return Some(cstr!(
                "group leader {} for {} is not a dynamic direct-reference leaf update",
                leader,
                op.id
            ));
        };
        if index.position(leader).is_none_or(|head| head >= position) {
            return Some(cstr!("group leader {} does not precede {}", leader, op.id));
        }
        if head_reference != reference {
            return Some(cstr!(
                "group for {} reads `{}` but leader {} reads `{}`",
                op.id,
                reference,
                leader,
                head_reference
            ));
        }
        let scope = index.group_scope(op.region);
        if scope.is_none() || scope != index.group_scope(head.region) {
            return Some(cstr!(
                "group for {} leaves the root, if-branch, or for-item scope of leader {}",
                op.id,
                leader
            ));
        }
        if self
            .record(leader)
            .is_some_and(|record| record.alternatives.contains(Placement::Group))
        {
            return Some(cstr!(
                "group leader {} for {} is itself grouped",
                leader,
                op.id
            ));
        }
        let listed = |record: &PlacementRecord| record.alternatives.contains(Placement::Group);
        (!self.reaches(op.id, leader, listed)).then(|| {
            cstr!(
                "group for {} is not contiguous with leader {}",
                op.id,
                leader
            )
        })
    }

    fn choice(&self, record: &PlacementRecord, op: &Op) -> Option<String> {
        match (record.chosen, record.leader) {
            (Placement::Group, Some(leader)) => {
                let committed = |record: &PlacementRecord| record.chosen == Placement::Group;
                (!self.reaches(op.id, leader, committed)).then(|| {
                    cstr!(
                        "chosen group for {} is not contiguous with committed leader {}",
                        op.id,
                        leader
                    )
                })
            }
            (Placement::Hoist, _) => self.hoisted_ancestor(op).map(|ancestor| {
                cstr!(
                    "chosen hoist for {} is nested inside hoisted {}",
                    op.id,
                    ancestor
                )
            }),
            _ => None,
        }
    }

    /// Walk keyed effect-order predecessors from `op` back to `leader`,
    /// crossing only ops `member` places in the same group.
    fn reaches(&self, op: OpId, leader: OpId, member: impl Fn(&PlacementRecord) -> bool) -> bool {
        let mut current = self.index.keyed_pred(op);
        for _ in 0..self.index.program.ops.len() {
            match current {
                Some(pred) if pred == leader => return true,
                Some(pred)
                    if self
                        .record(pred)
                        .is_some_and(|record| member(record) && record.leader == Some(leader)) =>
                {
                    current = self.index.keyed_pred(pred);
                }
                _ => return false,
            }
        }
        false
    }

    fn hoisted_ancestor(&self, op: &Op) -> Option<OpId> {
        let index = self.index;
        let mut region = index.region(op.region);
        for _ in 0..=index.program.regions.len() {
            let owner = region?.owner?;
            if self
                .record(owner)
                .is_some_and(|record| record.chosen == Placement::Hoist)
            {
                return Some(owner);
            }
            region = index.region(index.op(owner)?.region);
        }
        None
    }
}
