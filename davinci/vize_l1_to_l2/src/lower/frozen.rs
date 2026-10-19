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
mod tests {
    //! Whole-source membership laws for the sole explicit compiler lowering entry.

    use super::{BOUNDS, EXTRA, FOREIGN, MISSING};
    use crate::lower::{lower, lower_with_frozen_element_spans};
    use alloc::{borrow::ToOwned, vec};
    use vize_l0::{Allocator, SourceRoot, Span, cstr};
    use vize_l2::op::Op;

    #[test]
    fn absent_and_explicit_empty_custody_have_distinct_contracts() {
        let arena = Allocator::new();
        let source = "<slot v-pre></slot>";
        let (tree, errors) = vize_l1::parse(&arena, source);
        let original = lower(&arena, &tree, &errors);
        assert!(matches!(&original.root.ops[0], Op::Slot(_)));
        let block = SourceRoot::new(source).unwrap().whole_block();
        assert_eq!(
            lower_with_frozen_element_spans(&arena, &tree, &errors, block, &[]).err(),
            Some(MISSING)
        );
        let (normal, errors) = vize_l1::parse(&arena, "<slot></slot>");
        let block = SourceRoot::new(normal.source).unwrap().whole_block();
        let explicit =
            lower_with_frozen_element_spans(&arena, &normal, &errors, block, &[]).unwrap();
        let default = lower(&arena, &normal, &errors);
        assert_eq!(cstr!("{explicit:?}"), cstr!("{default:?}"));
    }

    #[test]
    fn complete_opening_membership_changes_only_the_explicit_owner() {
        let arena = Allocator::new();
        let source = "<slot v-pre></slot>";
        let (tree, errors) = vize_l1::parse(&arena, source);
        let block = SourceRoot::new(source).unwrap().whole_block();
        let spans = [Span::new(0, 12)];
        let explicit =
            lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).unwrap();
        assert!(
            matches!(&explicit.root.ops[0], Op::Element(element) if element.tag == "slot" && element.children.ops.is_empty())
        );
        assert!(matches!(
            &lower(&arena, &tree, &errors).root.ops[0],
            Op::Slot(_)
        ));
    }

    #[test]
    fn foreign_whole_block_and_nonzero_blocks_refuse_without_tree_mutation() {
        let arena = Allocator::new();
        let source = "<slot v-pre></slot>".to_owned();
        let copy = source.clone();
        let (tree, errors) = vize_l1::parse(&arena, &source);
        let before = cstr!("{tree:?}");
        let foreign = SourceRoot::new(&copy).unwrap().whole_block();
        assert_eq!(
            lower_with_frozen_element_spans(&arena, &tree, &errors, foreign, &[]).err(),
            Some(FOREIGN)
        );
        assert_eq!(cstr!("{tree:?}"), before);
        let root = "prefix<slot v-pre></slot>";
        let block = SourceRoot::new(root).unwrap().block(&root[6..], 6).unwrap();
        let (tree, errors) = vize_l1::parse(&arena, block.source());
        let before = cstr!("{tree:?}");
        assert_eq!(
            lower_with_frozen_element_spans(&arena, &tree, &errors, block, &[]).err(),
            Some(FOREIGN)
        );
        assert_eq!(cstr!("{tree:?}"), before);
    }

    #[test]
    fn duplicate_overlapping_out_of_bounds_and_non_frozen_spans_refuse() {
        let arena = Allocator::new();
        let source = "<slot v-pre></slot><i></i>";
        let (tree, errors) = vize_l1::parse(&arena, source);
        let block = SourceRoot::new(source).unwrap().whole_block();
        let before = cstr!("{tree:?}");
        for spans in [
            vec![Span::new(0, 12), Span::new(0, 12)],
            vec![Span::new(0, 12), Span::new(11, 13)],
            vec![Span::new(0, 999)],
            vec![Span::new(12, 0)],
        ] {
            assert_eq!(
                lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err(),
                Some(BOUNDS)
            );
        }
        for spans in [
            vec![Span::new(0, 11)],
            vec![Span::new(0, 12), Span::new(19, 22)],
        ] {
            let error =
                lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err();
            assert_eq!(error, Some(if spans.len() == 1 { MISSING } else { EXTRA }));
        }
        assert_eq!(cstr!("{tree:?}"), before);
    }

    #[test]
    fn non_pre_opening_custody_is_not_an_arbitrary_literal_permission() {
        let arena = Allocator::new();
        let source = "<Child></Child>";
        let (tree, errors) = vize_l1::parse(&arena, source);
        let block = SourceRoot::new(source).unwrap().whole_block();
        let spans = [Span::new(0, 7)];
        assert_eq!(
            lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err(),
            Some(EXTRA)
        );
        assert!(matches!(
            &lower(&arena, &tree, &errors).root.ops[0],
            Op::Component(_)
        ));
    }

    #[test]
    fn explicit_utf8_boundary_and_extra_nonopening_ranges_refuse() {
        let arena = Allocator::new();
        let source = "<div v-pre>ä</div>";
        let (tree, errors) = vize_l1::parse(&arena, source);
        let block = SourceRoot::new(source).unwrap().whole_block();
        let spans = [Span::new(0, 12)];
        assert_eq!(
            lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err(),
            Some(BOUNDS)
        );
        let spans = [Span::new(0, 11), Span::new(11, 13)];
        assert_eq!(
            lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err(),
            Some(EXTRA)
        );
    }
}
