//! The checker: one depth-first pass over a skeleton, evaluating every node
//! against its ancestor chain.
//!
//! Totality: the pass visits each node once, every rule is a bounded scan of
//! the chain, and every verdict is one of three values — no input makes the
//! checker fail, loop or refuse.

use super::chain::{Base, Chain, Dispatch, Frame, NsSet};
use super::class::{Family, ViolationClass};
use super::content_rules;
use super::facts::{Ns, facts};
use super::parser_rules::{Outcome, Subject};
use super::parser_verdict::{ParserVerdict, evaluate_element};
use super::skeleton::{NodeKind, Skeleton, component_usage_name};
use super::table_rules::{html_text, inert_whitespace};
use super::tri::Tri;

/// The context a skeleton is checked in.
#[derive(Debug, Clone)]
pub enum Context {
    /// An unknown mount point (a component's own template).
    Truncated,
    /// A no-quirks document's `<body>`: `[html, body]` open, nothing else.
    Body,
    /// An explicit chain (a composed parent context).
    Chain(Chain),
}

impl Context {
    fn chain(&self) -> Chain {
        match self {
            Self::Truncated => Chain::truncated(),
            Self::Body => {
                let frame = |tag: &str| Frame {
                    ids: [facts().id(Ns::Html, tag), None, None],
                    ns: NsSet::one(Ns::Html),
                    attrs: Default::default(),
                    origin: None,
                };
                Chain {
                    frames: vec![frame("html"), frame("body")],
                    base: Base::Document,
                }
            }
            Self::Chain(chain) => chain.clone(),
        }
    }
}

/// The verdict for one node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Proven violation; `evidence` is the deciding ancestor, as a chain frame
    /// origin (`None` when the node itself or a context frame decides).
    Proven {
        /// The violation class.
        class: ViolationClass,
        /// The deciding ancestor's `(skeleton, node)` origin.
        evidence: Option<(u32, u32)>,
    },
    /// Proven conforming.
    Refuted,
    /// The parser family depends on something the checker cannot see.
    Unknown,
    /// Proven parser-stable; the content-model family depends on something
    /// the checker cannot see.
    ContentUnknown,
    /// Not evaluated: below a proven parser divergence, inside a subtree whose
    /// content is dynamic, or not an element/text node.
    Skipped,
}

/// Per-node verdicts, indexed like the skeleton's nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// One verdict per skeleton node.
    pub verdicts: Vec<Verdict>,
    /// The parser family alone, per node: `Some(true)` proven divergence,
    /// `Some(false)` proven faithful insertion, `None` unknown or not
    /// evaluated. A content-model verdict can be proven where this is
    /// unknown: the node violates a content model in every context where the
    /// parser inserts it faithfully, and diverges in every other.
    pub parser: Vec<Option<bool>>,
}

impl Report {
    /// Proven violations as `(node, class, evidence)`, in node order.
    pub fn findings(&self) -> impl Iterator<Item = (u32, ViolationClass, Option<(u32, u32)>)> + '_ {
        self.verdicts
            .iter()
            .enumerate()
            .filter_map(|(index, verdict)| match verdict {
                Verdict::Proven { class, evidence } => Some((index as u32, *class, *evidence)),
                _ => None,
            })
    }
}

/// Check a skeleton in a context. `skeleton_id` tags frame origins.
pub fn check(skeleton: &Skeleton, skeleton_id: u32, context: &Context) -> Report {
    check_with(skeleton, skeleton_id, context, false, &mut |_, _| {})
}

/// [`check`] with a hook that sees every component usage with the chain it
/// is rendered in. `composed` marks a check in a parent's context: slot
/// fallback content is then skipped, since whether it renders depends on
/// what the usage passes.
pub fn check_with(
    skeleton: &Skeleton,
    skeleton_id: u32,
    context: &Context,
    composed: bool,
    on_component: &mut dyn FnMut(u32, &Chain),
) -> Report {
    check_pruned(skeleton, skeleton_id, context, composed, &[], on_component)
}

/// [`check_with`] with the subtrees rooted at `pruned` (ascending node
/// indices) left out: nodes the usage proves are not rendered. Their
/// verdicts stay [`Verdict::Skipped`].
pub fn check_pruned(
    skeleton: &Skeleton,
    skeleton_id: u32,
    context: &Context,
    composed: bool,
    pruned: &[u32],
    on_component: &mut dyn FnMut(u32, &Chain),
) -> Report {
    let mut walker = Walker {
        skeleton,
        skeleton_id,
        verdicts: vec![Verdict::Skipped; skeleton.nodes.len()],
        parser: vec![None; skeleton.nodes.len()],
        composed,
        pruned,
        on_component,
    };
    let mut chain = context.chain();
    for root in skeleton.roots() {
        walker.visit(root, &mut chain);
    }
    Report {
        verdicts: walker.verdicts,
        parser: walker.parser,
    }
}

struct Walker<'s, 'h> {
    skeleton: &'s Skeleton,
    skeleton_id: u32,
    verdicts: Vec<Verdict>,
    parser: Vec<Option<bool>>,
    composed: bool,
    pruned: &'s [u32],
    on_component: &'h mut dyn FnMut(u32, &Chain),
}

impl Walker<'_, '_> {
    fn origin(&self, chain: &Chain, frame: Option<usize>) -> Option<(u32, u32)> {
        chain.frames.get(frame?)?.origin
    }

    fn set_parser(&mut self, index: u32, value: Option<bool>) {
        if let Some(slot) = self.parser.get_mut(index as usize) {
            *slot = value;
        }
    }

    fn set_verdict(&mut self, index: u32, verdict: Verdict) {
        if let Some(slot) = self.verdicts.get_mut(index as usize) {
            *slot = verdict;
        }
    }

    fn visit(&mut self, index: u32, chain: &mut Chain) {
        if self.pruned.binary_search(&index).is_ok() {
            return;
        }
        let node = self.skeleton.node(index);
        match &node.kind {
            NodeKind::Element(_) => {
                // Before the element is pushed: a resolved `<my-card />` is
                // replaced by its root, so the root is checked in this chain,
                // not inside a `my-card` frame. Unresolved tags stay elements.
                if component_usage_name(self.skeleton.node(index)).is_some() {
                    (self.on_component)(index, chain);
                }
                let node = self.skeleton.node(index);
                let NodeKind::Element(element) = &node.kind else {
                    return;
                };
                let subject = Subject { element };
                let (parser, ns) = evaluate_element(chain, &subject);
                if let ParserVerdict::Diverges(class, frame) = parser {
                    self.set_parser(index, Some(true));
                    self.set_verdict(
                        index,
                        Verdict::Proven {
                            class,
                            evidence: self.origin(chain, frame),
                        },
                    );
                    return;
                }
                let stable = parser == ParserVerdict::Stable;
                self.set_parser(index, stable.then_some(false));
                let (tri, class, frame) = if chain.top().is_some_and(|top| top.ns.is(Ns::Html)) {
                    content_rules::element(chain, &subject, ns)
                } else {
                    (Tri::Maybe, ViolationClass::ChildNotPermitted, None)
                };
                let verdict = match (tri, stable) {
                    (Tri::Yes, _) => Verdict::Proven {
                        class,
                        evidence: self.origin(chain, frame),
                    },
                    (_, false) => Verdict::Unknown,
                    (Tri::No, true) => Verdict::Refuted,
                    (Tri::Maybe, true) => Verdict::ContentUnknown,
                };
                self.set_verdict(index, verdict);
                if element.dynamic_content {
                    return;
                }
                chain.frames.push(Frame {
                    ids: element.ids,
                    ns,
                    attrs: element.attrs,
                    origin: Some((self.skeleton_id, index)),
                });
                for child in self.skeleton.children(index) {
                    self.visit(child, chain);
                }
                chain.frames.pop();
            }
            NodeKind::Text { whitespace_only } => {
                let inert = *whitespace_only && inert_whitespace(chain);
                let verdict = if inert {
                    Verdict::Refuted
                } else {
                    self.text(chain, *whitespace_only, false)
                };
                self.record_text(index, verdict);
            }
            NodeKind::DynamicText => {
                let verdict = self.text(chain, false, true);
                self.record_text(index, verdict);
            }
            NodeKind::Boundary(_) | NodeKind::Component { .. } => {
                if matches!(node.kind, NodeKind::Component { .. }) {
                    (self.on_component)(index, chain);
                }
                let mut fresh = Chain::truncated();
                for child in self.skeleton.children(index) {
                    self.visit(child, &mut fresh);
                }
            }
            NodeKind::SlotOutlet { .. } if self.composed => {}
            NodeKind::SlotContent { .. } | NodeKind::SlotOutlet { .. } => {
                for child in self.skeleton.children(index) {
                    self.visit(child, chain);
                }
            }
        }
    }

    fn record_text(&mut self, index: u32, verdict: Verdict) {
        self.set_parser(
            index,
            match verdict {
                Verdict::Proven { class, .. } => Some(class.family() == Family::Parser),
                Verdict::Refuted | Verdict::ContentUnknown => Some(false),
                Verdict::Unknown | Verdict::Skipped => None,
            },
        );
        self.set_verdict(index, verdict);
    }

    fn text(&self, chain: &mut Chain, whitespace_only: bool, dynamic: bool) -> Verdict {
        let Some(top) = chain.top().copied() else {
            return if whitespace_only {
                Verdict::Refuted
            } else {
                Verdict::Unknown
            };
        };
        let last = chain.frames.len().saturating_sub(1);
        let mut outcome: Option<Tri> = None;
        let mut proven = None;
        for (dispatch, ns) in top.dispatches() {
            if let Some(frame) = chain.frames.get_mut(last) {
                frame.ns = NsSet::one(ns);
            }
            let tri = match dispatch {
                Dispatch::Html => match html_text(chain, whitespace_only, dynamic) {
                    Some(Outcome::Diverge(class, frame)) => {
                        proven.get_or_insert((class, frame));
                        Tri::Yes
                    }
                    Some(_) => Tri::Maybe,
                    None => Tri::No,
                },
                _ => Tri::No,
            };
            outcome = Some(outcome.map_or(tri, |acc| acc.join(tri)));
        }
        if let Some(frame) = chain.frames.get_mut(last) {
            frame.ns = top.ns;
        }
        match (outcome.unwrap_or(Tri::No), proven) {
            (Tri::Yes, Some((class, frame))) => Verdict::Proven {
                class,
                evidence: self.origin(chain, frame),
            },
            (Tri::Maybe, _) => Verdict::Unknown,
            _ if top.ns.is(Ns::Html) => {
                match content_rules::text(chain, whitespace_only, dynamic) {
                    (Tri::Yes, class, frame) => Verdict::Proven {
                        class,
                        evidence: self.origin(chain, frame),
                    },
                    (Tri::No, ..) => Verdict::Refuted,
                    (Tri::Maybe, ..) => Verdict::ContentUnknown,
                }
            }
            _ => Verdict::Refuted,
        }
    }
}
