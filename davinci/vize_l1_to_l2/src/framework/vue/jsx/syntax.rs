//! Sparse pre-realization JSX facts, keyed by exact allocated object identity.

use alloc::vec::Vec;
use vize_l0::Span;
use vize_l2::op::{ComponentOp, ElementOp};

pub(super) trait SyntaxSink {
    fn slot_range(&mut self, _element: &ElementOp<'_>, _authored: Span) {}
    fn intrinsic_component(&mut self, _component: &ComponentOp<'_>) {}
    fn finish(&mut self) {}
}

/// The ordinary producer has no fact tables or recording work.
pub(super) struct NoSyntax;
impl SyntaxSink for NoSyntax {}

/// Observations retained before a synthetic slot range is widened or an
/// explicit intrinsic component is promoted. Facts cannot be constructed or
/// attached by callers; queries accept only the exact original arena object.
/// Source-facing authority additionally requires the original parse factory.
pub struct JsxSyntaxFacts<'a> {
    source: &'a str,
    slot_ranges: Vec<(usize, Span)>,
    intrinsic_components: Vec<usize>,
}

impl<'a> JsxSyntaxFacts<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            source,
            slot_ranges: Vec::new(),
            intrinsic_components: Vec::new(),
        }
    }

    /// Complete source supplied to the original lowering invocation.
    pub fn source(&self) -> &'a str {
        self.source
    }

    /// Original slot-function diagnostic range for this exact minted element.
    pub fn slot_diagnostic_range(&self, element: &ElementOp<'_>) -> Option<Span> {
        let address = core::ptr::from_ref(element).addr();
        self.slot_ranges
            .binary_search_by_key(&address, |(address, _)| *address)
            .ok()
            .and_then(|index| self.slot_ranges.get(index).map(|(_, span)| *span))
    }

    /// Whether this exact component came from an explicit lowercase intrinsic.
    pub fn component_was_intrinsic(&self, component: &ComponentOp<'_>) -> bool {
        self.intrinsic_components
            .binary_search(&core::ptr::from_ref(component).addr())
            .is_ok()
    }
}

impl SyntaxSink for JsxSyntaxFacts<'_> {
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
