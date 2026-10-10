//! Private custody for the original admitted empty literal v-pre slot.
//!
//! This trusts the compiler's original parser bundle and its freshly lowered
//! template artifact. It is not an authentication scheme for public mutable
//! ASTs. Direct caller-built L2 receives no fact, even for identical offsets.

use vize_armature::parser::FrozenElements;
use vize_l0::Span;
use vize_l2::op::{ElementOp, Region};
use vize_relief::RootNode;

const FOREIGN_SOURCE: &str =
    "Frozen literal slot provenance belongs to a different template source.";
const FOREIGN_ARTIFACT: &str = "Frozen literal slot facts belong to a different L2 artifact.";

/// A validated borrowed view of the existing parser sidecar; no allocation.
#[derive(Clone, Copy)]
pub(super) struct FrozenTemplate<'a> {
    source: &'a str,
    spans: &'a [Span],
}

impl<'a> FrozenTemplate<'a> {
    pub(super) fn new(
        root: &RootNode<'a>,
        source: &'a str,
        frozen: &FrozenElements<'a>,
    ) -> Result<Self, &'static str> {
        let spans = frozen
            .spans_for(root.source, source)
            .ok_or(FOREIGN_SOURCE)?;
        Ok(Self { source, spans })
    }

    pub(super) fn lower(
        self,
        allocator: &'a vize_l0::Allocator,
        tree: &vize_l1::SurfaceTree<'a>,
        errors: &[vize_l1::SurfaceError],
    ) -> Result<vize_l1_to_l2::Lowered<'a>, &'static str> {
        let source = vize_l0::SourceRoot::new(self.source).map_err(|_| FOREIGN_SOURCE)?;
        vize_l1_to_l2::lower_with_frozen_element_spans(
            allocator,
            tree,
            errors,
            source.whole_block(),
            self.spans,
        )
    }
}

/// Only the source bridge mints this alongside its actual lowered Region.
/// The selector validates that same artifact before any lowering or context.
pub(super) struct FrozenSlotFacts<'s, 'a> {
    source: &'a str,
    root: &'s Region<'a>,
    spans: &'s [Span],
}

impl<'s, 'a> FrozenSlotFacts<'s, 'a> {
    pub(super) fn new(
        template: FrozenTemplate<'a>,
        source: &'a str,
        root: &'s Region<'a>,
    ) -> Result<Self, &'static str> {
        if !core::ptr::eq(template.source, source) {
            return Err(FOREIGN_SOURCE);
        }
        Ok(Self {
            source,
            root,
            spans: template.spans,
        })
    }

    pub(super) fn validate(&self, source: &str, root: &Region<'_>) -> Result<(), &'static str> {
        if !core::ptr::eq(self.source, source) || !core::ptr::eq(self.root, root) {
            return Err(FOREIGN_ARTIFACT);
        }
        Ok(())
    }

    /// Called for the actual ElementOp borrowed by the validated string plan.
    /// Opening custody comes from Parser, while ElementOp.span includes its
    /// closing tag. Compare the shared start and bounds without scanning again.
    pub(super) fn permits(&self, element: &ElementOp<'_>) -> bool {
        if element.tag != "slot"
            || !element.bindings.is_empty()
            || !element.children.ops.is_empty()
            || element.span.start >= element.span.end
            || element.span.end as usize > self.source.len()
        {
            return false;
        }
        let Ok(index) = self
            .spans
            .binary_search_by_key(&element.span.start, |span| span.start)
        else {
            return false;
        };
        let Some(opening) = self.spans.get(index) else {
            return false;
        };
        let Some(start) = (opening.start as usize).checked_add(1) else {
            return false;
        };
        let Some(end) = start.checked_add(element.tag.len()) else {
            return false;
        };
        opening.end <= element.span.end
            && end <= opening.end as usize
            && self
                .source
                .get(start..end)
                .is_some_and(|tag| core::ptr::eq(tag, element.tag))
    }
}

#[cfg(test)]
mod tests;
