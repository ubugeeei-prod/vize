//! The content-model family: DOM-stable nesting the content models of §4
//! forbid. Evaluated only for nodes the parser family proved stable, so a
//! node carries at most one class and parser divergences take precedence.

use super::chain::{Base, Chain, Frame, NsSet, member_of};
use super::class::ViolationClass as V;
use super::facts::{Attr, Cond, Ns, Row, eval_attr_cond, facts};
use super::parser_rules::Subject;
use super::tri::Tri;

/// A three-valued content-model verdict with the deciding chain frame.
pub type Finding = (Tri, V, Option<usize>);

/// The nearest ancestor whose content model applies (skipping transparent
/// elements); `Err` carries whether a violation is still possible when the
/// chain cannot name it.
fn content_parent(chain: &Chain) -> Result<(usize, &Frame), Tri> {
    for (index, frame) in chain.frames.iter().enumerate().rev() {
        if !frame.ns.is(Ns::Html) {
            return Err(if frame.ns.has(Ns::Html) {
                Tri::Maybe
            } else {
                Tri::No
            });
        }
        if !facts().is(Row::Transparent, frame.id(Ns::Html)) {
            return Ok((index, frame));
        }
    }
    Err(match chain.base {
        Base::Document => Tri::No,
        Base::Truncated => Tri::Maybe,
    })
}

/// `source`/`track` directly inside `audio`/`video` precede the transparent
/// part of the media content model.
fn media_prefix(chain: &Chain, name: &str) -> bool {
    matches!(name, "source" | "track")
        && chain
            .top()
            .and_then(|frame| frame.id(Ns::Html))
            .is_some_and(|id| matches!(facts().name(id).1, "audio" | "video"))
}

/// Content-model check of an element the parser family proved stable in
/// `ns` (the namespaces it may have).
pub fn element(chain: &Chain, subject: &Subject<'_>, ns: NsSet) -> Finding {
    let permitted = permitted_child(chain, subject, ns);
    if permitted.0 == Tri::Yes {
        return permitted;
    }
    let nested = interactive(chain, subject, ns);
    if nested.0 == Tri::Yes || permitted.0 == Tri::No {
        return nested;
    }
    permitted
}

/// The content parent's content model lists the element.
fn permitted_child(chain: &Chain, subject: &Subject<'_>, ns: NsSet) -> Finding {
    let t = facts();
    let name = subject.element.id(Ns::Html).map_or("", |id| t.name(id).1);
    let (index, parent) = match content_parent(chain) {
        Ok(found) => found,
        Err(tri) => return (tri, V::ChildNotPermitted, None),
    };
    if media_prefix(chain, name) {
        return (Tri::No, V::ChildNotPermitted, None);
    }
    let Some(row) = parent.id(Ns::Html).and_then(|id| t.children(id)) else {
        return (Tri::No, V::ChildNotPermitted, None);
    };
    let class = if row.text {
        V::PhrasingContentExpected
    } else {
        V::ChildNotPermitted
    };
    let structural = |cond: Cond| match cond {
        Cond::InMap => chain.has_open("map").0,
        _ => Tri::Maybe,
    };
    let mut allowed: Option<Tri> = None;
    for ns in ns.iter() {
        let tri = match subject.element.id(ns) {
            Some(id) => match row.condition(id) {
                None => Tri::No,
                Some(None) => Tri::Yes,
                Some(Some(cond)) => eval_attr_cond(cond, |attr| subject.element.attrs.get(attr))
                    .unwrap_or_else(|| structural(cond)),
            },
            // A tag the table does not list (a custom or unknown element) is
            // outside the declared domain.
            None => Tri::Maybe,
        };
        allowed = Some(allowed.map_or(tri, |acc| acc.join(tri)));
    }
    (allowed.unwrap_or(Tri::Maybe).not(), class, Some(index))
}

/// `a` forbids interactive content, `a` descendants and `tabindex`
/// descendants; `button` forbids interactive content and `tabindex`.
fn interactive(chain: &Chain, subject: &Subject<'_>, ns: NsSet) -> Finding {
    if !ns.is(Ns::Html) {
        return (Tri::No, V::InteractiveContentNested, None);
    }
    let element = subject.element;
    let category = member_of(
        Row::CatInteractive,
        element.id(Ns::Html),
        &element.attrs,
        |_| Tri::Maybe,
    );
    let forbidden_in_button = category.or(element.attrs.get(Attr::Tabindex));
    let is_anchor = Tri::from_bool(element.id(Ns::Html) == facts().id(Ns::Html, "a"));
    let forbidden_in_anchor = forbidden_in_button.or(is_anchor);
    let mut maybe = false;
    for (index, frame) in chain.frames.iter().enumerate().rev() {
        let forbidden = frame
            .is_html("a")
            .and(forbidden_in_anchor)
            .or(frame.is_html("button").and(forbidden_in_button));
        match forbidden {
            Tri::Yes => return (Tri::Yes, V::InteractiveContentNested, Some(index)),
            Tri::Maybe => maybe = true,
            Tri::No => {}
        }
    }
    if chain.base == Base::Truncated && forbidden_in_anchor != Tri::No {
        maybe = true;
    }
    let verdict = if maybe { Tri::Maybe } else { Tri::No };
    (verdict, V::InteractiveContentNested, None)
}

/// Content-model check of a text node whose parent is an HTML element.
pub fn text(chain: &Chain, whitespace_only: bool, dynamic: bool) -> Finding {
    if whitespace_only {
        return (Tri::No, V::ChildNotPermitted, None);
    }
    match content_parent(chain) {
        Ok((index, parent)) => match parent.id(Ns::Html).and_then(|id| facts().children(id)) {
            Some(row) if !row.text => {
                let verdict = if dynamic { Tri::Maybe } else { Tri::Yes };
                (verdict, V::ChildNotPermitted, Some(index))
            }
            _ => (Tri::No, V::ChildNotPermitted, None),
        },
        Err(tri) => (tri.and(Tri::Maybe), V::ChildNotPermitted, None),
    }
}
