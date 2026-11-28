//! Optional original facts captured by the projection before realization.

use vize_l0::Span;
use vize_l2::op::{ComponentOp, ElementOp};

pub(super) trait ProjectionSink {
    fn slot_range(&mut self, _element: &ElementOp<'_>, _authored: Span) {}
    fn intrinsic_component(&mut self, _component: &ComponentOp<'_>) {}
    fn finish(&mut self) {}
}

pub(super) struct NoSyntax;
impl ProjectionSink for NoSyntax {}

pub(crate) struct ProjectionFacts {
    slot_ranges: std::vec::Vec<(usize, Span)>,
    intrinsic_components: std::vec::Vec<usize>,
}

impl ProjectionFacts {
    pub(super) fn new() -> Self {
        Self {
            slot_ranges: std::vec::Vec::new(),
            intrinsic_components: std::vec::Vec::new(),
        }
    }

    pub(crate) fn slot_diagnostic_range(&self, element: &ElementOp<'_>) -> Option<Span> {
        let address = core::ptr::from_ref(element).addr();
        self.slot_ranges
            .binary_search_by_key(&address, |(address, _)| *address)
            .ok()
            .and_then(|index| self.slot_ranges.get(index).map(|(_, range)| *range))
    }

    pub(crate) fn component_was_intrinsic(&self, component: &ComponentOp<'_>) -> bool {
        self.intrinsic_components
            .binary_search(&core::ptr::from_ref(component).addr())
            .is_ok()
    }
}

impl ProjectionSink for ProjectionFacts {
    fn slot_range(&mut self, element: &ElementOp<'_>, authored: Span) {
        self.slot_ranges
            .push((core::ptr::from_ref(element).addr(), authored));
    }

    fn intrinsic_component(&mut self, component: &ComponentOp<'_>) {
        self.intrinsic_components
            .push(core::ptr::from_ref(component).addr());
    }

    fn finish(&mut self) {
        self.slot_ranges
            .sort_unstable_by_key(|(address, _)| *address);
        self.intrinsic_components.sort_unstable();
    }
}
