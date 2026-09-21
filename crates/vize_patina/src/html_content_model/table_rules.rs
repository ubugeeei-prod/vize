//! The table insertion modes ("in table", "in table body", "in row", "in
//! column group") and the character-token rules: the part of §13.2.6 that
//! decides by the current node alone.

use super::chain::{Base, Chain, NsSet, Scan};
use super::class::ViolationClass as V;
use super::facts::{Ns, Row, facts};
use super::parser_rules::{Outcome, Subject};
use super::tri::Tri;

/// A start tag whose current node is an HTML `table`, table section, `tr`
/// or `colgroup` (the "in table", "in table body", "in row" and "in column
/// group" insertion modes).
pub(super) fn table_start_tag(chain: &Chain, subject: &Subject<'_>, cname: &str) -> Outcome {
    let t = facts();
    let top = chain.frames.len().checked_sub(1);
    let xid = subject.html_id();
    let name = subject.html_name();
    let stable = subject.html_rules_ns();
    let decide = |member: Tri, otherwise: Outcome| match member {
        Tri::Yes => Outcome::Stable(stable),
        Tri::Maybe => Outcome::Unknown(stable),
        Tri::No => otherwise,
    };
    if t.is(Row::DocumentPart, xid) {
        return Outcome::Diverge(V::DocumentElementDropped, None);
    }
    if cname == "colgroup" {
        let member = subject.member(Row::ColgroupChildren, chain);
        return decide(member, Outcome::Diverge(V::TableAutoClosed, top));
    }
    let (children, wrapped, closers): (Row, Option<Row>, &[&str]) = match cname {
        "table" => (Row::TableChildren, Some(Row::TableWrapped), &[]),
        "tr" => (
            Row::TrChildren,
            None,
            &[
                "caption", "col", "colgroup", "tbody", "tfoot", "thead", "tr",
            ],
        ),
        _ => (
            Row::SectionChildren,
            Some(Row::SectionWrapped),
            &["caption", "col", "colgroup", "tbody", "tfoot", "thead"],
        ),
    };
    if name == "form" {
        let (open, evidence) = chain.has_open("form");
        return match open {
            Tri::Yes => Outcome::Diverge(V::NestedFormDropped, evidence),
            Tri::No => Outcome::Stable(stable),
            Tri::Maybe => Outcome::Unknown(stable),
        };
    }
    let otherwise = if wrapped.is_some_and(|row| t.is(row, xid)) {
        Outcome::Diverge(V::TableWrapperInserted, top)
    } else if name == "table" || closers.contains(&name) {
        Outcome::Diverge(V::TableAutoClosed, top)
    } else {
        Outcome::Diverge(V::FosterParented, top)
    };
    decide(subject.member(children, chain), otherwise)
}

/// The current node is a `form` the "in table", "in table body" or "in row"
/// rules inserted: they pop it at once, so nothing is ever inserted into it.
pub(super) fn form_in_table(chain: &Chain) -> Scan {
    let len = chain.frames.len();
    let top = len.checked_sub(1);
    let is_form = chain
        .top()
        .is_some_and(|frame| frame.is_html("form") == Tri::Yes);
    if !is_form {
        return (Tri::No, None);
    }
    let parent = match len.checked_sub(2).map(|index| &chain.frames[index]) {
        Some(frame) => ["table", "tbody", "thead", "tfoot", "tr"]
            .iter()
            .fold(Tri::No, |acc, tag| acc.or(frame.is_html(tag))),
        None => match chain.base {
            Base::Document => Tri::No,
            Base::Truncated => Tri::Maybe,
        },
    };
    (parent, if parent == Tri::Yes { top } else { None })
}

/// A character token (static text or an expression) whose current node is
/// an HTML element; `None` when the text is inserted as that node's child.
pub fn html_text(chain: &Chain, whitespace_only: bool, dynamic: bool) -> Option<Outcome> {
    let top = chain.frames.len().checked_sub(1);
    let cid = chain.top().and_then(|frame| frame.id(Ns::Html))?;
    if facts().is(Row::Void, Some(cid)) {
        return Some(Outcome::Diverge(V::VoidElementContent, top));
    }
    match form_in_table(chain).0 {
        Tri::Yes if !dynamic => return Some(Outcome::Diverge(V::FormInTableEmptied, top)),
        Tri::No => {}
        _ => return Some(Outcome::Unknown(NsSet::EMPTY)),
    }
    let class = match facts().name(cid).1 {
        "table" | "tbody" | "thead" | "tfoot" | "tr" => V::FosterParented,
        "colgroup" => V::TableAutoClosed,
        _ => return None,
    };
    if whitespace_only {
        None
    } else if dynamic {
        Some(Outcome::Unknown(NsSet::EMPTY))
    } else {
        Some(Outcome::Diverge(class, top))
    }
}
