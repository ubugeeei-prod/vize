//! Site enumeration and exclusion predicates for the metamorphic
//! mutators (P2-15, TS-21).
//!
//! One immutable walk over a parsed S1 tree yields every candidate
//! mutation site together with the verdict of its exclusion predicates.
//! A site a predicate refuses is a **counted skip**, never a silent
//! pass — the TS-11 scope-proof discipline applied to mutation. The
//! predicates are deliberately conservative: a skip costs coverage, an
//! over-broad mutator asserts a wrong expected value (assurance §4),
//! so every "allow" below has a written justification in `mutators.rs`
//! and everything unproven skips.
//!
//! # Scope rule
//!
//! Sites exist only inside an **attribute-less root-level `<template>`
//! element** — the SFC template block, the one region of a `.vue` file
//! where Vue's template-equivalence arguments apply. Script, style and
//! custom blocks are not template markup (their whitespace and text
//! belong to another language), and a root `<template>` carrying any
//! attribute (`lang="pug"`, `src=…`, `functional`) does not hold
//! Vue-HTML template content, so nothing inside those regions is a
//! site. Within scope, three flags narrow individual mutators:
//!
//! - `rawtext` — inside `script`/`style`/`textarea`/`title`/`iframe`/
//!   `noscript`/`xmp`/`listing`/`plaintext`: the content is not
//!   template text, every mutator skips.
//! - `pre` — inside `<pre>`: whitespace is significant
//!   (`crates/vize_armature/src/parser/whitespace.rs:155-161` stops
//!   condensing there), the whitespace mutator skips; byte-preserving
//!   mutators still apply.
//! - `v_pre` — inside a `v-pre` subtree: directives are inert text in
//!   Vue but P2-8's lowering still defers `v-pre` and lowers directives
//!   normally, so a mutation could compare two artifacts that are both
//!   wrong the same way. Every mutator skips.

use vize_s1::{Element, SurfaceChild, SurfaceTree};

use super::predicates::{
    has_attr_named, has_slot_attr, is_branch_name, merge_skip, reorder_skip, wrap_skip,
};

/// Which mutator a site belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Reorder,
    Wrap,
    Split,
    Merge,
    Whitespace,
}

/// One candidate mutation site.
#[derive(Debug, Clone)]
pub struct Site {
    pub kind: Kind,
    /// Child-index path from the S1 root to the carrying node.
    pub path: Vec<usize>,
    pub detail: Detail,
    /// `None` when every predicate passed; otherwise the skip reason.
    pub skip: Option<&'static str>,
}

/// The per-kind payload of a site.
#[derive(Debug, Clone)]
pub enum Detail {
    /// Swap the element's attributes `index` and `index + 1`.
    AttrPair { index: usize },
    /// Move the branch directive at `index` onto a `<template>` wrapper.
    BranchAttr { index: usize },
    /// Split the text token at byte `at` of its `text` slice.
    SplitAt { at: usize },
    /// Merge the text child at the path with its following text sibling.
    MergeNext,
    /// Replace the whitespace run at `start..start + len` of the token
    /// text; `newline` records whether the run contains `\n`/`\r`.
    WsRun {
        start: usize,
        len: usize,
        newline: bool,
    },
}

/// Vue's whitespace alphabet for the condense strategy — exactly
/// `[ \t\n\f\r]` (`crates/vize_armature/src/parser/whitespace.rs:12-16`).
pub fn is_vue_ws(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{000C}' | '\r')
}

/// Elements whose content is raw text, not template markup.
const RAWTEXT_TAGS: [&str; 9] = [
    "script",
    "style",
    "textarea",
    "title",
    "iframe",
    "noscript",
    "xmp",
    "listing",
    "plaintext",
];

#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    pub rawtext: bool,
    pub pre: bool,
    pub v_pre: bool,
}

/// What the wrap mutator needs to know about the parent of a candidate.
#[derive(Debug, Clone, Copy, Default)]
pub struct Parent {
    /// The parent element is a component: its children are slot content.
    pub component: bool,
    /// The parent is a `<template>` carrying `v-slot`/`#…`.
    pub slot_template: bool,
}

/// Enumerate every site in `tree`, in document order.
pub fn enumerate(tree: &SurfaceTree<'_>) -> Vec<Site> {
    let mut sites = Vec::new();
    for (index, child) in tree.children.iter().enumerate() {
        if let SurfaceChild::Element(element) = child
            && element.tag() == "template"
            && element.open.attrs.is_empty()
        {
            let path = vec![index];
            walk_children(
                &element.children,
                &path,
                tree.source,
                Flags::default(),
                Parent::default(),
                &mut sites,
            );
        }
    }
    sites
}

fn walk_children(
    children: &[SurfaceChild<'_>],
    path: &[usize],
    source: &str,
    flags: Flags,
    parent: Parent,
    sites: &mut Vec<Site>,
) {
    for (index, child) in children.iter().enumerate() {
        let mut child_path = path.to_vec();
        child_path.push(index);
        match child {
            SurfaceChild::Element(element) => {
                element_sites(element, &child_path, flags, parent, sites);
                let tag = element.tag();
                let child_flags = Flags {
                    rawtext: flags.rawtext || RAWTEXT_TAGS.contains(&tag),
                    pre: flags.pre || tag == "pre",
                    v_pre: flags.v_pre || has_attr_named(element, "v-pre"),
                };
                let child_parent = Parent {
                    component: !vize_s0::is_native_tag(tag),
                    slot_template: tag == "template" && has_slot_attr(element),
                };
                walk_children(
                    &element.children,
                    &child_path,
                    source,
                    child_flags,
                    child_parent,
                    sites,
                );
            }
            SurfaceChild::Text(token) => {
                text_sites(token.text, &child_path, flags, sites);
                if let Some(SurfaceChild::Text(next)) = children.get(index + 1) {
                    sites.push(Site {
                        kind: Kind::Merge,
                        path: child_path.clone(),
                        detail: Detail::MergeNext,
                        skip: merge_skip(source, token, next, flags),
                    });
                }
            }
            _ => {}
        }
    }
}

fn element_sites(
    element: &Element<'_>,
    path: &[usize],
    flags: Flags,
    parent: Parent,
    sites: &mut Vec<Site>,
) {
    let attrs = &element.open.attrs;
    for index in 0..attrs.len().saturating_sub(1) {
        sites.push(Site {
            kind: Kind::Reorder,
            path: path.to_vec(),
            detail: Detail::AttrPair { index },
            skip: reorder_skip(element, index, flags),
        });
    }
    if let Some(index) = attrs.iter().position(|attr| is_branch_name(attr.name.text)) {
        sites.push(Site {
            kind: Kind::Wrap,
            path: path.to_vec(),
            detail: Detail::BranchAttr { index },
            skip: wrap_skip(element, index, flags, parent),
        });
    }
}

fn text_sites(text: &str, path: &[usize], flags: Flags, sites: &mut Vec<Site>) {
    sites.push(Site {
        kind: Kind::Split,
        path: path.to_vec(),
        detail: Detail::SplitAt {
            at: split_point(text).unwrap_or(0),
        },
        skip: if flags.rawtext {
            Some("rawtext-content")
        } else if flags.v_pre {
            Some("v-pre-subtree")
        } else if split_point(text).is_none() {
            Some("no-split-point")
        } else {
            None
        },
    });
    let mut start = 0usize;
    for (offset, c) in text.char_indices() {
        if is_vue_ws(c) {
            continue;
        }
        if offset > start {
            push_ws_site(text, start, offset, path, flags, sites);
        }
        start = offset + c.len_utf8();
    }
    if text.len() > start {
        push_ws_site(text, start, text.len(), path, flags, sites);
    }
}

fn push_ws_site(
    text: &str,
    start: usize,
    end: usize,
    path: &[usize],
    flags: Flags,
    sites: &mut Vec<Site>,
) {
    let run = &text[start..end];
    sites.push(Site {
        kind: Kind::Whitespace,
        path: path.to_vec(),
        detail: Detail::WsRun {
            start,
            len: end - start,
            newline: run.contains('\n') || run.contains('\r'),
        },
        skip: if flags.rawtext {
            Some("rawtext-content")
        } else if flags.pre {
            Some("pre-content")
        } else if flags.v_pre {
            Some("v-pre-subtree")
        } else {
            None
        },
    });
}

/// The midpoint split position of a text slice, snapped to a char
/// boundary strictly inside the slice; `None` when no such position
/// exists (fewer than two chars).
pub fn split_point(text: &str) -> Option<usize> {
    let mut at = text.len() / 2;
    while at > 0 && !text.is_char_boundary(at) {
        at -= 1;
    }
    if at == 0 {
        at = text.len() / 2;
        while at < text.len() && !text.is_char_boundary(at) {
            at += 1;
        }
    }
    (at > 0 && at < text.len()).then_some(at)
}
