//! Region ownership becomes the native tree: element children, branch roots,
//! and loop bodies. Every owner is accounted for exactly once.

use vize_carton::{Allocator, Vec};
use vize_s3::op::Program;

use super::super::{Content, Node};
use super::{Result, Slots, at, at_mut};
use crate::s3::{AdmissionFailure, LegacyReason};

/// Returns each node's owning node. Branch roots and loop bodies are owned by
/// their control node, whose body starts a separate template. Owned regions
/// move their members into the owner; the unowned root region stays.
pub(super) fn assemble<'a>(
    program: &Program<'_>,
    nodes: &mut [Node<'a>],
    slots: &Slots,
    regions: &mut [Vec<'a, usize>],
    alloc: &'a Allocator,
) -> Result<Vec<'a, Option<usize>>> {
    let mut owned = super::filled(alloc, false, nodes.len());
    let mut parents = super::filled(alloc, None, nodes.len());
    for region in &program.regions {
        let Some(owner) = region.owner else { continue };
        let Some(&Some((index, _))) = slots.get(owner.index() as usize) else {
            return Err(LegacyReason::Structure.into());
        };
        let children = regions.get_mut(region.id.index() as usize).map_or_else(
            || Vec::new_in(&alloc),
            |members| std::mem::replace(members, Vec::new_in(&alloc)),
        );
        if children.iter().any(|child| *child <= index) {
            return Err(LegacyReason::Structure.into());
        }
        for child in &children {
            *at_mut(&mut parents, *child)? = Some(index);
        }
        // A body is its carrier element, or the non-empty fragment a
        // `<template>` carrier unwrapped (text and nested control flow too).
        let element = matches!(
            children.as_slice(),
            [child] if matches!(nodes.get(*child), Some(Node { content: Content::Element { .. }, .. }))
        );
        let owner_owned = at_mut(&mut owned, index)?;
        let node = at_mut(nodes, index)?;
        match &mut node.content {
            Content::Element { tag, .. } => {
                if std::mem::replace(owner_owned, true)
                    || super::ident::admitted_void(tag) && !children.is_empty()
                {
                    return Err(LegacyReason::Structure.into());
                }
            }
            Content::If { branches } => {
                let branch = branches
                    .iter_mut()
                    .find(|branch| branch.region == region.id)
                    .ok_or(AdmissionFailure::Invalid(
                        "branch region lacks its condition",
                    ))?;
                if children.is_empty() {
                    return Err(LegacyReason::ControlFlow.into());
                }
                branch.roots = children;
                continue;
            }
            Content::For(body) => {
                if std::mem::replace(owner_owned, true) {
                    return Err(AdmissionFailure::Invalid("loop owns several bodies"));
                }
                if !(element || body.template && !children.is_empty()) {
                    return Err(LegacyReason::ControlFlow.into());
                }
            }
            // Slot content and fallbacks are fragments rendered by their own block.
            Content::Component { .. } | Content::Outlet { .. } => {
                if std::mem::replace(owner_owned, true) {
                    return Err(AdmissionFailure::Invalid("slot owner has several regions"));
                }
            }
            Content::Text { .. } => return Err(LegacyReason::Structure.into()),
        }
        node.children = children;
    }
    for (node, &owned) in nodes.iter().zip(owned.iter()) {
        match &node.content {
            Content::Element { .. } | Content::Component { .. } | Content::Outlet { .. }
                if !owned =>
            {
                return Err(LegacyReason::Structure.into());
            }
            Content::For(_) if !owned => {
                return Err(AdmissionFailure::Invalid("loop lacks its body"));
            }
            Content::If { branches } if branches.iter().any(|branch| branch.roots.is_empty()) => {
                return Err(AdmissionFailure::Invalid("branch lacks its region"));
            }
            _ => {}
        }
    }
    Ok(parents)
}

/// Authored element depth the native lane admits, far below the legacy
/// parser's 4096-element flattening limit (which it reports).
const MAX_DEPTH: u32 = 1024;

/// Open elements whose authored descendants HTML tree construction repairs.
const BUTTON: u8 = 1;
const ITEM: u8 = 2;
const PARAGRAPH: u8 = 4;
const ANCHOR: u8 = 8;
const FORM: u8 = 16;

/// Start tags that close an open `<p>` in button scope.
fn closes_paragraph(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "div"
            | "dl"
            | "fieldset"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "ul"
    )
}

/// HTML tree construction closes an open button, anchor or form-nested form
/// instead of nesting a new one, closes a list item before another in list
/// item scope, and closes a paragraph before a block in button scope. The
/// legacy parser reads authored markup, so the guard follows authored
/// nesting, not template strings; components and outlets bound the scoped
/// cases, and branches and loops are transparent. Parents precede children.
pub(super) fn check_nesting(
    nodes: &[Node<'_>],
    parents: &[Option<usize>],
    alloc: &Allocator,
) -> Result<()> {
    let mut open = super::filled(alloc, (0_u8, 0_u32), nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        let parent = *at(parents, index)?;
        let (mut flags, depth) = match parent {
            Some(parent) => *at(&open, parent)?,
            None => (0, 0),
        };
        if let Some(parent) = parent {
            match at(nodes, parent)?.content {
                Content::Element {
                    tag: "ul" | "ol", ..
                } => flags &= !ITEM,
                Content::Element { tag: "button", .. } => flags &= !PARAGRAPH,
                Content::Component { .. } | Content::Outlet { .. } => {
                    flags &= !(ITEM | PARAGRAPH);
                }
                _ => {}
            }
        }
        let tag = match node.content {
            Content::Element { tag, .. } => tag,
            _ => "",
        };
        let own = match tag {
            "button" => BUTTON,
            "li" => ITEM,
            "p" => PARAGRAPH,
            "a" => ANCHOR,
            "form" => FORM,
            _ => 0,
        };
        // Every non-text node counts: a `<template>` carrier is an element
        // for the legacy parser, and double-counting an element carrier only
        // lowers the bound.
        let depth = depth + u32::from(!matches!(node.content, Content::Text { .. }));
        if flags & own & (BUTTON | ITEM | ANCHOR | FORM) != 0
            || flags & PARAGRAPH != 0 && closes_paragraph(tag)
            || depth > MAX_DEPTH
        {
            return Err(LegacyReason::Structure.into());
        }
        *at_mut(&mut open, index)? = (flags | own, depth);
    }
    Ok(())
}
