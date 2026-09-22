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

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::notes;
    use crate::diagnostic::{
        Advisory, Diagnostic, Exemption, Stage, WitnessChain, WitnessKey, WitnessLink,
    };
    use crate::fact::ids;
    use crate::pass::AnalysisId;
    use crate::render::{Catalog, EnglishCatalog, Phrase, SourceFile};
    use vize_s0::{Span, String};

    /// Supplies one fact sentence, ahead of the built-in group phrases.
    struct Knows;

    impl Catalog for Knows {
        fn phrase(&self, phrase: Phrase) -> &str {
            EnglishCatalog.phrase(phrase)
        }

        fn witness(&self, link: &WitnessLink, subject: &str) -> Option<String> {
            (link.group == ids::UNUSED_BINDINGS).then(|| {
                let mut sentence = String::from(subject);
                sentence.push_str(" is never read");
                sentence
            })
        }
    }

    #[test]
    fn known_groups_use_their_sentence_and_an_override_wins() {
        let file = SourceFile::new("a.vue", "const total = 1\n<p>\n");
        let chain = WitnessChain::new(WitnessLink::new(
            ids::UNUSED_BINDINGS,
            Span::new(6, 11),
            WitnessKey::Name("total".into()),
        ))
        .then(WitnessLink::new(
            ids::HTML_ELEMENTS,
            Span::new(16, 18),
            WitnessKey::Index(0),
        ))
        .then(WitnessLink::new(
            AnalysisId::new(9),
            Span::new(0, 0),
            WitnessKey::Index(4),
        ))
        .then(WitnessLink::new(
            AnalysisId::new(9),
            Span::new(0, 0),
            WitnessKey::Artifact,
        ));
        let hint = Diagnostic::new(Advisory::Hint, Stage::Semantic, Span::new(6, 11), "hint")
            .with_witness(chain);
        let notes = notes(&file, &Knows, &hint);
        let notes: Vec<&str> = notes.iter().map(String::as_str).collect();
        assert_eq!(
            notes,
            [
                "because `total` is never read",
                "because `<p` is the ancestor element this nesting proof cites",
                "because fact group 9 holds for #4",
                "because fact group 9 holds for a.vue",
            ]
        );
    }

    #[test]
    fn an_unproven_diagnostic_and_a_legacy_exemption_have_no_notes() {
        let file = SourceFile::new("a.vue", "x");
        let plain = Diagnostic::new(Advisory::Warning, Stage::Semantic, Span::new(0, 1), "w");
        assert_eq!(notes(&file, &EnglishCatalog, &plain), Vec::<String>::new());
        static EXEMPT: Exemption = Exemption::new("why", "fixture");
        let exempt = Diagnostic::legacy_error(&EXEMPT, Stage::Semantic, Span::new(0, 1), "e");
        assert_eq!(notes(&file, &EnglishCatalog, &exempt), Vec::<String>::new());
    }
}
