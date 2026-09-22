//! Witness-derived "why" notes (P4-14c).
//!
//! A proven diagnostic carries the fact chain that proves it, so the renderer
//! says why in the facts' own terms: one `= note: because …` footer per
//! [`WitnessLink`](crate::diagnostic::WitnessLink), in proof order. The
//! P4-3c and P4-11b groups have a sentence in the [`Catalog`]; a group it
//! does not know still names the group and the fact's subject. A legacy
//! exemption is not a proof and adds no note.

use alloc::vec::Vec;
use core::fmt::Write as _;

use super::catalog::{Catalog, Phrase};
use super::source::SourceFile;
use crate::diagnostic::{Diagnostic, WitnessKey, WitnessLink};
use crate::fact::ids;
use crate::pass::AnalysisId;
use vize_s0::String;

/// Characters of a fact's source text quoted in a note before it is cut.
const SUBJECT_LIMIT: usize = 40;

/// The note text for every link of `diagnostic`'s witness chain.
pub(crate) fn notes<C: Catalog + ?Sized>(
    file: &SourceFile<'_>,
    catalog: &C,
    diagnostic: &Diagnostic,
) -> Vec<String> {
    let Some(chain) = diagnostic.witness_chain() else {
        return Vec::new();
    };
    chain
        .links()
        .iter()
        .map(|link| {
            let quoted = subject(file, link);
            let fact =
                catalog.witness(link, quoted.as_str()).unwrap_or_else(|| {
                    match group_phrase(link.group) {
                        Some(phrase) => {
                            fill(catalog.phrase(phrase), &[("subject", quoted.as_str())])
                        }
                        None => {
                            let mut group = String::new("");
                            let _ = write!(group, "{}", link.group.index());
                            fill(
                                catalog.phrase(Phrase::FactFallback),
                                &[("group", group.as_str()), ("subject", quoted.as_str())],
                            )
                        }
                    }
                });
            fill(catalog.phrase(Phrase::Because), &[("fact", fact.as_str())])
        })
        .collect()
}

/// The sentence the renderer has for a production group, if it has one.
fn group_phrase(group: AnalysisId) -> Option<Phrase> {
    if group == ids::UNUSED_BINDINGS {
        Some(Phrase::UnusedBinding)
    } else if group == ids::HTML_ELEMENTS {
        Some(Phrase::HtmlElement)
    } else if group == ids::HTML_COMPOSED_NESTING {
        Some(Phrase::HtmlComposedNesting)
    } else {
        None
    }
}

/// What a link's fact is about: its source text quoted (first line, cut at
/// [`SUBJECT_LIMIT`] characters), or its key when the span covers nothing.
fn subject(file: &SourceFile<'_>, link: &WitnessLink) -> String {
    let (start, end) = file.range(link.span);
    let text = file.text()[start..end].lines().next().unwrap_or("").trim();
    let mut out = String::new("");
    if !text.is_empty() {
        out.push('`');
        let mut chars = text.chars();
        for _ in 0..SUBJECT_LIMIT {
            let Some(ch) = chars.next() else {
                out.push('`');
                return out;
            };
            out.push(ch);
        }
        if chars.next().is_some() {
            out.push('…');
        }
        out.push('`');
        return out;
    }
    match &link.key {
        WitnessKey::Artifact => {
            let _ = write!(out, "{}", file.path());
        }
        WitnessKey::Node(node) => {
            let _ = write!(out, "{node}");
        }
        WitnessKey::Index(index) => {
            let _ = write!(out, "#{index}");
        }
        WitnessKey::Name(name) => {
            out.push('`');
            out.push_str(name.as_str());
            out.push('`');
        }
    }
    out
}

/// `template` with each `{name}` replaced by its value.
fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::new("");
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let name = &after[..end];
        let value = vars
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value);
        if let Some(value) = value {
            out.push_str(value);
        } else {
            out.push('{');
            out.push_str(name);
            out.push('}');
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}
