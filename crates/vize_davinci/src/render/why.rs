//! Witness-derived "why" notes (P4-14c, charter #42).
//!
//! `assurance.md`'s witnesses double as explanations: a proven diagnostic
//! carries the fact chain that proves it, so the renderer can say *why* in the
//! facts' own terms — one `= note: because …` footer per
//! [`WitnessLink`], in proof order. The sentence for a link comes from the
//! [`Catalog`] ([`Catalog::witness`]), which knows the fact groups a build
//! registers; a group it does not know still gets an honest, localized
//! fallback naming the group and the fact's subject, so a proof is never
//! rendered as nothing. A legacy exemption is not a proof and adds no note.

use alloc::vec::Vec;
use core::fmt::Write as _;

use super::catalog::{Catalog, Phrase};
use super::source::SourceFile;
use crate::diagnostic::{Diagnostic, WitnessKey, WitnessLink};
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
            let subject = subject(file, link);
            let fact = catalog.witness(link, &subject).unwrap_or_else(|| {
                let mut group = String::new("");
                let _ = write!(group, "{}", link.group.index());
                fill(
                    catalog.phrase(Phrase::FactFallback),
                    &[("group", &group), ("subject", &subject)],
                )
            });
            fill(catalog.phrase(Phrase::Because), &[("fact", &fact)])
        })
        .collect()
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
        out.extend(chars.by_ref().take(SUBJECT_LIMIT));
        if chars.next().is_some() {
            out.push('…');
        }
        out.push('`');
        return out;
    }
    let _ = match &link.key {
        WitnessKey::Artifact => write!(out, "{}", file.path()),
        WitnessKey::Node(node) => write!(out, "{node}"),
        WitnessKey::Index(index) => write!(out, "#{index}"),
        WitnessKey::Name(name) => write!(out, "`{name}`"),
    };
    out
}

/// `template` with each `{name}` replaced by its value.
fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::from(template);
    for (name, value) in vars {
        let mut placeholder = String::new("{");
        placeholder.push_str(name);
        placeholder.push('}');
        out = out.replace(placeholder.as_str(), value).into();
    }
    out
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::notes;
    use crate::diagnostic::{Advisory, Diagnostic, Stage, WitnessChain, WitnessKey, WitnessLink};
    use crate::pass::AnalysisId;
    use crate::render::{Catalog, EnglishCatalog, Phrase, SourceFile};
    use vize_s0::{Span, String};

    /// Knows one fact group, as a build that registers it would.
    struct Knows;

    impl Catalog for Knows {
        fn phrase(&self, phrase: Phrase) -> &str {
            EnglishCatalog.phrase(phrase)
        }

        fn witness(&self, link: &WitnessLink, subject: &str) -> Option<String> {
            (link.group == AnalysisId::new(3)).then(|| {
                let mut sentence = String::from(subject);
                sentence.push_str(" is never read");
                sentence
            })
        }
    }

    #[test]
    fn a_known_fact_speaks_the_catalog_and_an_unknown_one_falls_back() {
        let file = SourceFile::new("a.vue", "const total = 1");
        let chain = WitnessChain::new(WitnessLink::new(
            AnalysisId::new(3),
            Span::new(6, 11),
            WitnessKey::Name("total".into()),
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
                "because fact group 9 holds for #4",
                "because fact group 9 holds for a.vue",
            ]
        );
    }

    #[test]
    fn an_unproven_diagnostic_has_no_notes() {
        let file = SourceFile::new("a.vue", "x");
        let plain = Diagnostic::new(Advisory::Warning, Stage::Semantic, Span::new(0, 1), "w");
        assert_eq!(notes(&file, &Knows, &plain), Vec::<String>::new());
    }
}
