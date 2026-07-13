//! Parent navigation for the mutable template transform traversal.

use vize_carton::Vec;

use crate::{ElementNode, ForNode, IfBranchNode, IfNode, RootNode, TemplateChildNode};

/// Enum for parent node types.
#[derive(Clone, Copy)]
pub enum ParentNode<'a> {
    Root(*mut RootNode<'a>),
    Element(*mut ElementNode<'a>),
    If(*mut IfNode<'a>),
    IfBranch(*mut IfBranchNode<'a>),
    For(*mut ForNode<'a>),
}

impl<'a> ParentNode<'a> {
    /// Get mutable access to children through raw pointer.
    ///
    /// # Safety
    /// `ParentNode` stores pointers into the bump-allocated template tree so the
    /// traversal can mutate parents without cloning children on every visit. The
    /// active traversal loop owns the only mutable child slice for the current
    /// parent, and nested calls only use descendants created from that slice.
    /// Keeping the invariant here avoids an allocation-heavy zipper structure in
    /// the hottest template transform path.
    #[allow(clippy::mut_from_ref)]
    pub fn children_mut(&self) -> &mut Vec<'a, TemplateChildNode<'a>> {
        // SAFETY: every pointer is produced from a live `RootNode`, `ElementNode`,
        // `IfBranchNode`, or `ForNode` borrowed by the transform driver. The
        // transform is single-threaded and never keeps two active mutable child
        // slices for the same parent; sibling traversal advances only after the
        // previous borrow has been consumed. `ParentNode::If` intentionally has no
        // children slice, so that variant is rejected before any pointer deref.
        unsafe {
            match self {
                ParentNode::Root(r) => &mut (*(*r)).children,
                ParentNode::Element(e) => &mut (*(*e)).children,
                ParentNode::If(_) => {
                    // Panic path by design: callers must traverse concrete
                    // branches (`ParentNode::IfBranch`) because an `IfNode`
                    // itself is only a branch container and has no direct child
                    // vector to return.
                    panic!("IfNode doesn't have direct children")
                }
                ParentNode::IfBranch(b) => &mut (*(*b)).children,
                ParentNode::For(f) => &mut (*(*f)).children,
            }
        }
    }
}
