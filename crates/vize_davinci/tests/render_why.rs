// Exercise witness formatting through the public renderer.
// Test-only exemptions live outside the production source inventory.
use vize_davinci::diagnostic::{
    Advisory, Diagnostic, Exemption, Stage, WitnessChain, WitnessKey, WitnessLink,
};
use vize_davinci::fact::ids;
use vize_davinci::pass::AnalysisId;
use vize_davinci::render::{Catalog, EnglishCatalog, Phrase, Renderer, SourceFile};
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
    static EXEMPT: Exemption = Exemption::new("vize_davinci", "render-why-fixture");
    let exempt = Diagnostic::legacy_error(&EXEMPT, Stage::Semantic, Span::new(0, 1), "e");
    assert_eq!(notes(&file, &EnglishCatalog, &exempt), Vec::<String>::new());
}

fn notes<C: Catalog>(file: &SourceFile<'_>, catalog: &C, diagnostic: &Diagnostic) -> Vec<String> {
    Renderer::new(catalog)
        .render(file, None, diagnostic)
        .lines()
        .filter_map(|line| line.trim().strip_prefix("= note: ").map(String::from))
        .collect()
}
