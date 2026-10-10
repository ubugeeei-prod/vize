//! Closed-default source custody for the compiler-owned whole-template entry.

use super::{Allocator, LegacyCaps, Lowered, cx::Cx, finish};
use vize_l0::{SourceBlock, Span};
use vize_l1::{Element, SurfaceError, SurfaceTree};

const FOREIGN: &str = "Frozen lowering custody belongs to a different whole template source.";
const BOUNDS: &str = "Frozen lowering opening spans are unordered, overlapping or out of bounds.";
const MISSING: &str = "Frozen lowering custody is missing an authored frozen opening.";
const EXTRA: &str = "Frozen lowering custody contains an unmatched or non-frozen opening.";

/// Only the explicit compiler entry constructs Pending; public defaults stay Absent.
/// The trusted original compiler bundle is not a forge-proof public AST contract.
#[derive(Default)]
pub(super) enum FrozenOpenings<'a> {
    #[default]
    Absent,
    Pending(&'a [Span]),
    Failed(&'static str),
}

impl<'a> FrozenOpenings<'a> {
    pub(super) fn new(
        tree: &SurfaceTree<'a>,
        block: SourceBlock<'a>,
        spans: &'a [Span],
    ) -> Result<Self, &'static str> {
        if !core::ptr::eq(tree.source, block.source())
            || block.start() != 0
            || !core::ptr::eq(block.source(), block.root_source())
        {
            return Err(FOREIGN);
        }
        let mut previous_end = 0;
        for span in spans {
            if span.start < previous_end
                || span.start >= span.end
                || !block.contains_block_span(*span)
            {
                return Err(BOUNDS);
            }
            previous_end = span.end;
        }
        Ok(Self::Pending(spans))
    }

    fn take(&mut self, opening: Option<Span>, frozen: bool) -> bool {
        let Self::Pending(remaining) = self else {
            return false;
        };
        let Some(opening) = opening else {
            *self = Self::Failed(FOREIGN);
            return false;
        };
        let next = remaining.first().copied();
        if frozen {
            if next != Some(opening) {
                *self = Self::Failed(MISSING);
                return false;
            }
            let Some((_, tail)) = remaining.split_first() else {
                return false;
            };
            *remaining = tail;
            true
        } else {
            if next.is_some_and(|span| span.start <= opening.start) {
                *self = Self::Failed(EXTRA);
            }
            false
        }
    }

    pub(super) fn complete(&self) -> Result<(), &'static str> {
        match self {
            Self::Absent | Self::Pending([]) => Ok(()),
            Self::Pending(_) => Err(EXTRA),
            Self::Failed(message) => Err(message),
        }
    }
}

impl<'a> Cx<'a> {
    /// Consume only the actual element walk, never lookahead or synthetic owners.
    pub(super) fn frozen_literal(&mut self, element: &Element<'a>, frozen: bool) -> bool {
        if matches!(self.frozen_openings, FrozenOpenings::Absent) {
            return false;
        }
        let opening = self
            .source_block()
            .span_of(element.open.lt_name.text)
            .zip(self.source_block().span_of(element.open.gt.text))
            .map(|(name, end)| Span::new(name.start, end.end));
        self.frozen_openings.take(opening, frozen)
    }
}

/// Compiler-owned lowering with complete, source-bound frozen opening custody.
///
/// All ordinary entries preserve their original default behavior. This explicit
/// trusted bundle accepts only the exact base-zero whole template. Missing,
/// extra or foreign custody is rejected before an artifact escapes; mutable
/// public input objects are not a cryptographic authentication boundary.
#[doc(hidden)]
pub fn lower_with_frozen_element_spans<'a>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    block: SourceBlock<'a>,
    spans: &'a [Span],
) -> Result<Lowered<'a>, &'static str> {
    let openings = FrozenOpenings::new(tree, block, spans)?;
    let mut cx = Cx::with_source_block_and_comment_policy(
        allocator,
        block,
        LegacyCaps::VUE3,
        false,
        &[],
        None,
    );
    cx.frozen_openings = openings;
    finish::lower_in_frozen_context(cx, tree, errors, block)
}

#[cfg(test)]
mod tests;
