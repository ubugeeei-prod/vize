//! A forward cursor over disjoint groups and their original member slices.
use super::{FusionGroup, PassDesc, Pipeline, Preserved};
use core::iter::FusedIterator;
mod prefix;

impl Pipeline {
    /// Visit successive walks together with their original pass descriptions.
    /// A started group reads only the remaining suffix. Stopping iteration
    /// leaves later groups unexamined. No plan or list of members is allocated.
    pub fn grouped_passes(&self) -> impl FusedIterator<Item = (FusionGroup, &'static [PassDesc])> {
        GroupedPasses {
            remaining: self.passes,
            start: 0,
        }
    }
}

struct GroupedPasses<'a> {
    remaining: &'a [PassDesc],
    start: usize,
}

impl<'a> Iterator for GroupedPasses<'a> {
    type Item = (FusionGroup, &'a [PassDesc]);

    fn next(&mut self) -> Option<Self::Item> {
        let (len, barrier, mask) = prefix::next_group(self.remaining, |pass| {
            (pass.fusability.is_fusable(), pass.preserved.to_bits())
        })?;
        let (members, remaining) = self.remaining.split_at(len);
        let group = FusionGroup {
            start: self.start,
            len,
            is_barrier: barrier,
            preserved: Preserved::from_bits(mask),
        };
        self.remaining = remaining;
        self.start = group.end();
        Some((group, members))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            usize::from(!self.remaining.is_empty()),
            Some(self.remaining.len()),
        )
    }
}

impl FusedIterator for GroupedPasses<'_> {}
