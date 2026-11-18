//! Every non-JSX case of typescript-eslint v8.72.2's
//! `no-unsafe-enum-assignment.test.ts` (#12994), with the exact packets
//! upstream reports. The cases left are tracked in #13070.
use super::super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;

const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/typescript-eslint-872-enum-assignment-12994/cases.json"
));

/// The compiler options the report ran with.
const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ESNext", "DOM", "DOM.Iterable"],
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "experimentalDecorators": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
"#;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    code: String,
    expected: Vec<Packet>,
    /// Why vize cannot decide the case yet; it must stay silent until then.
    #[serde(default)]
    vize_gap: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct Packet {
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
    message: String,
}

/// 1-based line and UTF-16 column of `offset` in `code`, which starts on the
/// SFC's second line.
fn position(source: &str, offset: u32) -> (usize, usize) {
    let before = source
        .get(..usize::try_from(offset).expect("diagnostic offset"))
        .expect("diagnostic offset on a character boundary");
    let line = before.matches('\n').count();
    let column = before
        .rsplit('\n')
        .next()
        .unwrap_or_default()
        .encode_utf16()
        .count()
        + 1;
    (line, column)
}

#[test]
fn every_upstream_case_reports_upstream_packets() {
    if !corsa_available() {
        return;
    }
    let corpus: Value = serde_json::from_str(CORPUS).expect("upstream corpus JSON");
    assert_eq!(corpus["schema"], "vize.typescript-eslint.rule-corpus");
    assert_eq!(corpus["version"], 1);
    assert_eq!(corpus["rule"], "type/no-unsafe-enum-assignment");
    assert_eq!(corpus["upstream"]["release"], "v8.72.2");
    let cases: Vec<Case> = serde_json::from_value(corpus["cases"].clone()).expect("cases");
    assert_eq!(corpus["caseCount"], cases.len());
    assert_eq!(cases.len(), 247);
    assert_eq!(
        cases.iter().filter(|case| case.vize_gap.is_some()).count(),
        7
    );

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("typescript-eslint-8-72-2-12994-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(project.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");

    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec!["type/no-unsafe-enum-assignment".into()]));
    let mut mismatches = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let source = format!(
            "<script lang=\"ts\">\n{}\nexport {{}};\n</script>\n",
            case.code
        );
        let path = source_dir.join(format!("case-{index:03}.vue"));
        std::fs::write(&path, &source).expect("case source");
        let result =
            lint_sfc_with_corsa(&linter, &source, path.to_str().expect("UTF-8 fixture path"));
        let mut actual = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "type/no-unsafe-enum-assignment")
            .map(|diagnostic| {
                let (line, column) = position(&source, diagnostic.start);
                let (end_line, end_column) = position(&source, diagnostic.end);
                Packet {
                    line,
                    column,
                    end_line,
                    end_column,
                    message: diagnostic.message.to_string(),
                }
            })
            .collect::<Vec<_>>();
        actual.sort();
        // A documented gap pins today's silence, so closing it updates the corpus.
        let mut expected = if case.vize_gap.is_some() {
            Vec::new()
        } else {
            case.expected.clone()
        };
        expected.sort();
        if actual != expected {
            mismatches.push(format!(
                "{}: expected {expected:?}, got {actual:?}",
                case.id
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{}/{} upstream cases disagree:\n{}",
        mismatches.len(),
        cases.len(),
        mismatches.join("\n")
    );
}
