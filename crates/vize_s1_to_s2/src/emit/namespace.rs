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
            cx.source,
            element.namespace,
            &element.children,
            direct_static_children_hoisted,
            cx.is_ts,
        )
}

fn structural_children_cross_boundary(
    source: &str,
    ns: Namespace,
    children: &Region<'_>,
    direct_static_children_hoisted: bool,
    is_ts: bool,
) -> bool {
    children.ops.iter().any(|child| match child {
        Op::Element(element) => {
            child_crosses_direct(is_ts, ns, element, direct_static_children_hoisted)
        }
        Op::If(if_op) => if_op.branches.iter().any(|branch| {
            ensure_sufficient_stack(|| {
                !authored_template_branch(source, branch)
                    && children_cross_boundary(ns, &branch.region)
            })
        }),
        Op::For(for_op) => ensure_sufficient_stack(|| children_cross_boundary(ns, &for_op.region)),
        Op::Component(_) | Op::Slot(_) | Op::Text(_) | Op::Interpolation(_) => false,
    })
}

fn child_crosses_direct(
    is_ts: bool,
    ns: Namespace,
    element: &ElementOp<'_>,
    direct_static_children_hoisted: bool,
) -> bool {
    if element.namespace != ns {
        return !(direct_static_children_hoisted && super::hoist::is_hoistable(element, is_ts));
    }
    false
}

fn children_cross_boundary(ns: Namespace, children: &Region<'_>) -> bool {
    children.ops.iter().any(|child| child_crosses(ns, child))
}

fn child_crosses(ns: Namespace, child: &Op<'_>) -> bool {
    match child {
        Op::Element(element) => element.namespace != ns,
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
            .any(|op| matches!(op, Op::Element(_) | Op::If(_) | Op::For(_)))
}

fn authored_template_branch(source: &str, branch: &vize_s2::op::IfBranch<'_>) -> bool {
    let Ok(start) = usize::try_from(branch.span.start) else {
        return false;
    };
    let Ok(end) = usize::try_from(branch.span.end) else {
        return false;
    };
    source
        .get(start..end)
        .is_some_and(|source| source.trim_start().starts_with("<template"))
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
