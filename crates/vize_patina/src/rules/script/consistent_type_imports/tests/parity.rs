//! typescript-eslint `consistent-type-imports` with vue-eslint-parser,
//! recorded on each SFC case, beside this rule's reports on the same source.

use super::reports;

const FIXTURE: &str = include_str!(
    "../../../../../tests/fixtures/typescript-eslint-consistent-type-imports-template-uses.txt"
);

#[derive(Default)]
struct Case {
    label: String,
    sfc: String,
    typescript_eslint: Vec<String>,
    vize: Vec<String>,
    kept: Option<String>,
}

fn cases() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    let mut section = "";
    for line in FIXTURE.lines() {
        if let Some(label) = line.strip_prefix("=== ") {
            cases.push(Case {
                label: label.to_string(),
                ..Case::default()
            });
            continue;
        }
        if let Some(name) = line.strip_prefix("--- ") {
            section = match name {
                "sfc" => "sfc",
                "typescript-eslint" => "typescript-eslint",
                "vize" => "vize",
                "kept" => "kept",
                other => panic!("unknown section {other}"),
            };
            continue;
        }
        // The lines before the first case describe the fixture.
        let Some(case) = cases.last_mut() else {
            continue;
        };
        match section {
            "sfc" => {
                case.sfc.push_str(line);
                case.sfc.push('\n');
            }
            "typescript-eslint" => case.typescript_eslint.push(line.to_string()),
            "vize" => case.vize.push(line.to_string()),
            _ => case.kept = Some(line.to_string()),
        }
    }
    cases
}

/// `line N`, the part both tools share: their wording differs.
fn lines(reports: &[String]) -> Vec<&str> {
    reports
        .iter()
        .map(|report| report.split(':').next().unwrap_or_default())
        .collect()
}

#[test]
fn every_recorded_case_is_present() {
    let cases = cases();
    assert_eq!(cases.len(), 58);
    let kept: Vec<&str> = cases
        .iter()
        .filter(|case| case.kept.is_some())
        .map(|case| case.label.as_str())
        .collect();
    assert_eq!(kept, ["template[27]", "type-import-read-by-template"]);
}

#[test]
fn the_rule_reports_what_the_fixture_records() {
    for case in cases() {
        let found: Vec<String> = reports(&case.sfc, "Component.vue", None)
            .into_iter()
            .map(|(line, message)| format!("line {line}: {message}"))
            .collect();
        assert_eq!(found, case.vize, "{}", case.label);
    }
}

#[test]
fn the_reported_lines_are_typescript_eslints_unless_kept() {
    for case in cases() {
        if case.kept.is_some() {
            continue;
        }
        assert_eq!(
            lines(&case.vize),
            lines(&case.typescript_eslint),
            "{}",
            case.label
        );
    }
}
