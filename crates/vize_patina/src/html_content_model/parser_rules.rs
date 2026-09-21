//! The parser family: the tree-construction rules (§13.2.6) that can put a
//! serialized node somewhere other than under its authored parent.
//!
//! Each function mirrors one spec rule, in spec order, over the three-valued
//! stack scans of [`Chain`]. A rule sequence diverges as soon as one step is
//! proven; an unknown step before a proven one still yields a divergence
//! (whichever step fires, the node leaves its parent), and an unknown step
//! with no proven one after it yields `unknown`.

use super::chain::{Chain, NsSet, Scan, member_of};
use super::class::ViolationClass as V;
use super::facts::{Cond, ElemId, Ns, Row, facts};
use super::skeleton::Element;
use super::table_rules::{form_in_table, table_start_tag};
use super::tri::Tri;

/// What inserting a node does, under one dispatch of its parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Inserted as the authored parent's child, in one of these namespaces.
    Stable(NsSet),
    /// Not inserted as the authored parent's child; the deciding chain frame.
    Diverge(V, Option<usize>),
    /// Depends on something unknown; the namespaces it has if it is stable.
    Unknown(NsSet),
}

/// An element being inserted.
#[derive(Debug, Clone, Copy)]
pub struct Subject<'e> {
    /// The element.
    pub element: &'e Element,
}

impl Subject<'_> {
    pub(super) fn html_id(&self) -> Option<ElemId> {
        self.element.id(Ns::Html)
    }

    /// The start tag's name as the tokenizer emits it.
    pub fn html_name(&self) -> &str {
        self.element.name.as_str()
    }

    /// The namespace the insertion-mode rules create this element in.
    pub fn html_rules_ns(&self) -> NsSet {
        NsSet::one(match self.html_name() {
            "svg" => Ns::Svg,
            "math" => Ns::MathMl,
            _ => Ns::Html,
        })
    }

    pub(super) fn member(&self, row: Row, chain: &Chain) -> Tri {
        member_of(
            row,
            self.html_id(),
            &self.element.attrs,
            |cond| match cond {
                Cond::InMap => chain.has_open("map").0,
                _ => Tri::Maybe,
            },
        )
    }

    /// Whether the rules for foreign content break this start tag out.
    pub fn breaks_out(&self, chain: &Chain) -> Tri {
        self.member(Row::ForeignBreakout, chain)
    }
}

struct Seq {
    maybe: bool,
    ns: NsSet,
}

impl Seq {
    fn step(&mut self, scan: Scan, class: V) -> Option<Outcome> {
        match scan.0 {
            Tri::Yes => Some(Outcome::Diverge(class, scan.1)),
            Tri::Maybe => {
                self.maybe = true;
                None
            }
            Tri::No => None,
        }
    }

    fn finish(self) -> Outcome {
        if self.maybe {
            Outcome::Unknown(self.ns)
        } else {
            Outcome::Stable(self.ns)
        }
    }
}

/// `if cond { then } else { otherwise }` over three-valued scans.
fn scan_if(cond: Scan, then: Scan, otherwise: Scan) -> Scan {
    match cond.0 {
        Tri::Yes => then,
        Tri::No => otherwise,
        Tri::Maybe => match (then.0, otherwise.0) {
            (Tri::Yes, Tri::Yes) => then,
            (Tri::No, Tri::No) => (Tri::No, None),
            _ => (Tri::Maybe, None),
        },
    }
}

fn scan_and(left: Scan, right: Scan) -> Scan {
    match left.0.and(right.0) {
        Tri::Yes => right,
        other => (other, None),
    }
}

/// The current node is the HTML element whose name satisfies `pred`.
fn parent_html(chain: &Chain, parent_is_html: bool, pred: impl Fn(&str) -> bool) -> Scan {
    let top = chain.frames.len().checked_sub(1);
    let matches = parent_is_html
        && chain
            .top()
            .and_then(|frame| frame.id(Ns::Html))
            .is_some_and(|id| pred(facts().name(id).1));
    if matches {
        (Tri::Yes, top)
    } else {
        (Tri::No, None)
    }
}

/// A start tag processed by the insertion-mode rules ("in body" and the
/// table modes). `parent_is_html` is false when the current node is an
/// SVG/MathML integration point.
pub fn html_start_tag(chain: &Chain, subject: &Subject<'_>, parent_is_html: bool) -> Outcome {
    let t = facts();
    let top = chain.frames.len().checked_sub(1);
    let xid = subject.html_id();
    let name = subject.html_name();
    let stable = subject.html_rules_ns();
    if parent_is_html {
        let cid = chain.top().and_then(|frame| frame.id(Ns::Html));
        if t.is(Row::RawText, cid) {
            return Outcome::Diverge(V::RawTextContent, top);
        }
        if t.is(Row::Void, cid) {
            return Outcome::Diverge(V::VoidElementContent, top);
        }
        if t.is(Row::ScriptingDependent, cid) {
            return Outcome::Unknown(stable);
        }
        let cname = cid.map_or("", |id| t.name(id).1);
        if matches!(
            cname,
            "table" | "tbody" | "thead" | "tfoot" | "tr" | "colgroup"
        ) {
            return table_start_tag(chain, subject, cname);
        }
    }
    let emptied = if parent_is_html {
        form_in_table(chain)
    } else {
        (Tri::No, None)
    };
    if emptied.0 == Tri::Yes {
        return Outcome::Diverge(V::FormInTableEmptied, emptied.1);
    }
    if t.is(Row::DocumentPart, xid) {
        return Outcome::Diverge(V::DocumentElementDropped, None);
    }
    if t.is(Row::TablePart, xid) {
        return Outcome::Diverge(V::TablePartMisplaced, top);
    }
    let p = || chain.in_scope("p", Row::ButtonScope);
    let implied_end = |except: &'static str| {
        parent_html(chain, parent_is_html, move |parent| {
            parent != except && t.is(Row::ImpliedEnd, t.id(Ns::Html, parent))
        })
    };
    let mut seq = Seq {
        maybe: false,
        ns: stable,
    };
    macro_rules! step {
        ($scan:expr, $class:expr) => {
            if let Some(outcome) = seq.step($scan, $class) {
                return outcome;
            }
        };
    }
    step!(emptied, V::FormInTableEmptied);
    match name {
        "image" => return Outcome::Diverge(V::ImageRenamed, None),
        "form" => {
            step!(chain.has_open("form"), V::NestedFormDropped);
            step!(p(), V::ParagraphAutoClosed);
        }
        "li" => {
            step!(chain.list_item_loop(&["li"]), V::ListItemAutoClosed);
            step!(p(), V::ParagraphAutoClosed);
        }
        "dd" | "dt" => {
            step!(chain.list_item_loop(&["dd", "dt"]), V::ListItemAutoClosed);
            step!(p(), V::ParagraphAutoClosed);
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            step!(p(), V::ParagraphAutoClosed);
            let heading = parent_html(chain, parent_is_html, |parent| {
                t.is(Row::Heading, t.id(Ns::Html, parent))
            });
            step!(heading, V::HeadingAutoClosed);
        }
        "hr" => {
            step!(p(), V::ParagraphAutoClosed);
            let select = chain.in_scope("select", Row::Scope);
            step!(scan_and(select, implied_end("")), V::ImpliedEndTagClosed);
        }
        "plaintext" => {
            step!(p(), V::ParagraphAutoClosed);
            return Outcome::Diverge(V::RawTextContent, None);
        }
        "button" => step!(chain.in_scope("button", Row::Scope), V::ButtonAutoClosed),
        // The adoption agency only restructures when the open `a` is also in
        // scope; across a non-marker scope boundary (`select`, an SVG/MathML
        // integration point) it aborts and the new `a` is inserted in place
        // (the content-model family still reports the nested `a`).
        "a" => step!(
            scan_and(chain.anchor_after_marker(), chain.in_scope("a", Row::Scope)),
            V::FormattingAdopted
        ),
        "nobr" => step!(chain.in_scope("nobr", Row::Scope), V::FormattingAdopted),
        "select" | "input" => {
            step!(chain.in_scope("select", Row::Scope), V::SelectAutoClosed);
        }
        "option" | "optgroup" => {
            let select = chain.in_scope("select", Row::Scope);
            let in_select = implied_end(if name == "option" { "optgroup" } else { "" });
            let outside = parent_html(chain, parent_is_html, |parent| parent == "option");
            step!(scan_if(select, in_select, outside), V::ImpliedEndTagClosed);
        }
        "rb" | "rtc" | "rp" | "rt" => {
            let ruby = chain.in_scope("ruby", Row::Scope);
            let except = if matches!(name, "rp" | "rt") {
                "rtc"
            } else {
                ""
            };
            step!(scan_and(ruby, implied_end(except)), V::ImpliedEndTagClosed);
        }
        _ if t.is(Row::ClosesP, xid) => step!(p(), V::ParagraphAutoClosed),
        _ => {}
    }
    seq.finish()
}

/// A start tag processed by the rules for foreign content, whose adjusted
/// current node is in `ns`.
pub fn foreign_start_tag(chain: &Chain, subject: &Subject<'_>, ns: Ns) -> Outcome {
    match subject.breaks_out(chain) {
        Tri::Yes => Outcome::Diverge(V::ForeignContentBreakout, chain.frames.len().checked_sub(1)),
        Tri::Maybe => Outcome::Unknown(NsSet::one(ns)),
        Tri::No => Outcome::Stable(NsSet::one(ns)),
    }
}
