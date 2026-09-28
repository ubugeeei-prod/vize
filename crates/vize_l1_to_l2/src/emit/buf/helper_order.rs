use alloc::vec::Vec as StdVec;
use core::cmp::Ordering;

use super::super::helper::Helper;
use super::Buf;
use super::call_position::helper_call_position;
use super::call_position::underscore_call_sites;
use vize_l0::String;

type RankFiveKey = (usize, u8, u8);

/// Per-sort memo: each helper's first alias call position in the
/// generated module, and each rank-five key, are computed at most once
/// per [`Buf::ordered_helpers`] instead of once per comparison.
struct OrderCache {
    rank_five_keys: [Option<RankFiveKey>; 8],
    /// One full scan pays off only for modules with many helpers and code.
    scan_all_aliases: bool,
    /// Filled by one scan of the module on first use, indexed by
    /// `Helper::bit().trailing_zeros()`.
    alias_positions: Option<[Option<usize>; 64]>,
}

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
        let mut cache = OrderCache {
            rank_five_keys: [None; 8],
            scan_all_aliases: self.used.count_ones() > 8 && self.code.len() > 8_192,
            alias_positions: None,
        };
        listed.sort_by(|left, right| {
            left.rank()
                .cmp(&right.rank())
                .then_with(|| self.order_same_rank_helper(*left, *right, &mut cache))
        });
        listed
    }

    fn order_same_rank_helper(
        &self,
        left: Helper,
        right: Helper,
        cache: &mut OrderCache,
    ) -> Ordering {
        // Alias positions scan the generated module. Only ranks that consume
        // them should pay that cost; transform preference settles most ties.
        match left.rank() {
            2 => match (
                self.preferred_position(left),
                self.preferred_position(right),
            ) {
                (Some(left_pos), Some(right_pos)) => left_pos.cmp(&right_pos),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => match (
                    self.first_alias_position(left, cache),
                    self.first_alias_position(right, cache),
                ) {
                    (Some(left_pos), Some(right_pos)) => left_pos.cmp(&right_pos),
                    _ => Ordering::Equal,
                },
            },
            5 => {
                let mut key = |helper| {
                    let slot = usize::from(rank_five_all_order(helper));
                    if let Some(Some(cached)) = cache.rank_five_keys.get(slot) {
                        return *cached;
                    }
                    let key = self.rank_five_key(helper, cache);
                    if let Some(cached) = cache.rank_five_keys.get_mut(slot) {
                        *cached = Some(key);
                    }
                    key
                };
                let left_key = key(left);
                left_key.cmp(&key(right))
            }
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

    fn first_alias_position(&self, helper: Helper, cache: &mut OrderCache) -> Option<usize> {
        if !cache.scan_all_aliases {
            return self.scan_alias_position(helper);
        }
        let positions = cache
            .alias_positions
            .get_or_insert_with(|| self.scan_alias_positions());
        positions
            .get(helper.bit().trailing_zeros() as usize)
            .copied()
            .flatten()
    }

    /// Every helper's first alias call position over the hoists, then the
    /// code, from a single scan.
    fn scan_alias_positions(&self) -> [Option<usize>; 64] {
        let mut positions = [None; 64];
        let mut offset = 0;
        for text in self
            .hoists
            .iter()
            .map(String::as_str)
            .chain(core::iter::once(self.code.as_str()))
        {
            underscore_call_sites(text, |position, name| {
                if let Some(helper) = Helper::ALL.iter().find(|helper| helper.alias() == name)
                    && let Some(slot) = positions.get_mut(helper.bit().trailing_zeros() as usize)
                    && slot.is_none()
                {
                    *slot = Some(offset + position);
                }
                false
            });
            offset += text.len();
        }
        positions
    }

    fn scan_alias_position(&self, helper: Helper) -> Option<usize> {
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

    fn rank_five_key(&self, helper: Helper, cache: &mut OrderCache) -> RankFiveKey {
        if let Some((position, order)) = self.normalize_props_guard_merge_order(helper, cache) {
            return (position, order, rank_five_all_order(helper));
        }
        let position = self
            .first_alias_position(helper, cache)
            .map(alias_sort_position)
            .or_else(|| self.virtual_alias_position(helper, cache))
            .unwrap_or_else(|| usize::MAX - 16 + usize::from(rank_five_all_order(helper)));
        (position, 0, rank_five_all_order(helper))
    }

    fn normalize_props_guard_merge_order(
        &self,
        helper: Helper,
        cache: &mut OrderCache,
    ) -> Option<(usize, u8)> {
        if !matches!(helper, Helper::GuardReactiveProps | Helper::MergeProps) {
            return None;
        }
        let normalize_pos = self.first_alias_position(Helper::NormalizeProps, cache)?;
        let merge_pos = self.first_alias_position(Helper::MergeProps, cache)?;
        if normalize_pos >= merge_pos {
            return None;
        }
        let base = alias_sort_position(merge_pos);
        match helper {
            Helper::GuardReactiveProps
                if self
                    .first_alias_position(helper, cache)
                    .is_some_and(|position| position > merge_pos) =>
            {
                Some((base, 0))
            }
            Helper::MergeProps => Some((base, 1)),
            _ => None,
        }
    }

    fn virtual_alias_position(&self, helper: Helper, cache: &mut OrderCache) -> Option<usize> {
        let index = self
            .used_order
            .iter()
            .position(|candidate| candidate.bit() == helper.bit())?;
        let (before, rest) = self.used_order.split_at_checked(index)?;
        let after = rest.split_first().map_or(&[][..], |(_, after)| after);
        before
            .iter()
            .rev()
            .find_map(|candidate| self.first_alias_position(*candidate, cache))
            .map(|position| alias_sort_position(position) + 1)
            .or_else(|| {
                after
                    .iter()
                    .find_map(|candidate| self.first_alias_position(*candidate, cache))
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

#[cfg(test)]
mod tests {
    use super::{Buf, Helper};

    #[test]
    fn one_scan_finds_every_alias_where_the_per_alias_scan_does() {
        for (hoist, code) in [
            ("", "_createVNode(a); _openBlock()"),
            ("_normalizeProps (p)", "x._createVNode(a), _createVNode (b)"),
            (
                "'_mergeProps(' /* _toDisplayString( */",
                "_mergeProps(x) // _openBlock(\n_openBlock()",
            ),
            (
                "`${_toDisplayString(a)}`",
                "a_createVNode(), $_createBlock(), _createBlockX(), _createBlock\n(x)",
            ),
            ("", "\"esc\\\"_renderList(\" _renderList(l)"),
            (
                "",
                "_createElementVNode(\"div\", null, _toDisplayString(_ctx.msg))",
            ),
        ] {
            let mut buf = Buf::new(false);
            if !hoist.is_empty() {
                buf.push_hoist(hoist.into());
            }
            buf.push(code);
            let positions = buf.scan_alias_positions();
            for helper in Helper::ALL {
                assert_eq!(
                    positions
                        .get(helper.bit().trailing_zeros() as usize)
                        .copied()
                        .flatten(),
                    buf.scan_alias_position(helper),
                    "{} in {hoist:?} + {code:?}",
                    helper.alias(),
                );
            }
        }
    }
}
