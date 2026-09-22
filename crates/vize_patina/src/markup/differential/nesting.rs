//! Whether the lint parse restructured the authored element tree.
//!
//! The Relief parser applies browser HTML tree construction (a `<div>` start
//! tag closes an open `<p>`, table content is foster-parented, a nested
//! `<a>` closes the outer one, …). S1 is lossless and S2 keeps Vue's authored
//! nesting — the tree Vue's `createElement` builds at runtime. When the two
//! parses nest the same elements differently the facades present different
//! documents *by design*, so the lane counts such inputs (pinned) instead of
//! comparing them.
//!
//! The check reads the two raw parses, never a facade, so a facade bug cannot
//! hide in this bucket: both sides are reduced to the pre-order sequence of
//! `(element start, parent element start)`; the implicit table owners the
//! Relief parse inserts (zero-width) are transparent, as S1 has none.

use crate::markup::s2::surface::offset_in;
use vize_relief::{RootNode, TemplateChildNode};
use vize_s1::{SurfaceChild, SurfaceTree};

type Nesting = std::vec::Vec<(u32, Option<u32>)>;

fn relief_nesting(children: &[TemplateChildNode<'_>], parent: Option<u32>, out: &mut Nesting) {
    for child in children {
        if let TemplateChildNode::Element(element) = child {
            let span = element.loc.span;
            if span.start == span.end {
                relief_nesting(&element.children, parent, out);
            } else {
                out.push((span.start, parent));
                relief_nesting(&element.children, Some(span.start), out);
            }
        }
    }
}

fn surface_nesting(
    source: &str,
    children: &[SurfaceChild<'_>],
    parent: Option<u32>,
    out: &mut Nesting,
) {
    for child in children {
        if let SurfaceChild::Element(element) = child {
            let start = offset_in(source, element.open.lt_name.text);
            out.push((start, parent));
            surface_nesting(source, &element.children, Some(start), out);
        }
    }
}

/// Whether the Relief parse of `root` nests elements differently from the
/// authored S1 tree.
pub(super) fn is_restructured(root: &RootNode<'_>, tree: &SurfaceTree<'_>) -> bool {
    let (mut relief, mut surface) = (Nesting::new(), Nesting::new());
    relief_nesting(&root.children, None, &mut relief);
    surface_nesting(tree.source, &tree.children, None, &mut surface);
    relief != surface
}
