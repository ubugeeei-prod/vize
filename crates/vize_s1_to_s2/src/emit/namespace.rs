//! DOM namespace threading for SVG / MathML render emission.
//!
//! Vue's runtime infers SVG / MathML creation from the contiguous vnode tree:
//! a boundary element must open its own block, while descendants that remain
//! in the same namespace stay plain VNodes.

use vize_s0::ensure_sufficient_stack;
use vize_s2::op::{ElementOp, Namespace, Op, Region};

use super::{EmitCx, EmitError};

pub(super) fn child_namespace(element: &ElementOp<'_>) -> Namespace {
    match element.namespace {
        Namespace::Svg if matches!(element.tag, "foreignObject" | "desc" | "title") => {
            Namespace::Html
        }
        Namespace::MathMl
            if matches!(
                element.tag,
                "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext"
            ) =>
        {
            Namespace::Html
        }
        namespace => namespace,
    }
}

pub(super) fn crosses_boundary(
    cx: &EmitCx<'_>,
    element: &ElementOp<'_>,
    direct_static_children_hoisted: bool,
) -> bool {
    element.namespace != cx.parent_ns
        || child_namespace_crosses(element)
        || structural_children_cross_boundary(
            element.namespace,
            &element.children,
            direct_static_children_hoisted,
        )
}

fn structural_children_cross_boundary(
    ns: Namespace,
    children: &Region<'_>,
    direct_static_children_hoisted: bool,
) -> bool {
    children.ops.iter().any(|child| match child {
        Op::Element(element) => child_crosses_direct(ns, element, direct_static_children_hoisted),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| ensure_sufficient_stack(|| children_cross_boundary(ns, &branch.region))),
        Op::For(for_op) => ensure_sufficient_stack(|| children_cross_boundary(ns, &for_op.region)),
        Op::Component(_) | Op::Slot(_) | Op::Text(_) | Op::Interpolation(_) => false,
    })
}

fn child_crosses_direct(
    ns: Namespace,
    element: &ElementOp<'_>,
    direct_static_children_hoisted: bool,
) -> bool {
    if element.namespace != ns {
        return !(direct_static_children_hoisted && super::hoist::is_hoistable(element));
    }
    child_namespace_crosses(element)
}

fn children_cross_boundary(ns: Namespace, children: &Region<'_>) -> bool {
    children.ops.iter().any(|child| child_crosses(ns, child))
}

fn child_crosses(ns: Namespace, child: &Op<'_>) -> bool {
    match child {
        Op::Element(element) => element.namespace != ns || child_namespace_crosses(element),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| ensure_sufficient_stack(|| children_cross_boundary(ns, &branch.region))),
        Op::For(for_op) => ensure_sufficient_stack(|| children_cross_boundary(ns, &for_op.region)),
        Op::Component(_) | Op::Slot(_) | Op::Text(_) | Op::Interpolation(_) => false,
    }
}

fn child_namespace_crosses(element: &ElementOp<'_>) -> bool {
    child_namespace(element) != element.namespace
        && element
            .children
            .ops
            .iter()
            .any(|op| !matches!(op, Op::Text(_) | Op::Interpolation(_)))
}

pub(super) fn with_child<T>(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    f: impl FnOnce(&mut EmitCx<'_>) -> Result<T, EmitError>,
) -> Result<T, EmitError> {
    let saved = cx.parent_ns;
    cx.parent_ns = child_namespace(element);
    let result = f(cx);
    cx.parent_ns = saved;
    result
}
