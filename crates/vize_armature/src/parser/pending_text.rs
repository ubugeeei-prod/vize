//! Buffered text runs: build-then-freeze for decoded template text.
//!
//! A text node's `content` is an `&\'a str` (Davinci P1-10), so a run whose
//! decoded bytes diverge from the source — an entity rewrote them — cannot be
//! appended to in place. Recopying the run into the arena per entity would be
//! quadratic, so the decoded bytes accumulate here and are frozen once, at the
//! first tokenizer callback that is not itself text.

use vize_relief::TemplateChildNode;
use vize_s0::String;

use super::Parser;

/// A text run buffered before it is frozen into its node's `&\'a str`.
pub(in crate::parser) struct PendingText {
    /// Decoded bytes accumulated so far.
    pub(in crate::parser) buf: String,
    /// Container holding the node this run belongs to.
    pub(in crate::parser) slot: TextSlot,
    /// Start offset of the node\'s source span.
    pub(in crate::parser) start: usize,
}

/// Which child list holds the text node a [`PendingText`] run belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::parser) enum TextSlot {
    Stack(usize),
    Fostered(usize),
    Root,
}

impl<'a> Parser<'a> {
    /// The container [`Parser::add_child`] would append to right now.
    pub(in crate::parser) fn current_text_slot(&self) -> TextSlot {
        if self.stack.is_empty() {
            TextSlot::Root
        } else {
            TextSlot::Stack(self.stack.len() - 1)
        }
    }

    /// Freeze a buffered text run into its node\'s arena-resident content.
    ///
    /// The run is a contiguous source span, so the decoded bytes usually equal
    /// the span verbatim and the node can borrow the template text; only a run
    /// an entity actually rewrote pays an arena copy.
    pub(in crate::parser) fn flush_pending_text(&mut self) {
        let Some(pending) = self.pending_text.take() else {
            return;
        };
        let (source, allocator) = (self.source, self.allocator);
        let children = match pending.slot {
            TextSlot::Stack(index) => match self.stack.get_mut(index) {
                Some(entry) => &mut entry.element.children,
                None => return,
            },
            TextSlot::Fostered(index) => match self.stack.get_mut(index) {
                Some(entry) => &mut entry.fostered_before,
                None => return,
            },
            TextSlot::Root => match self.root.as_mut() {
                Some(root) => &mut root.children,
                None => return,
            },
        };
        let Some(TemplateChildNode::Text(node)) = children.last_mut() else {
            return;
        };
        let end = node.loc.span.end as usize;
        let slice = source.get(pending.start..end);
        node.content = match slice {
            Some(slice) if slice == pending.buf => slice,
            _ => allocator.alloc_str(&pending.buf),
        };
    }
}
