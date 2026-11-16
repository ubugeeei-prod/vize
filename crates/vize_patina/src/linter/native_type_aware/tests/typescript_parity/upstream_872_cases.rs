//! Selected v8.72.2 cases reported in #12975, preserved byte-for-byte as data.
use super::super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

const CORPUS: &str = include_str!("../fixtures/typescript_eslint_8_72_2_selected.json");

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ES2022"
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
"#;

const CLASS_FIXTURE: &str = r#"// typescript-eslint v8.72.2 tests/fixtures/class.ts
export class Error {}
export const console = { log() {} };
export class Reducable { reduce() {} }
export class Function {}
"#;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    rule: String,
    expected_diagnostics: usize,
    code: String,
}

#[test]
fn selected_typescript_eslint_872_cases_match() {
    if !corsa_available() {
        return;
    }

    let corpus: Value = serde_json::from_str(CORPUS).expect("selected upstream corpus JSON");
    assert_eq!(corpus["schema"], "vize.typescript-eslint.selected-corpus");
    assert_eq!(corpus["version"], 1);
    assert_eq!(corpus["issue"], 12975);
    assert_eq!(corpus["upstream"]["release"], "v8.72.2");
    assert_eq!(
        corpus["upstream"]["commit"],
        "12e452f0292212bf9bf443b3988ec230f4ae7097"
    );
    let cases: Vec<Case> =
        serde_json::from_value(corpus["cases"].clone()).expect("selected upstream cases");
    assert_eq!(corpus["caseCount"], cases.len());
    assert_eq!(cases.len(), 46);

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("typescript-eslint-8-72-2-12975-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(project.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");
    std::fs::write(source_dir.join("class.ts"), CLASS_FIXTURE).expect("class fixture");

    let mut linters = BTreeMap::new();
    let mut mismatches = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let slug = case
            .id
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '-'
                }
            })
            .collect::<String>();
        let path = source_dir.join(format!("{index:02}-{slug}.vue"));
        let source = format!("<script lang=\"ts\">\n{}\n</script>\n", case.code);
        std::fs::write(&path, &source).expect("case source");

        let linter = linters.entry(case.rule.clone()).or_insert_with(|| {
            Linter::with_preset(LintPreset::Opinionated)
                .with_enabled_rules(Some(vec![case.rule.clone().into()]))
        });
        let result =
            lint_sfc_with_corsa(linter, &source, path.to_str().expect("UTF-8 fixture path"));
        let actual = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == case.rule)
            .count();
        if actual != case.expected_diagnostics {
            let findings = result
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    format!(
                        "{}@{}..{}: {}",
                        diagnostic.rule_name, diagnostic.start, diagnostic.end, diagnostic.message
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
            mismatches.push(format!(
                "{} expected {} diagnostic(s), got {actual}: {findings}",
                case.id, case.expected_diagnostics
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "{}/{} selected upstream cases disagree:\n{}",
        mismatches.len(),
        cases.len(),
        mismatches.join("\n")
    );
}
