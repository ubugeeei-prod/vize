//! The ancestor chain — the checker's model of the parser's stack of open
//! elements — and the stack scans §13.2 defines over it.
//!
//! A chain is a known suffix of the stack (innermost last) above a [`Base`]:
//! either the bottom of a document (the scan has seen every open element) or
//! an unknown context (a template root, a slot's content). Every scan is
//! three-valued: it answers `Yes`/`No` when the known frames decide it and
//! `Maybe` when the answer depends on frames it cannot see — which is what
//! makes a per-file verdict stay true when the context is later composed in.

use super::facts::{Attr, Cond, ElemId, Ns, Row, eval_attr_cond, facts};
use super::skeleton::AttrFacts;
use super::tri::Tri;

/// A set of possible namespaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NsSet(u8);

impl NsSet {
    /// The empty set.
    pub const EMPTY: Self = Self(0);
    /// Every namespace.
    pub const ALL: Self = Self(0b111);

    /// The singleton `{ns}`.
    pub const fn one(ns: Ns) -> Self {
        Self(1 << ns as u8)
    }

    /// Set union.
    #[must_use]
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether `ns` is possible.
    pub const fn has(self, ns: Ns) -> bool {
        self.0 & (1 << ns as u8) != 0
    }

    /// Whether the set is exactly `{ns}`.
    pub const fn is(self, ns: Ns) -> bool {
        self.0 == 1 << ns as u8
    }

    /// The possible namespaces.
    pub fn iter(self) -> impl Iterator<Item = Ns> {
        [Ns::Html, Ns::Svg, Ns::MathMl]
            .into_iter()
            .filter(move |ns| self.has(*ns))
    }
}

/// How the tree-construction dispatcher treats a start tag whose adjusted
/// current node is a given element (§13.2.6, "tree construction dispatcher").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dispatch {
    /// An HTML element: the insertion-mode rules apply.
    Html,
    /// An HTML integration point: start tags use the insertion-mode rules.
    HtmlIntegration,
    /// A MathML text integration point: start tags other than `mglyph` and
    /// `malignmark` use the insertion-mode rules.
    MathText,
    /// A MathML `annotation-xml` that is not an HTML integration point: `svg`
    /// uses the insertion-mode rules, everything else foreign content.
    AnnotationXml,
    /// Any other foreign element: the rules for foreign content.
    Foreign(Ns),
}

/// A known element on the chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    /// Fact-table ids of the element's tag per namespace.
    pub ids: [Option<ElemId>; 3],
    /// The namespaces the element may have been inserted in.
    pub ns: NsSet,
    /// Attribute facts.
    pub attrs: AttrFacts,
    /// Where the element came from: `(skeleton, node)`, `None` for frames a
    /// context supplies.
    pub origin: Option<(u32, u32)>,
}

impl Frame {
    /// The fact-table id of this element in `ns`.
    #[inline]
    pub fn id(&self, ns: Ns) -> Option<ElemId> {
        self.ids[ns as usize]
    }

    /// Whether this frame is certainly the HTML element `tag`.
    pub fn is_html(&self, tag: &str) -> Tri {
        let matches = self
            .id(Ns::Html)
            .is_some_and(|id| facts().name(id).1 == tag);
        if !matches || !self.ns.has(Ns::Html) {
            Tri::No
        } else if self.ns.is(Ns::Html) {
            Tri::Yes
        } else {
            Tri::Maybe
        }
    }

    /// Three-valued membership of this element in a table row.
    pub fn member(&self, row: Row) -> Tri {
        let mut result: Option<Tri> = None;
        for ns in self.ns.iter() {
            let tri = member_of(row, self.id(ns), &self.attrs, |_| Tri::Maybe);
            result = Some(result.map_or(tri, |acc| acc.join(tri)));
        }
        result.unwrap_or(Tri::No)
    }

    /// The dispatch kinds this element may produce as the adjusted current
    /// node, each with the namespace that produces it.
    pub fn dispatches(&self) -> impl Iterator<Item = (Dispatch, Ns)> + '_ {
        let mut out: [Option<(Dispatch, Ns)>; 4] = [None; 4];
        if self.ns == NsSet::one(Ns::Html) {
            // The common case: an HTML element has the one HTML dispatch.
            out[0] = Some((Dispatch::Html, Ns::Html));
            return out.into_iter().flatten();
        }
        let mut len = 0;
        let mut push = |dispatch, ns| {
            out[len] = Some((dispatch, ns));
            len += 1;
        };
        for ns in self.ns.iter() {
            let id = self.id(ns);
            match ns {
                Ns::Html => push(Dispatch::Html, ns),
                Ns::Svg if facts().is(Row::HtmlIntegration, id) => {
                    push(Dispatch::HtmlIntegration, ns);
                }
                Ns::Svg => push(Dispatch::Foreign(Ns::Svg), ns),
                Ns::MathMl if facts().is(Row::MathmlTextIntegration, id) => {
                    push(Dispatch::MathText, ns);
                }
                Ns::MathMl if id.is_some_and(|id| facts().name(id).1 == "annotation-xml") => {
                    match self.attrs.get(Attr::EncodingHtml) {
                        Tri::Yes => push(Dispatch::HtmlIntegration, ns),
                        Tri::No => push(Dispatch::AnnotationXml, ns),
                        Tri::Maybe => {
                            push(Dispatch::HtmlIntegration, ns);
                            push(Dispatch::AnnotationXml, ns);
                        }
                    }
                }
                Ns::MathMl => push(Dispatch::Foreign(Ns::MathMl), ns),
            }
        }
        out.into_iter().flatten()
    }
}

/// Three-valued membership of an element in a table row; `structural`
/// answers the conditions that depend on the tree rather than attributes.
pub fn member_of(
    row: Row,
    id: Option<ElemId>,
    attrs: &AttrFacts,
    structural: impl Fn(Cond) -> Tri,
) -> Tri {
    let Some(id) = id else { return Tri::No };
    match facts().row(row).condition(id) {
        None => Tri::No,
        Some(None) => Tri::Yes,
        Some(Some(cond)) => {
            eval_attr_cond(cond, |attr| attrs.get(attr)).unwrap_or_else(|| structural(cond))
        }
    }
}

/// What lies below the known frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    /// The bottom of the document: nothing else is open.
    Document,
    /// An unknown context: body content outside native `template` contents,
    /// whose open elements the checker cannot see.
    Truncated,
}

/// The ancestor chain of the node being checked.
#[derive(Debug, Clone)]
pub struct Chain {
    /// Known frames, outermost first.
    pub frames: Vec<Frame>,
    /// What lies below `frames[0]`.
    pub base: Base,
}

/// A scan's verdict and, when `Yes`, the index of the deciding frame.
pub type Scan = (Tri, Option<usize>);

/// How a frame participates in a scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Target,
    Stop,
    Pass,
}

impl Chain {
    /// An unknown context with no known frames.
    pub fn truncated() -> Self {
        Self {
            frames: Vec::with_capacity(32),
            base: Base::Truncated,
        }
    }

    /// The current node, if known.
    pub fn top(&self) -> Option<&Frame> {
        self.frames.last()
    }

    fn exhausted(&self) -> Scan {
        match self.base {
            Base::Document => (Tri::No, None),
            Base::Truncated => (Tri::Maybe, None),
        }
    }

    /// Walk up from the current node; `role` classifies a frame under one
    /// namespace. A frame whose role differs across its possible namespaces
    /// makes the scan unknown.
    fn walk(&self, role: impl Fn(&Frame, Ns) -> Role) -> Scan {
        for (index, frame) in self.frames.iter().enumerate().rev() {
            let mut roles = frame.ns.iter().map(|ns| role(frame, ns));
            let first = roles.next().unwrap_or(Role::Pass);
            if roles.any(|other| other != first) {
                return (Tri::Maybe, None);
            }
            match first {
                Role::Target => return (Tri::Yes, Some(index)),
                Role::Stop => return (Tri::No, None),
                Role::Pass => {}
            }
        }
        self.exhausted()
    }

    /// "has an HTML `target` element in the specific scope" whose boundary
    /// list is `scope` (§13.2.4.2).
    pub fn in_scope(&self, target: &str, scope: Row) -> Scan {
        self.walk(|frame, ns| {
            let id = frame.id(ns);
            if ns == Ns::Html && id.is_some_and(|id| facts().name(id).1 == target) {
                Role::Target
            } else if facts().is(scope, id) {
                Role::Stop
            } else {
                Role::Pass
            }
        })
    }

    /// The `li` / `dd`+`dt` loop of the "in body" start-tag rules: find an
    /// open `targets` element before a special element other than `address`,
    /// `div` and `p`.
    pub fn list_item_loop(&self, targets: &[&str]) -> Scan {
        self.walk(|frame, ns| {
            let id = frame.id(ns);
            let name = id.map(|id| facts().name(id).1);
            if ns == Ns::Html && name.is_some_and(|name| targets.contains(&name)) {
                Role::Target
            } else if facts().is(Row::Special, id)
                && !(ns == Ns::Html && facts().is(Row::ListItemLoopTransparent, id))
            {
                Role::Stop
            } else {
                Role::Pass
            }
        })
    }

    /// Whether an HTML `a` sits in the list of active formatting elements
    /// after the last marker (§13.2.4.3): an open `a` with no marker-inserting
    /// element above it.
    pub fn anchor_after_marker(&self) -> Scan {
        self.walk(|frame, ns| {
            let id = frame.id(ns);
            if ns == Ns::Html && id.is_some_and(|id| facts().name(id).1 == "a") {
                Role::Target
            } else if ns == Ns::Html && facts().is(Row::Marker, id) {
                Role::Stop
            } else {
                Role::Pass
            }
        })
    }

    /// Whether any open element is the HTML `tag` (no boundary).
    pub fn has_open(&self, tag: &str) -> Scan {
        self.walk(|frame, ns| {
            let id = frame.id(ns);
            if ns == Ns::Html && id.is_some_and(|id| facts().name(id).1 == tag) {
                Role::Target
            } else {
                Role::Pass
            }
        })
    }
}
