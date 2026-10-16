//! Original Matrix sources and independently executed ESLint token references.

use serde::Deserialize;
use vize_l0::String;
use vize_patina::rules::opinionated::vue::ThisInTemplate;
use vize_patina::{Linter, RuleRegistry, Severity};

#[derive(Debug, Deserialize, PartialEq)]
struct Range {
    start: u32,
    end: u32,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    filename: String,
    source: String,
    expected: Vec<Range>,
}

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ThisInTemplate));
    Linter::with_registry(registry)
}

#[test]
fn exact_this_tokens_match_pinned_reference_for_originals_and_neighboring_constructs() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "fixtures/this-in-template-reference/cases.json"
    ))
    .unwrap();
    assert_eq!(corpus.cases.len(), 25);
    let linter = linter();
    for case in corpus.cases {
        let result = linter.lint_sfc(case.source.as_str(), case.filename.as_str());
        let ranges: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                assert_eq!(diagnostic.rule_name, "vue/this-in-template", "{}", case.id);
                assert_eq!(diagnostic.severity, Severity::Warning, "{}", case.id);
                assert_eq!(
                    diagnostic.message.as_str(),
                    "Unexpected usage of 'this.' in a template expression.",
                    "{}",
                    case.id
                );
                assert!(diagnostic.fix.is_none());
                assert!(diagnostic.labels.is_empty());
                assert_eq!(
                    diagnostic.help.as_ref().map(|help| help.as_str()),
                    Some("Remove the 'this.' prefix; Vue resolves template identifiers against the component instance automatically.")
                );
                Range { start: diagnostic.start, end: diagnostic.end }
            })
            .collect();
        assert_eq!(result.error_count, 0, "{}", case.id);
        assert_eq!(
            result.warning_count as usize,
            case.expected.len(),
            "{}",
            case.id
        );
        assert_eq!(ranges, case.expected, "{}", case.id);
    }
}

#[test]
fn existing_template_suppression_applies_to_every_this_token() {
    let source = concat!(
        "<template>\n",
        "<!-- eslint-disable-next-line vue/this-in-template -->\n",
        "<div>{{ this.hidden + this.hiddenAgain }}</div>\n",
        "<div>{{ this.visible }}</div>\n",
        "</template>"
    );
    let result = linter().lint_sfc(source, "suppressed.vue");
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = result.diagnostics.first().unwrap();
    let start = source.find("this.visible").unwrap() as u32;
    assert_eq!((diagnostic.start, diagnostic.end), (start, start + 4));
}
