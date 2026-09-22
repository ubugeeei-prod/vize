use alloc::vec::Vec as StdVec;
use core::cell::OnceCell;
use core::cmp::Ordering;

use super::super::helper::Helper;
use super::Buf;
use super::call_position::AliasPositions;

impl Buf {
    pub(super) fn ordered_helpers(&self) -> StdVec<Helper> {
        let mut listed = StdVec::new();
        let mut bits = 0u64;
        let mut push = |helper: Helper| {
            if self.used & helper.bit() == 0 || bits & helper.bit() != 0 {
                return;
            }
            bits |= helper.bit();
            listed.push(helper);
        };
        for helper in self.preferred.iter().copied() {
            push(helper);
        }
        for helper in self.used_order.iter().copied() {
            push(helper);
        }
        for helper in Helper::ALL {
            push(helper);
        }
        let positions = LazyAliasPositions {
            buf: self,
            scanned: OnceCell::new(),
        };
        listed.sort_by(|left, right| {
            left.rank()
                .cmp(&right.rank())
                .then_with(|| self.order_same_rank_helper(&positions, *left, *right))
        });
        listed
    }

    fn order_same_rank_helper(
        &self,
        positions: &LazyAliasPositions<'_>,
        left: Helper,
        right: Helper,
    ) -> Ordering {
        // Alias positions are read only where the order needs them, so a
        // module without same-rank ties never scans its text.
        match left.rank() {
            2 => match (
                self.preferred_position(left),
                self.preferred_position(right),
            ) {
                (Some(left_pos), Some(right_pos)) => left_pos.cmp(&right_pos),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => match (positions.first(left), positions.first(right)) {
                    (Some(left_pos), Some(right_pos)) => left_pos.cmp(&right_pos),
                    _ => Ordering::Equal,
                },
            },
            5 => self
                .rank_five_key(positions, left)
                .cmp(&self.rank_five_key(positions, right)),
            10 if self.used & Helper::ResolveDirective.bit() != 0
                && self.used & Helper::CreateText.bit() != 0
                && create_slots_show_pair(left, right) =>
            {
                create_slots_before_v_show(left, right)
            }
            _ => Ordering::Equal,
        }
    }

    fn preferred_position(&self, helper: Helper) -> Option<usize> {
        self.preferred
            .iter()
            .position(|candidate| candidate.bit() == helper.bit())
    }

    fn rank_five_key(&self, positions: &LazyAliasPositions<'_>, helper: Helper) -> (usize, u8, u8) {
        if let Some((position, order)) = normalize_props_guard_merge_order(positions, helper) {
            return (position, order, rank_five_all_order(helper));
        }
        let position = positions
            .first(helper)
            .map(alias_sort_position)
            .or_else(|| self.virtual_alias_position(positions, helper))
            .unwrap_or_else(|| usize::MAX - 16 + usize::from(rank_five_all_order(helper)));
        (position, 0, rank_five_all_order(helper))
    }

    fn virtual_alias_position(
        &self,
        positions: &LazyAliasPositions<'_>,
        helper: Helper,
    ) -> Option<usize> {
        let index = self
            .used_order
            .iter()
            .position(|candidate| candidate.bit() == helper.bit())?;
        self.used_order[..index]
            .iter()
            .rev()
            .find_map(|candidate| positions.first(*candidate))
            .map(|position| alias_sort_position(position) + 1)
            .or_else(|| {
                self.used_order[index + 1..]
                    .iter()
                    .find_map(|candidate| positions.first(*candidate))
                    .map(|position| alias_sort_position(position).saturating_sub(1))
            })
    }
}

fn normalize_props_guard_merge_order(
    positions: &LazyAliasPositions<'_>,
    helper: Helper,
) -> Option<(usize, u8)> {
    let normalize_pos = positions.first(Helper::NormalizeProps)?;
    let merge_pos = positions.first(Helper::MergeProps)?;
    if normalize_pos >= merge_pos {
        return None;
    }
    let base = alias_sort_position(merge_pos);
    match helper {
        Helper::GuardReactiveProps
            if positions
                .first(helper)
                .is_some_and(|position| position > merge_pos) =>
        {
            Some((base, 0))
        }
        Helper::MergeProps => Some((base, 1)),
        _ => None,
    }
}

/// [`AliasPositions`] of a buffer, scanned on first read.
struct LazyAliasPositions<'b> {
    buf: &'b Buf,
    scanned: OnceCell<AliasPositions>,
}

impl LazyAliasPositions<'_> {
    fn first(&self, helper: Helper) -> Option<usize> {
        self.scanned
            .get_or_init(|| {
                AliasPositions::scan(
                    self.buf
                        .hoists
                        .iter()
                        .map(|hoist| hoist.as_str())
                        .chain(core::iter::once(self.buf.code.as_str())),
                )
            })
            .first(helper)
    }
}

fn alias_sort_position(position: usize) -> usize {
    position.saturating_mul(2)
}

fn rank_five_all_order(helper: Helper) -> u8 {
    match helper {
        Helper::NormalizeClass => 0,
        Helper::NormalizeStyle => 1,
        Helper::NormalizeProps => 2,
        Helper::GuardReactiveProps => 3,
        Helper::MergeProps => 4,
        Helper::ToHandlers => 5,
        Helper::ToHandlerKey => 6,
        Helper::Camelize => 7,
        _ => 8,
    }
}

fn create_slots_show_pair(left: Helper, right: Helper) -> bool {
    matches!(
        (left, right),
        (Helper::CreateSlots, Helper::VShow) | (Helper::VShow, Helper::CreateSlots)
    )
}

fn create_slots_before_v_show(left: Helper, _right: Helper) -> Ordering {
    if matches!(left, Helper::CreateSlots) {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}
