use alloc::vec::Vec as StdVec;
use core::cmp::Ordering;

use super::super::helper::Helper;
use super::Buf;
use super::call_position::helper_call_position;
use super::call_position::underscore_call_sites;
use vize_l0::String;

type RankFiveKey = (usize, u8, u8);

/// Every helper's bit, so the shared scan caches the first call of each
/// alias. A constant: ordering a module must not pay to rebuild it.
const ALL_ALIASES: u64 = {
    let mut mask = 0;
    let mut rest: &[Helper] = &Helper::ALL;
    while let [helper, tail @ ..] = rest {
        mask |= helper.bit();
        rest = tail;
    }
    mask
};

struct OrderCache {
    rank_five_keys: [Option<RankFiveKey>; 8],
    alias_mask: u64,
    alias_scan: Option<AliasScan>,
}

/// The shared lexical scan, stopped where the last question was answered.
///
/// The scan reads hoists and then the body once, in order, and records the
/// first call of every alias it passes. It stops as soon as the alias asked
/// for is found and resumes from there for the next question, so a module
/// whose ordering-sensitive helpers are called early is never read to its
/// end. An alias that is never called costs the rest of the scan, once.
struct AliasScan {
    positions: [Option<usize>; 64],
    /// The bits of the aliases whose first call has been recorded.
    found: u64,
    /// The text being scanned: a hoist index, then `hoists.len()` for the
    /// body, then one past it when everything has been read.
    text: usize,
    /// Where to resume inside that text.
    resume: usize,
    /// The byte length of the texts before it.
    offset: usize,
}

impl Buf {
    pub(super) fn ordered_helpers(&self) -> StdVec<Helper> {
        let listed = self.listed_helpers();
        // The comparator asks where an alias is first called. One lexical
        // scan of the module answers every such question, and it runs only
        // when the first one is asked. Scanning per alias instead read the
        // module once per question: a third of the whole emit on a 12 KiB
        // module. Virtual positions consult transient helpers too, including
        // helpers replaced in `used_order`, so the scan caches every alias.
        // The per-alias scans stay as the debug oracle of that one scan.
        #[cfg(debug_assertions)]
        let per_alias = self.sort_helpers::<false>(listed.clone(), 0);
        let ordered = self.sort_helpers::<true>(listed, ALL_ALIASES);
        #[cfg(debug_assertions)]
        debug_assert!(
            ordered
                .iter()
                .map(|helper| helper.bit())
                .eq(per_alias.iter().map(|helper| helper.bit())),
            "one shared alias scan must order helpers as the per-alias scans do"
        );
        ordered
    }

    /// The used helpers before ranking: preferred first, then first use,
    /// then the rest in declaration order.
    fn listed_helpers(&self) -> StdVec<Helper> {
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
        listed
    }

    fn sort_helpers<const SHARED: bool>(
        &self,
        mut listed: StdVec<Helper>,
        alias_mask: u64,
    ) -> StdVec<Helper> {
        let mut cache = OrderCache {
            rank_five_keys: [None; 8],
            alias_mask,
            alias_scan: None,
        };
        listed.sort_by(|left, right| {
            left.rank()
                .cmp(&right.rank())
                .then_with(|| self.order_same_rank_helper::<SHARED>(*left, *right, &mut cache))
        });
        listed
    }

    fn order_same_rank_helper<const SHARED: bool>(
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
                    self.first_alias_position::<SHARED>(left, cache),
                    self.first_alias_position::<SHARED>(right, cache),
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
                    let key = self.rank_five_key::<SHARED>(helper, cache);
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
                match (
                    self.first_alias_occurrence(left),
                    self.first_alias_occurrence(right),
                ) {
                    (Some(left_pos), Some(right_pos)) => left_pos.cmp(&right_pos),
                    _ => create_slots_before_v_show(left, right),
                }
            }
            _ => Ordering::Equal,
        }
    }

    fn preferred_position(&self, helper: Helper) -> Option<usize> {
        self.preferred
            .iter()
            .position(|candidate| candidate.bit() == helper.bit())
    }

    fn first_alias_position<const SHARED: bool>(
        &self,
        helper: Helper,
        cache: &mut OrderCache,
    ) -> Option<usize> {
        if SHARED && cache.alias_mask & helper.bit() != 0 {
            let scan = cache.alias_scan.get_or_insert(AliasScan {
                positions: [None; 64],
                found: 0,
                text: 0,
                resume: 0,
                offset: 0,
            });
            if scan.found & helper.bit() == 0 {
                self.scan_until(scan, helper.bit());
            }
            return scan
                .positions
                .get(helper.bit().trailing_zeros() as usize)
                .copied()
                .flatten();
        }
        self.scan_alias_position(helper)
    }

    /// Continue `scan` until the alias with bit `wanted` is first called or
    /// every text has been read.
    fn scan_until(&self, scan: &mut AliasScan, wanted: u64) {
        while let Some(text) = self
            .hoists
            .get(scan.text)
            .map(String::as_str)
            .or_else(|| (scan.text == self.hoists.len()).then_some(self.code.as_str()))
        {
            let AliasScan {
                positions,
                found,
                offset,
                ..
            } = scan;
            let resume = underscore_call_sites(text, scan.resume, |position, alias| {
                let Some(helper) = Helper::from_alias(alias) else {
                    return false;
                };
                let bit = helper.bit();
                if *found & bit != 0 {
                    return false;
                }
                if let Some(slot) = positions.get_mut(bit.trailing_zeros() as usize) {
                    *slot = Some(*offset + position);
                    *found |= bit;
                }
                bit == wanted
            });
            if scan.found & wanted != 0 {
                scan.resume = resume;
                return;
            }
            scan.offset += text.len();
            scan.text += 1;
            scan.resume = 0;
        }
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

    fn first_alias_occurrence(&self, helper: Helper) -> Option<usize> {
        let alias = helper.alias();
        let mut offset = 0;
        for hoist in self.hoists.iter() {
            if let Some(position) = hoist.find(alias) {
                return Some(offset + position);
            }
            offset += hoist.len();
        }
        self.code.find(alias).map(|position| offset + position)
    }

    fn rank_five_key<const SHARED: bool>(
        &self,
        helper: Helper,
        cache: &mut OrderCache,
    ) -> RankFiveKey {
        if let Some((position, order)) =
            self.normalize_props_guard_merge_order::<SHARED>(helper, cache)
        {
            return (position, order, rank_five_all_order(helper));
        }
        let position = self
            .first_alias_position::<SHARED>(helper, cache)
            .map(alias_sort_position)
            .or_else(|| self.virtual_alias_position::<SHARED>(helper, cache))
            .unwrap_or_else(|| usize::MAX - 16 + usize::from(rank_five_all_order(helper)));
        (position, 0, rank_five_all_order(helper))
    }

    fn normalize_props_guard_merge_order<const SHARED: bool>(
        &self,
        helper: Helper,
        cache: &mut OrderCache,
    ) -> Option<(usize, u8)> {
        if !matches!(helper, Helper::GuardReactiveProps | Helper::MergeProps) {
            return None;
        }
        let normalize_pos = self.first_alias_position::<SHARED>(Helper::NormalizeProps, cache)?;
        let merge_pos = self.first_alias_position::<SHARED>(Helper::MergeProps, cache)?;
        if normalize_pos >= merge_pos {
            return None;
        }
        let base = alias_sort_position(merge_pos);
        match helper {
            Helper::GuardReactiveProps
                if self
                    .first_alias_position::<SHARED>(helper, cache)
                    .is_some_and(|position| position > merge_pos) =>
            {
                Some((base, 0))
            }
            Helper::MergeProps => Some((base, 1)),
            _ => None,
        }
    }

    fn virtual_alias_position<const SHARED: bool>(
        &self,
        helper: Helper,
        cache: &mut OrderCache,
    ) -> Option<usize> {
        let index = self
            .used_order
            .iter()
            .position(|candidate| candidate.bit() == helper.bit())?;
        let (before, rest) = self.used_order.split_at_checked(index)?;
        let after = rest.split_first().map_or(&[][..], |(_, after)| after);
        before
            .iter()
            .rev()
            .find_map(|candidate| self.first_alias_position::<SHARED>(*candidate, cache))
            .map(|position| alias_sort_position(position) + 1)
            .or_else(|| {
                after
                    .iter()
                    .find_map(|candidate| self.first_alias_position::<SHARED>(*candidate, cache))
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
    use super::{Buf, Helper, OrderCache};

    #[test]
    fn shared_scan_matches_each_alias_scan_across_lexical_boundaries() {
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
        ] {
            let mut buf = Buf::new(false);
            for helper in Helper::ALL {
                buf.use_helper(helper);
            }
            if !hoist.is_empty() {
                buf.push_hoist(hoist.into());
            }
            buf.push(code);
            // The scan stops at each answer and resumes for the next
            // question, so the order of the questions must not matter.
            let mut forward = Helper::ALL.to_vec();
            let mut backward = forward.clone();
            backward.reverse();
            forward.rotate_left(7);
            for questions in [Helper::ALL.to_vec(), backward, forward] {
                let mut cache = OrderCache {
                    rank_five_keys: [None; 8],
                    alias_mask: super::ALL_ALIASES,
                    alias_scan: None,
                };
                for helper in questions {
                    assert_eq!(
                        buf.first_alias_position::<true>(helper, &mut cache),
                        buf.scan_alias_position(helper),
                        "{} in {hoist:?} + {code:?}",
                        helper.alias(),
                    );
                }
            }
        }
    }

    #[test]
    fn large_helper_module_keeps_ordinary_order_with_one_scan() {
        let mut buf = Buf::new(false);
        for helper in Helper::ALL.into_iter().rev() {
            buf.use_helper(helper);
            buf.push(helper.alias());
            buf.push("();");
        }
        buf.push("/* _mergeProps() */ '_normalizeProps()' object._camelize();");
        buf.push(&"x".repeat(65_536));

        let shared = buf.ordered_helpers();
        let mut listed = Helper::ALL.to_vec();
        listed.reverse();
        let ordinary = buf.sort_helpers::<false>(listed, 0);

        assert!(
            shared
                .iter()
                .map(|helper| helper.bit())
                .eq(ordinary.iter().map(|helper| helper.bit()))
        );
        let mut cache = OrderCache {
            rank_five_keys: [None; 8],
            alias_mask: buf.used,
            alias_scan: None,
        };
        assert!(
            buf.first_alias_position::<true>(Helper::MergeProps, &mut cache)
                .is_some()
        );
        assert_eq!(
            buf.first_alias_position::<true>(Helper::CreateVNode, &mut cache),
            buf.scan_alias_position(Helper::CreateVNode)
        );
    }

    /// The one shared scan is the only product path now, at every module
    /// size. Generated modules mix used, preferred, called, hoisted, quoted
    /// and never-called helpers; each must order exactly as the per-alias
    /// scans order it.
    #[test]
    fn one_scan_orders_generated_modules_as_per_alias_scans_do() {
        // Fixed-seed xorshift: the corpus is the same on every run.
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = |bound: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % bound as u64) as usize
        };
        let mut orders_with_alias_keys = 0_usize;
        for _ in 0..4_000 {
            let mut buf = Buf::new(false);
            let mut chosen = std::vec::Vec::new();
            for _ in 0..next(14) {
                let helper = Helper::ALL[next(Helper::ALL.len())];
                chosen.push(helper);
                match next(5) {
                    0 => buf.prefer(helper),
                    1 => {
                        buf.use_helper(helper);
                        let mut hoist = vize_l0::String::from(helper.alias());
                        hoist.push_str("(h)");
                        buf.push_hoist(hoist);
                    }
                    2 => {
                        buf.use_helper(helper);
                        buf.push("'");
                        buf.push(helper.alias());
                        buf.push("(' + ");
                    }
                    3 => buf.use_helper(helper),
                    _ => {
                        buf.use_helper(helper);
                        buf.push(helper.alias());
                        buf.push(if next(2) == 0 { "(x), " } else { " (x)\n" });
                    }
                }
            }
            let per_alias = buf.sort_helpers::<false>(buf.listed_helpers(), 0);
            let ordered = buf.ordered_helpers();
            assert!(
                ordered
                    .iter()
                    .map(|helper| helper.bit())
                    .eq(per_alias.iter().map(|helper| helper.bit())),
                "{:?}",
                buf.code
            );
            orders_with_alias_keys += usize::from(
                chosen
                    .iter()
                    .filter(|helper| matches!(helper.rank(), 2 | 5))
                    .count()
                    > 1,
            );
        }
        // The corpus is not vacuous: many modules need alias positions.
        assert!(orders_with_alias_keys > 1_000, "{orders_with_alias_keys}");
    }

    #[test]
    fn direct_alias_lookup_covers_every_helper() {
        for helper in Helper::ALL {
            assert_eq!(
                Helper::from_alias(helper.alias()).map(Helper::bit),
                Some(helper.bit())
            );
        }
        assert!(Helper::from_alias("_notAHelper").is_none());
    }
}
