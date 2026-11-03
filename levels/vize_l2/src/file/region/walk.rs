//! Stack guard surrounds the actual factory, including pre-mint callbacks.

use super::{FileRegion, TemplatePolicy};
use crate::{artifact::RegionBuilder, file::template::TemplateWalk};
use core::ops::DerefMut;
use vize_l0::Span;

struct WalkGuard<'g, 'f, 'b, 'a, R, P> {
    region: &'g mut FileRegion<'f, 'b, 'a, R, P>,
    previous: TemplateWalk,
    armed: bool,
}

impl<R, P> Drop for WalkGuard<'_, '_, '_, '_, R, P> {
    fn drop(&mut self) {
        if self.armed {
            self.region.facts.template_walk.interrupt();
        }
    }
}

impl<'a: 'b, 'b, R, P> FileRegion<'_, 'b, 'a, R, P>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
    P: TemplatePolicy,
{
    pub(super) fn with_walk<T>(&mut self, span: Span, operation: impl FnOnce(&mut Self) -> T) -> T {
        let previous = self.facts.template_walk.enter(span);
        let mut guard = WalkGuard {
            region: self,
            previous,
            armed: true,
        };
        let result = operation(guard.region);
        guard.region.facts.template_walk.complete(guard.previous);
        // An outer Pending state must not be interrupted by normal inner Drop.
        guard.armed = false;
        result
    }
}
