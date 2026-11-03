//! L1 child lists read the way the lowering reads them: one condense plan
//! per list, text runs through the lowering's text lane, `v-if` chains
//! through its grouping, each element's own children under the lane and
//! `v-pre` state the lowering gives them.
//!
//! The context travels down the walk with each element, so no element is
//! located again from the root.

use super::L1Markup;
use crate::ir::ByteRange;
use crate::markup::element::{MarkupElement, MarkupElementInner};
use crate::markup::l2::surface::{offset_in, token_range};
use crate::markup::node::{MarkupNode, MarkupText};
use vize_l0::Span;
use vize_l1::markup::{Attribute, Element, SurfaceChild};
use vize_l1_to_l2::lower::{
    RunShape, TextLane, branch_gap_text_at, children_ns, enter_ns, if_chain, structure, v_for_value,
};
use vize_l2::op::Namespace;

/// The reading one child list is under.
#[derive(Clone, Copy)]
pub(in crate::markup) struct ListCx {
    pub(in crate::markup) lane: TextLane,
    /// Inside a `v-pre` subtree.
    pub(in crate::markup) frozen: bool,
    pub(in crate::markup) ns: Namespace,
}

/// One child of an L1 list.
pub(in crate::markup) enum Item<'a> {
    /// An element outside a chain, with the `v-for` it builds a loop from.
    Element(MarkupElement<'a>, Option<&'a Attribute<'a>>),
    Chain(L1Chain<'a>),
    Text(MarkupText<'a>),
    Interpolation(ByteRange),
    Comment(ByteRange),
}

/// A `v-if` chain as the lowering groups it: siblings `start..end`, whose
/// elements are its branches and whose other children are consumed gaps.
#[derive(Clone, Copy)]
pub(in crate::markup) struct L1Chain<'a> {
    doc: &'a L1Markup<'a>,
    list: ListCx,
    siblings: &'a [SurfaceChild<'a>],
    start: usize,
    end: usize,
    first: &'a Element<'a>,
    last: &'a Element<'a>,
    branches: usize,
    unconditional: bool,
}

/// An element of a list and the `v-for` it builds a loop from.
pub(in crate::markup) fn element<'a>(
    doc: &'a L1Markup<'a>,
    list: ListCx,
    element: &'a Element<'a>,
    in_chain: bool,
) -> (MarkupElement<'a>, Option<&'a Attribute<'a>>) {
    let structure = structure(element, list.frozen);
    let tag = element.tag();
    let ns = enter_ns(list.ns, tag);
    let frozen = list.frozen || structure.opens_v_pre;
    // `<slot>` and an unwrapped `<template>` lower their children into the
    // enclosing list's reading; any other element owns its children's.
    let (lane, children_frozen) = if tag == "slot" || structure.unwraps(element, in_chain) {
        (list.lane, list.frozen)
    } else {
        let html = ns == Namespace::Html;
        let lane = list
            .lane
            .children_of(doc.text, element, tag, html, structure.opens_v_pre);
        (lane, frozen)
    };
    let vfor = structure
        .vfor
        .and_then(|index| element.open.attrs.get(index))
        .filter(|attr| v_for_value(attr).is_some());
    let wrapped = MarkupElement::from_inner(MarkupElementInner::Surface {
        element,
        doc,
        frozen,
        opens_v_pre: structure.opens_v_pre,
        ns,
        lane,
        children_frozen,
    });
    (wrapped, vfor)
}

/// The reading a Surface element's own children are under.
pub(in crate::markup) fn children_list(
    element: &Element<'_>,
    ns: Namespace,
    lane: TextLane,
    children_frozen: bool,
) -> ListCx {
    ListCx {
        lane,
        frozen: children_frozen,
        ns: children_ns(ns, element.tag()),
    }
}

/// Visit one list's children in authored order.
pub(in crate::markup) fn walk<'a>(
    doc: &'a L1Markup<'a>,
    list: ListCx,
    children: &'a [SurfaceChild<'a>],
    visit: &mut dyn FnMut(Item<'a>),
) {
    let plan = list.lane.plan(doc.text, children);
    let mut index = 0;
    while let Some(child) = children.get(index) {
        match child {
            SurfaceChild::Element(child) => {
                if let Some(chain) = if_chain(children, index, list.frozen) {
                    let last = chain
                        .branches
                        .last()
                        .and_then(|(at, _)| match children.get(*at) {
                            Some(SurfaceChild::Element(last)) => Some(&**last),
                            _ => None,
                        })
                        .unwrap_or(&**child);
                    visit(Item::Chain(L1Chain {
                        doc,
                        list,
                        siblings: children,
                        start: index,
                        end: chain.end,
                        first: child,
                        last,
                        branches: chain.branches.len(),
                        unconditional: chain.unconditional,
                    }));
                    index = chain.end;
                    continue;
                }
                let (element, vfor) = element(doc, list, child, false);
                visit(Item::Element(element, vfor));
            }
            SurfaceChild::Text(_) | SurfaceChild::Interpolation(_) => {
                let run = list.lane.run(doc.text, children, &plan, index);
                visit_run(doc, run.shape, visit);
                index = run.end;
                continue;
            }
            SurfaceChild::Comment(token) => visit(Item::Comment(token_range(doc.source, token))),
            SurfaceChild::Cdata(token) => {
                let inner = vize_l1_to_l2::lower::cdata_text(token.text);
                let start = offset_in(doc.source, inner);
                visit(Item::Text(MarkupText::from_static(
                    doc.decode_text(inner),
                    ByteRange::new(start, start + inner.len() as u32),
                )));
            }
            // Stray tags and processing instructions lower to nothing.
            SurfaceChild::ProcessingInstruction(_) | SurfaceChild::Unexpected(_) => {}
        }
        index += 1;
    }
}

fn visit_run<'a>(doc: &'a L1Markup<'a>, shape: RunShape<'a>, visit: &mut dyn FnMut(Item<'a>)) {
    match shape {
        RunShape::Dropped => {}
        RunShape::Text { content, span } => {
            visit(Item::Text(MarkupText::from_static(
                doc.decode_text(content),
                range(span),
            )));
        }
        RunShape::Interpolation { span } => visit(Item::Interpolation(range(span))),
        RunShape::Merged { parts, .. } => {
            for part in &parts {
                if part.dynamic {
                    visit(Item::Interpolation(range(part.span)));
                } else {
                    let text = owned_text(doc, part.text.as_str(), part.span);
                    visit(Item::Text(MarkupText::from_static(text, range(part.span))));
                }
            }
        }
        RunShape::Frozen { content, span } => {
            let text = owned_text(doc, content.as_str(), span);
            visit(Item::Text(MarkupText::from_static(text, range(span))));
        }
    }
}

/// A lowered static text, decoded: its source bytes when unchanged, else
/// an arena copy.
fn owned_text<'a>(doc: &'a L1Markup<'a>, text: &str, span: Span) -> &'a str {
    let text = match doc.source.get(span.start as usize..span.end as usize) {
        Some(authored) if authored == text => authored,
        _ => doc.allocator.alloc_str(text),
    };
    doc.decode_text(text)
}

#[inline]
fn range(span: Span) -> ByteRange {
    ByteRange::new(span.start, span.end)
}

impl<'a> Item<'a> {
    /// The child as authored: a chain contributes its branches and gaps.
    pub(in crate::markup) fn walk_nodes(self, visitor: &mut dyn FnMut(MarkupNode<'a>)) {
        match self {
            Item::Element(element, _) => visitor(MarkupNode::Element(element)),
            Item::Chain(chain) => chain.walk_nodes(visitor),
            Item::Text(text) => visitor(MarkupNode::Text(text)),
            Item::Interpolation(at) => visitor(MarkupNode::Interpolation(at)),
            Item::Comment(at) => visitor(MarkupNode::Comment(at)),
        }
    }
}

impl<'a> L1Chain<'a> {
    pub(in crate::markup) const fn branch_count(&self) -> usize {
        self.branches
    }

    pub(in crate::markup) const fn has_else(&self) -> bool {
        self.unconditional
    }

    fn members(&self) -> impl Iterator<Item = (usize, &'a SurfaceChild<'a>)> + 'a {
        let siblings = self.siblings;
        (self.start..self.end).filter_map(move |index| Some((index, siblings.get(index)?)))
    }

    /// From the first branch's tag to the end of the last branch's opening tag.
    pub(in crate::markup) fn range(&self) -> ByteRange {
        ByteRange::new(
            offset_in(self.doc.source, self.first.open.lt_name.text),
            token_range(self.doc.source, &self.last.open.gt).end,
        )
    }

    /// Each branch element, with the `v-for` it builds a loop from.
    pub(in crate::markup) fn walk_branches(
        &self,
        visit: &mut dyn FnMut(MarkupElement<'a>, Option<&'a Attribute<'a>>),
    ) {
        for (_, child) in self.members() {
            if let SurfaceChild::Element(branch) = child {
                let (branch, vfor) = element(self.doc, self.list, branch, true);
                visit(branch, vfor);
            }
        }
    }

    /// Branches interleaved with the gap comments and kept gap whitespace,
    /// read with the parse-time gap view.
    fn walk_nodes(&self, visitor: &mut dyn FnMut(MarkupNode<'a>)) {
        for (index, child) in self.members() {
            match child {
                SurfaceChild::Element(branch) => {
                    visitor(MarkupNode::Element(
                        element(self.doc, self.list, branch, true).0,
                    ));
                }
                SurfaceChild::Comment(token) => {
                    visitor(MarkupNode::Comment(token_range(self.doc.source, token)));
                }
                SurfaceChild::Text(token) => {
                    if let Some(text) = self.gap_text(index) {
                        visitor(MarkupNode::Text(MarkupText::from_static(
                            text,
                            token_range(self.doc.source, token),
                        )));
                    }
                }
                _ => {}
            }
        }
    }

    /// The kept gap whitespace, which follows the branch visits.
    pub(in crate::markup) fn walk_kept_gaps(&self, visit: &mut dyn FnMut(MarkupText<'a>)) {
        for (index, child) in self.members() {
            if let SurfaceChild::Text(token) = child
                && let Some(text) = self.gap_text(index)
            {
                visit(MarkupText::from_static(
                    text,
                    token_range(self.doc.source, token),
                ));
            }
        }
    }

    fn gap_text(&self, index: usize) -> Option<&'a str> {
        branch_gap_text_at(
            self.doc.allocator,
            self.siblings,
            index,
            self.list.lane.in_pre(),
        )
    }
}

/// Visit each element of one list in authored order, chain branches included.
fn walk_elements<'a>(
    doc: &'a L1Markup<'a>,
    list: ListCx,
    children: &'a [SurfaceChild<'a>],
    visit: &mut dyn FnMut(MarkupElement<'a>),
) {
    let mut index = 0;
    while let Some(child) = children.get(index) {
        if let SurfaceChild::Element(child) = child {
            if let Some(chain) = if_chain(children, index, list.frozen) {
                for branch in children.get(index..chain.end).unwrap_or_default() {
                    if let SurfaceChild::Element(branch) = branch {
                        visit(element(doc, list, branch, true).0);
                    }
                }
                index = chain.end;
                continue;
            }
            visit(element(doc, list, child, false).0);
        }
        index += 1;
    }
}

/// Walk the element tree of one list, calling `enter` / `exit`.
pub(in crate::markup) fn walk_tree<'a>(
    doc: &'a L1Markup<'a>,
    list: ListCx,
    children: &'a [SurfaceChild<'a>],
    enter: &mut dyn FnMut(MarkupElement<'a>),
    exit: &mut dyn FnMut(MarkupElement<'a>),
) {
    walk_elements(doc, list, children, &mut |entered| {
        enter(entered);
        if let MarkupElementInner::Surface {
            element,
            ns,
            lane,
            children_frozen,
            ..
        } = entered.inner
        {
            let list = children_list(element, ns, lane, children_frozen);
            vize_l0::ensure_sufficient_stack(|| {
                walk_tree(doc, list, &element.children, enter, exit);
            });
        }
        exit(entered);
    });
}
