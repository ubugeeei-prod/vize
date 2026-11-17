//! Immutable typescript-eslint v8.72.2 oracle for the #12988, #12989, #12990,
//! #12991 and #12993 conformance reports: every listed case, linted in the
//! project shape the reporter used, must report exactly the upstream lines.
//! A case blocked on another issue must keep reporting what it does today, so
//! the oracle notices when that issue lands.
use super::super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/typescript-eslint-conformance-b-12988/cases.json"
));

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
struct Expected {
    line: u32,
    message: Option<String>,
}

#[derive(Deserialize)]
struct Blocked {
    issue: u32,
    vize: Vec<Expected>,
}

#[derive(Deserialize)]
struct Case {
    issue: u32,
    rule: String,
    id: String,
    mode: String,
    code: String,
    expected: Vec<Expected>,
    blocked: Option<Blocked>,
}

fn line_of(source: &str, offset: u32) -> u32 {
    let prefix = source.get(..offset as usize).unwrap_or(source);
    // The opening `<script>` tag is line 0, so the case's first line is 1.
    prefix.matches('\n').count() as u32
}

#[test]
fn listed_conformance_cases_match_the_immutable_upstream_oracle() {
    if !corsa_available() {
        return;
    }
    let corpus: Value = serde_json::from_str(CORPUS).expect("oracle JSON");
    assert_eq!(corpus["schema"], "vize.typescript-eslint.immutable-oracle");
    assert_eq!(corpus["version"], 1);
    assert_eq!(corpus["upstream"]["release"], "v8.72.2");
    assert_eq!(
        corpus["upstream"]["commit"],
        "12e452f0292212bf9bf443b3988ec230f4ae7097"
    );
    let cases: Vec<Case> = serde_json::from_value(corpus["cases"].clone()).expect("oracle cases");
    assert_eq!(corpus["caseCount"], cases.len());
    assert_eq!(cases.len(), 44);
    let blocked = cases
        .iter()
        .filter_map(|case| case.blocked.as_ref())
        .map(|blocked| blocked.issue)
        .collect::<Vec<_>>();
    assert_eq!(blocked, [13002]);

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("typescript-eslint-conformance-b-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(project.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");

    // Every plain-script case that `<script setup>` can hold (no `export`)
    // must report the same lines there too.
    let runs = cases.iter().flat_map(|case| {
        let setup_too = case.mode == "script" && !case.code.contains("export");
        let extra = setup_too.then_some((case, "setup"));
        std::iter::once((case, case.mode.as_str())).chain(extra)
    });
    let mut linters = BTreeMap::new();
    let mut mismatches = Vec::new();
    let mut count = 0;
    for (index, (case, mode)) in runs.enumerate() {
        count += 1;
        let source = match mode {
            "script" => format!(
                "<script lang=\"ts\">\n{}\nexport {{}};\n</script>\n",
                case.code
            ),
            "setup" => format!("<script setup lang=\"ts\">\n{}\n</script>\n", case.code),
            other => panic!("{}: unknown mode {other}", case.id),
        };
        let path = source_dir.join(format!("case-{index:02}.vue"));
        std::fs::write(&path, &source).expect("case source");
        let linter = linters.entry(case.rule.clone()).or_insert_with(|| {
            Linter::with_preset(LintPreset::Opinionated)
                .with_enabled_rules(Some(vec![case.rule.clone().into()]))
        });
        let result =
            lint_sfc_with_corsa(linter, &source, path.to_str().expect("UTF-8 fixture path"));
        let mut actual = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == case.rule)
            .map(|diagnostic| {
                (
                    line_of(&source, diagnostic.start),
                    diagnostic.message.to_string(),
                )
            })
            .collect::<Vec<_>>();
        actual.sort();
        let mut expected = case
            .blocked
            .as_ref()
            .map_or(&case.expected, |blocked| &blocked.vize)
            .iter()
            .map(|expected| (expected.line, expected.message.clone()))
            .collect::<Vec<_>>();
        expected.sort();
        let matches = actual.len() == expected.len()
            && actual.iter().zip(&expected).all(|((line, message), want)| {
                *line == want.0 && want.1.as_ref().is_none_or(|want| want == message)
            });
        if !matches {
            mismatches.push(format!(
                "#{} {} ({mode}): expected {:?}, got {:?}",
                case.issue, case.id, expected, actual
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "{}/{} upstream cases disagree:\n{}",
        mismatches.len(),
        count,
        mismatches.join("\n")
    );
}
