//! The parser-family verdict for one element: the insertion the HTML
//! parser performs for its start tag, across every dispatch the chain allows.

use super::chain::{Chain, Dispatch, NsSet};
use super::class::ViolationClass;
use super::facts::{Ns, facts};
use super::parser_rules::{Outcome, Subject, foreign_start_tag, html_start_tag};

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
        // The mount point is unknown; the declared mount assumption places
        // the element where its compiled namespace holds.
        return (
            ParserVerdict::Unknown,
            NsSet::one(subject.element.compiler_ns),
        );
    };
    let last = chain.frames.len() - 1;
    let mut diverged = None;
    let (mut all_diverge, mut all_stable) = (true, true);
    let mut ns = NsSet::EMPTY;
    for (dispatch, parent_ns) in top.dispatches() {
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
