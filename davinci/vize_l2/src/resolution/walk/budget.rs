//! One unchanged shared retained-tree budget for ordinary and For walks.

use super::Resolver;

impl<'a, 'b, S> Resolver<'a, 'b, S> {
    #[inline(always)]
    pub(super) fn advance(&mut self, depth: usize) -> bool {
        self.visited += 1;
        depth <= 64 && self.visited <= 4096
    }
}
