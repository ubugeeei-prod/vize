//! Region navigation over the flat plan: open/close pairing, direct child
//! ranges, and the legacy child-list shape of a region.

use super::{Emitter, Result, text};
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringSegment, SsrStringSegmentKind as Kind,
};
use crate::s4::{AdmissionFailure, LegacyReason};

/// Segments that open a nested region (their close pairs by depth).
pub(super) const fn is_open(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::OpenElement
            | Kind::If
            | Kind::Branch
            | Kind::For
            | Kind::Component
            | Kind::SlotOutlet
    )
}

/// Segments that close a region.
pub(super) const fn is_close(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::CloseElement
            | Kind::CloseIf
            | Kind::CloseBranch
            | Kind::CloseFor
            | Kind::CloseComponent
            | Kind::CloseSlot
    )
}

/// A region's direct children as the legacy AST lists them.
pub(super) struct RegionShape {
    /// Legacy child count: a merged S2 text run counts one child per part,
    /// since the legacy lane never merges SSR text.
    pub(super) legacy_children: usize,
    pub(super) non_text: bool,
    /// Plan positions of the children that may inherit `_attrs`.
    pub(super) candidates: std::vec::Vec<usize>,
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// Describe the child list starting at `from`, up to its closing segment.
    pub(super) fn scan_region(&self, from: usize) -> Result<RegionShape> {
        let mut shape = RegionShape {
            legacy_children: 0,
            non_text: false,
            candidates: std::vec::Vec::new(),
        };
        for start in self.direct_children(from)? {
            let segment = self.segment(start)?;
            match segment.kind {
                Kind::Text => shape.legacy_children += 1,
                Kind::DynamicText => {
                    let Source::Interpolation(interpolation) = segment.source else {
                        return Err(LegacyReason::Operation.into());
                    };
                    shape.legacy_children +=
                        text::legacy_child_count(self.facts.texts, &segment, interpolation)?;
                    shape.non_text = true;
                }
                Kind::OpenElement | Kind::If | Kind::Component | Kind::SlotOutlet => {
                    shape.legacy_children += 1;
                    shape.non_text = true;
                    shape.candidates.push(start);
                }
                Kind::For => {
                    shape.legacy_children += 1;
                    shape.non_text = true;
                }
                Kind::Comment => {
                    shape.legacy_children += 1;
                    shape.non_text = true;
                }
                _ => return Err(LegacyReason::Operation.into()),
            }
        }
        Ok(shape)
    }

    /// The plan segment at `at`; cursors always stay inside the plan.
    pub(super) fn segment(&self, at: usize) -> Result<SsrStringSegment<'r, 'a>> {
        (self.segments.get(at).copied()).ok_or(AdmissionFailure::Invalid(
            "string plan segment is out of range",
        ))
    }

    /// Positions of the direct children of the region starting at `from`.
    pub(super) fn direct_children(&self, from: usize) -> Result<std::vec::Vec<usize>> {
        let mut children = std::vec::Vec::new();
        let mut pos = from;
        while let Some(segment) = self.segments.get(pos) {
            if is_close(segment.kind) {
                break;
            }
            children.push(pos);
            pos = self.child_end(pos)?;
        }
        Ok(children)
    }

    /// The position just past the child starting at `start`.
    pub(super) fn child_end(&self, start: usize) -> Result<usize> {
        let Some(first) = self.segments.get(start) else {
            return Err(AdmissionFailure::Invalid(
                "string plan child is out of range",
            ));
        };
        if !is_open(first.kind) {
            return Ok(start + 1);
        }
        let mut depth = 0usize;
        for (offset, segment) in self
            .segments
            .get(start..)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            if is_open(segment.kind) {
                depth += 1;
            } else if is_close(segment.kind) {
                depth -= 1;
                if depth == 0 {
                    return Ok(start + offset + 1);
                }
            }
        }
        Err(AdmissionFailure::Invalid(
            "string plan region is not closed",
        ))
    }

    /// The span of branch `index`'s extracted key on the `v-if` op whose
    /// fact is `fact`: a `:key` stays on the S2 carrier, so its segment is
    /// dropped from the branch root like the legacy transform drops it.
    pub(super) fn branch_key_span(&self, fact: u32, index: usize) -> Result<Option<vize_s0::Span>> {
        let node = vize_davinci::id::NodeId::from_index(fact).ok_or(AdmissionFailure::Invalid(
            "string plan fact index is not a node id",
        ))?;
        Ok(self
            .facts
            .if_facts
            .get(node)
            .and_then(|facts| facts.branches.get(index))
            .and_then(Option::as_ref)
            .map(|key| key.span))
    }

    /// The attached segments of the op at the cursor (which advances past
    /// them), minus the `v-if` branch key the legacy transform moved off
    /// the branch root.
    pub(super) fn take_attached(
        &mut self,
        len: usize,
    ) -> Result<std::vec::Vec<crate::s4::string_plan::SsrStringSegment<'r, 'a>>> {
        let key = self.branch_key.take();
        let end = self.pos + len;
        let attached = self
            .segments
            .get(self.pos..end)
            .ok_or(AdmissionFailure::Invalid(
                "string plan lost attached segments",
            ))?;
        self.pos = end;
        Ok(attached
            .iter()
            .copied()
            .filter(|segment| key != Some(segment.span))
            .collect())
    }

    /// Consume the segment `kind` whose operand is `owner`.
    pub(super) fn close(
        &mut self,
        kind: Kind,
        owner: impl Fn(&Source<'r, 'a>) -> bool,
    ) -> Result<()> {
        match self.segments.get(self.pos) {
            Some(close) if close.kind == kind && owner(&close.source) => {
                self.pos += 1;
                Ok(())
            }
            _ => Err(AdmissionFailure::Invalid(
                "string plan region is not closed by its owner",
            )),
        }
    }
}
