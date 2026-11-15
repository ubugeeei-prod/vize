//! #12972: frozen typescript-eslint 8.72.2 residual corpus.
use serde::Deserialize;

use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const RULE: &str = "type/no-unnecessary-type-parameters";
const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unnecessary-type-parameters-12972/cases.json"
));

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Corpus {
    schema_version: u8,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    classification: Classification,
    code: String,
    expected: Vec<Expected>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Classification {
    Missed,
    Extra,
    OtherLines,
}

#[derive(Deserialize)]
struct Expected {
    line: usize,
    column: usize,
}

#[test]
fn frozen_residual_matches_typescript_eslint_8_72_2() {
    if !corsa_available() {
        return;
    }
    let corpus: Corpus = serde_json::from_str(CORPUS).expect("#12972 corpus should be valid JSON");
    assert_eq!(corpus.schema_version, 1);
    assert_eq!(corpus.cases.len(), 212);
    let linter =
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]));
    let mut mismatches = Vec::new();
    let mut residuals = [0usize; 3];
    for (index, case) in corpus.cases.iter().enumerate() {
        let source = format!(
            "<script lang=\"ts\">\n{}\nexport {{}};\n</script>",
            case.code
        );
        let filename = format!("NoUnnecessaryTypeParameters{index:03}.vue");
        let result = lint_sfc_with_corsa(&linter, &source, &filename);
        let actual = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(|diagnostic| line_column(&source, diagnostic.start as usize))
            .collect::<Vec<_>>();
        let expected = case
            .expected
            .iter()
            // The frozen snippets become line 2 of the SFC script block.
            .map(|diagnostic| (diagnostic.line + 1, diagnostic.column))
            .collect::<Vec<_>>();
        if actual != expected {
            let classification = match (expected.is_empty(), actual.is_empty()) {
                (false, true) => Classification::Missed,
                (true, false) => Classification::Extra,
                (false, false) => Classification::OtherLines,
                (true, true) => case.classification,
            };
            residuals[classification as usize] += 1;
            mismatches.push(format!(
                "{}: expected {expected:?}, received {actual:?}",
                case.id
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} of 212 frozen cases diverged (missed {}, extra {}, other-lines {}):\n{}",
        mismatches.len(),
        residuals[Classification::Missed as usize],
        residuals[Classification::Extra as usize],
        residuals[Classification::OtherLines as usize],
        mismatches.join("\n")
    );
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let prefix = source.get(..offset).unwrap_or(source);
    let line = prefix.matches('\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.len(), |(_, tail)| tail.len())
        + 1;
    (line, column)
}
