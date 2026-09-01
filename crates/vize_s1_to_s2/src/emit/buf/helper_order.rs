use alloc::vec::Vec as StdVec;
use core::cmp::Ordering;

use super::super::helper::Helper;
use super::Buf;
use super::call_position::helper_call_position;

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
        listed.sort_by(|left, right| {
            left.rank()
                .cmp(&right.rank())
                .then_with(|| self.order_same_rank_helper(*left, *right))
        });
        listed
    }

    fn order_same_rank_helper(&self, left: Helper, right: Helper) -> Ordering {
        match (
            left.rank(),
            self.preferred_position(left),
            self.preferred_position(right),
            self.first_alias_position(left),
            self.first_alias_position(right),
        ) {
            (2, Some(left_pos), Some(right_pos), _, _) => left_pos.cmp(&right_pos),
            (2, Some(_), None, _, _) => Ordering::Less,
            (2, None, Some(_), _, _) => Ordering::Greater,
            (2, None, None, Some(left_pos), Some(right_pos)) => left_pos.cmp(&right_pos),
            (5, _, _, _, _) => self.rank_five_key(left).cmp(&self.rank_five_key(right)),
            (10, _, _, _, _)
                if self.used & Helper::ResolveDirective.bit() != 0
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

    fn first_alias_position(&self, helper: Helper) -> Option<usize> {
        let alias = helper.alias();
        let mut offset = 0;
        for hoist in self.hoists.iter() {
            if let Some(position) = helper_call_position(hoist, alias) {
                return Some(offset + position);
            }
            offset += hoist.len();
        }
        helper_call_position(self.code.as_str(), alias).map(|position| offset + position)
    }

    fn rank_five_key(&self, helper: Helper) -> (usize, u8, u8) {
        if let Some((position, order)) = self.normalize_props_guard_merge_order(helper) {
            return (position, order, rank_five_all_order(helper));
        }
        let position = self
            .first_alias_position(helper)
            .map(alias_sort_position)
            .or_else(|| self.virtual_alias_position(helper))
            .unwrap_or_else(|| usize::MAX - 16 + usize::from(rank_five_all_order(helper)));
        (position, 0, rank_five_all_order(helper))
    }

    fn normalize_props_guard_merge_order(&self, helper: Helper) -> Option<(usize, u8)> {
        let normalize_pos = self.first_alias_position(Helper::NormalizeProps)?;
        let merge_pos = self.first_alias_position(Helper::MergeProps)?;
        if normalize_pos >= merge_pos {
            return None;
        }
        let base = alias_sort_position(merge_pos);
        match helper {
            Helper::GuardReactiveProps
                if self
                    .first_alias_position(helper)
                    .is_some_and(|position| position > merge_pos) =>
            {
                Some((base, 0))
            }
            Helper::MergeProps => Some((base, 1)),
            _ => None,
        }
    }

    fn virtual_alias_position(&self, helper: Helper) -> Option<usize> {
        let index = self
            .used_order
            .iter()
            .position(|candidate| candidate.bit() == helper.bit())?;
        self.used_order[..index]
            .iter()
            .rev()
            .find_map(|candidate| self.first_alias_position(*candidate))
            .map(|position| alias_sort_position(position) + 1)
            .or_else(|| {
                self.used_order[index + 1..]
                    .iter()
                    .find_map(|candidate| self.first_alias_position(*candidate))
                    .map(|position| alias_sort_position(position).saturating_sub(1))
            })
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
