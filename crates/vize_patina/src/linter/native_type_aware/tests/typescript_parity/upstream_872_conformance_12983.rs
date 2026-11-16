//! typescript-eslint v8.72.2 cases reported in #12983-#12987, preserved as
//! data with the lines upstream reports them on.
use super::super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/typescript-eslint-872-conformance-12983/cases.json"
));

/// The compiler options the reports ran with.
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
    rule: String,
    block: String,
    code: String,
    expected_lines: Vec<usize>,
    /// Why vize cannot decide the case yet; it must stay silent until then.
    #[serde(default)]
    vize_gap: Option<String>,
}

/// 1-based line of `offset` in `source`.
fn line_of(source: &str, offset: u32) -> usize {
    let offset = usize::try_from(offset).expect("diagnostic offset");
    source
        .bytes()
        .take(offset)
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

#[test]
fn reported_typescript_eslint_872_cases_match() {
    if !corsa_available() {
        return;
    }

    let corpus: Value = serde_json::from_str(CORPUS).expect("upstream corpus JSON");
    assert_eq!(corpus["schema"], "vize.typescript-eslint.selected-corpus");
    assert_eq!(corpus["version"], 1);
    assert_eq!(corpus["upstream"]["release"], "v8.72.2");
    assert_eq!(
        corpus["upstream"]["commit"],
        "12e452f0292212bf9bf443b3988ec230f4ae7097"
    );
    let cases: Vec<Case> = serde_json::from_value(corpus["cases"].clone()).expect("cases");
    assert_eq!(corpus["caseCount"], cases.len());
    assert_eq!(cases.len(), 32);

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("typescript-eslint-8-72-2-12983-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(project.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");

    let mut linters = BTreeMap::new();
    let mut mismatches = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        // The case's first line is the SFC's second.
        let source = match case.block.as_str() {
            "script" => format!(
                "<script lang=\"ts\">\n{}\nexport {{}};\n</script>\n",
                case.code
            ),
            "setup" => format!("<script setup lang=\"ts\">\n{}\n</script>\n", case.code),
            other => panic!("unknown block {other}"),
        };
        let path = source_dir.join(format!("case-{index:02}.vue"));
        std::fs::write(&path, &source).expect("case source");
        let linter = linters.entry(case.rule.clone()).or_insert_with(|| {
            Linter::with_preset(LintPreset::Opinionated)
                .with_enabled_rules(Some(vec![case.rule.clone().into()]))
        });
        let result =
            lint_sfc_with_corsa(linter, &source, path.to_str().expect("UTF-8 fixture path"));
        let reported = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == case.rule)
            .collect::<Vec<_>>();
        let actual = reported
            .iter()
            .map(|diagnostic| line_of(&source, diagnostic.start) - 1)
            .collect::<Vec<_>>();
        // A documented gap pins today's silence, so closing it updates the corpus.
        let expected = if case.vize_gap.is_some() {
            Vec::new()
        } else {
            case.expected_lines.clone()
        };
        if actual != expected {
            let findings = reported
                .iter()
                .map(|diagnostic| {
                    format!(
                        "{}..{} {:?}: {}",
                        diagnostic.start,
                        diagnostic.end,
                        source.get(diagnostic.start as usize..diagnostic.end as usize),
                        diagnostic.message
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
            mismatches.push(format!(
                "{} expected lines {expected:?} (upstream {:?}), got {actual:?}: {findings}",
                case.id, case.expected_lines
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "{}/{} reported upstream cases disagree:\n{}",
        mismatches.len(),
        cases.len(),
        mismatches.join("\n")
    );
}

/// The SFC from the #12985 report: the alias declared in `<script setup>` is
/// reported where it is read, and the constant after it is not.
const ALIAS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/typescript-eslint-872-conformance-12983/Alias.vue.txt"
));

#[test]
fn deprecated_setup_alias_keeps_its_jsdoc() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec!["type/no-deprecated".into()]));
    let result = lint_sfc_with_corsa(&linter, ALIAS, "Alias.vue");
    let reported = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.message.as_str(),
                line_of(ALIAS, diagnostic.start),
                ALIAS.get(diagnostic.start as usize..diagnostic.end as usize),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        reported,
        [(
            "type/no-deprecated",
            "`Old` is deprecated. use Next",
            5,
            Some("Old")
        )]
    );
}
