//! The checker: one depth-first pass over a skeleton, evaluating every node
//! against its ancestor chain.
//!
//! Totality: the pass visits each node once, every rule is a bounded scan of
//! the chain, and every verdict is one of three values — no input makes the
//! checker fail, loop or refuse.

use super::chain::{Base, Chain, Dispatch, Frame, NsSet};
use super::class::ViolationClass;
use super::content_rules;
use super::facts::{Ns, facts};
use super::parser_rules::{Outcome, Subject, foreign_start_tag, html_start_tag};
use super::skeleton::{NodeKind, Skeleton};
use super::table_rules::html_text;
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
    let mut walker = Walker {
        skeleton,
        skeleton_id,
        verdicts: vec![Verdict::Skipped; skeleton.nodes.len()],
    };
    let mut chain = context.chain();
    for root in skeleton.roots() {
        walker.visit(root, &mut chain);
    }
    Report {
        verdicts: walker.verdicts,
    }
}

struct Walker<'s> {
    skeleton: &'s Skeleton,
    skeleton_id: u32,
    verdicts: Vec<Verdict>,
}

impl Walker<'_> {
    fn origin(&self, chain: &Chain, frame: Option<usize>) -> Option<(u32, u32)> {
        frame.and_then(|index| chain.frames[index].origin)
    }

    fn visit(&mut self, index: u32, chain: &mut Chain) {
        let node = self.skeleton.node(index);
        match &node.kind {
            NodeKind::Element(element) => {
                let subject = Subject { element };
                let (verdict, ns) = evaluate_element(chain, &subject);
                let verdict = match verdict {
                    ParserVerdict::Diverges(class, frame) => {
                        self.verdicts[index as usize] = Verdict::Proven {
                            class,
                            evidence: self.origin(chain, frame),
                        };
                        return;
                    }
                    ParserVerdict::Stable => {
                        let (tri, class, frame) =
                            if chain.top().is_some_and(|top| top.ns.is(Ns::Html)) {
                                content_rules::element(chain, &subject, ns)
                            } else {
                                (Tri::Maybe, ViolationClass::ChildNotPermitted, None)
                            };
                        match tri {
                            Tri::Yes => Verdict::Proven {
                                class,
                                evidence: self.origin(chain, frame),
                            },
                            Tri::No => Verdict::Refuted,
                            Tri::Maybe => Verdict::ContentUnknown,
                        }
                    }
                    ParserVerdict::Unknown => Verdict::Unknown,
                };
                self.verdicts[index as usize] = verdict;
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
                self.verdicts[index as usize] = self.text(chain, *whitespace_only, false);
            }
            NodeKind::DynamicText => {
                self.verdicts[index as usize] = self.text(chain, false, true);
            }
            NodeKind::Boundary(_) | NodeKind::Component { .. } => {
                let mut fresh = Chain::truncated();
                for child in self.skeleton.children(index) {
                    self.visit(child, &mut fresh);
                }
            }
            NodeKind::SlotContent { .. } | NodeKind::SlotOutlet { .. } => {
                for child in self.skeleton.children(index) {
                    self.visit(child, chain);
                }
            }
        }
    }

    fn text(&self, chain: &mut Chain, whitespace_only: bool, dynamic: bool) -> Verdict {
        let Some(top) = chain.top().copied() else {
            return if whitespace_only {
                Verdict::Refuted
            } else {
                Verdict::Unknown
            };
        };
        let last = chain.frames.len() - 1;
        let mut outcome: Option<Tri> = None;
        let mut proven = None;
        let dispatches: Vec<(Dispatch, Ns)> = top.dispatches().collect();
        for (dispatch, ns) in dispatches {
            chain.frames[last].ns = NsSet::one(ns);
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
        chain.frames[last].ns = top.ns;
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

/// The parser-family verdict for one element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserVerdict {
    /// Not inserted under its authored parent in any consistent context; the
    /// deciding chain frame.
    Diverges(ViolationClass, Option<usize>),
    /// Inserted faithfully in every consistent context.
    Stable,
    /// Depends on something unknown.
    Unknown,
}

/// The parser-family verdict for an element, with the namespaces the element
/// has when it is inserted faithfully.
pub fn evaluate_element(chain: &mut Chain, subject: &Subject<'_>) -> (ParserVerdict, NsSet) {
    let Some(top) = chain.frames.last().copied() else {
        let foreign = if subject.breaks_out(chain) == Tri::Yes {
            NsSet::EMPTY
        } else {
            NsSet::one(Ns::Svg).with(NsSet::one(Ns::MathMl))
        };
        return (
            ParserVerdict::Unknown,
            subject.html_rules_ns().with(foreign),
        );
    };
    let last = chain.frames.len() - 1;
    let mut diverged = None;
    let (mut all_diverge, mut all_stable) = (true, true);
    let mut ns = NsSet::EMPTY;
    let dispatches: Vec<(Dispatch, Ns)> = top.dispatches().collect();
    for (dispatch, parent_ns) in dispatches {
        chain.frames[last].ns = NsSet::one(parent_ns);
        let name = subject.element.id(Ns::Html).map(|id| facts().name(id).1);
        let outcome = match dispatch {
            Dispatch::Html => html_start_tag(chain, subject, true),
            Dispatch::HtmlIntegration => html_start_tag(chain, subject, false),
            Dispatch::MathText if matches!(name, Some("mglyph" | "malignmark")) => {
                Outcome::Stable(NsSet::one(Ns::MathMl))
            }
            Dispatch::MathText => html_start_tag(chain, subject, false),
            Dispatch::AnnotationXml if name == Some("svg") => html_start_tag(chain, subject, false),
            Dispatch::AnnotationXml => foreign_start_tag(chain, subject, Ns::MathMl),
            Dispatch::Foreign(foreign) => foreign_start_tag(chain, subject, foreign),
        };
        match outcome {
            Outcome::Stable(set) => {
                all_diverge = false;
                ns = ns.with(set);
            }
            Outcome::Unknown(set) => {
                all_diverge = false;
                all_stable = false;
                ns = ns.with(set);
            }
            Outcome::Diverge(class, frame) => {
                all_stable = false;
                diverged.get_or_insert((class, frame));
            }
        }
    }
    chain.frames[last].ns = top.ns;
    match (all_diverge, all_stable, diverged) {
        (true, _, Some((class, frame))) => {
            (ParserVerdict::Diverges(class, frame), NsSet::one(Ns::Html))
        }
        (false, true, _) => (ParserVerdict::Stable, ns),
        _ => (ParserVerdict::Unknown, ns),
    }
}
