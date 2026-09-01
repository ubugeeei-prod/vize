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
            (2, None, None, Some(left_pos), Some(right_pos))
                if self.has_codegen_only_with_directives() =>
            {
                left_pos.cmp(&right_pos)
            }
            (5, _, _, Some(left_pos), Some(right_pos)) => left_pos.cmp(&right_pos),
            _ => Ordering::Equal,
        }
    }

    fn preferred_position(&self, helper: Helper) -> Option<usize> {
        self.preferred
            .iter()
            .position(|candidate| candidate.bit() == helper.bit())
    }

    fn has_codegen_only_with_directives(&self) -> bool {
        self.used & Helper::WithDirectives.bit() != 0
            && self.preferred_position(Helper::WithDirectives).is_none()
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
}
