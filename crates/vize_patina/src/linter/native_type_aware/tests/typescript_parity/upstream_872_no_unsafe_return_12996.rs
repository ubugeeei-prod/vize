//! Immutable typescript-eslint v8.72.2 oracle for the three #12996 packets.
use super::super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintDiagnostic, LintPreset, Linter};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

const RULE: &str = "type/no-unsafe-return";
const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unsafe-return-composite-12996/cases.json"
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
#[serde(rename_all = "camelCase")]
struct Expected {
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
    message: String,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    code: String,
    expected: Expected,
}

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]))
}

fn complete_packet(diagnostic: &LintDiagnostic) -> Value {
    json!({
        "ruleName": diagnostic.rule_name,
        "severity": diagnostic.severity,
        "message": diagnostic.message,
        "start": diagnostic.start,
        "end": diagnostic.end,
        "help": diagnostic.help,
        "labels": diagnostic.labels.iter().map(|label| json!({
            "message": label.message,
            "start": label.start,
            "end": label.end,
        })).collect::<Vec<_>>(),
        "fix": diagnostic.fix.as_ref().map(|fix| json!({
            "message": fix.message,
            "edits": fix.edits,
        })),
    })
}

fn byte_offset(source: &str, line: u32, column: u32) -> u32 {
    let mut offset = 0usize;
    for _ in 1..line {
        let newline = source[offset..].find('\n').expect("oracle line");
        offset += newline + 1;
    }
    let column = usize::try_from(column.saturating_sub(1)).expect("oracle column");
    u32::try_from(offset + column).expect("oracle byte offset")
}

#[test]
fn reported_packets_match_the_immutable_upstream_oracle() {
    if !corsa_available() {
        return;
    }
    let corpus: Value = serde_json::from_str(CORPUS).expect("oracle JSON");
    assert_eq!(corpus["schema"], "vize.typescript-eslint.immutable-oracle");
    assert_eq!(corpus["version"], 1);
    assert_eq!(corpus["issue"], 12996);
    assert_eq!(corpus["upstream"]["release"], "v8.72.2");
    assert_eq!(
        corpus["upstream"]["commit"],
        "12e452f0292212bf9bf443b3988ec230f4ae7097"
    );
    let cases: Vec<Case> = serde_json::from_value(corpus["cases"].clone()).expect("oracle cases");
    assert_eq!(cases.len(), 3);

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("no-unsafe-return-12996-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(project.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");

    for (index, case) in cases.iter().enumerate() {
        let source = format!(
            "<script lang=\"ts\">\n{}\nexport {{}};\n</script>\n",
            case.code
        );
        let path = source_dir.join(format!("case-{index}.vue"));
        std::fs::write(&path, &source).expect("case source");
        let result = lint_sfc_with_corsa(
            &linter(),
            &source,
            path.to_str().expect("UTF-8 fixture path"),
        );
        let actual = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(complete_packet)
            .collect::<Vec<_>>();
        let start = byte_offset(&source, case.expected.line + 1, case.expected.column);
        let end = byte_offset(
            &source,
            case.expected.end_line + 1,
            case.expected.end_column,
        );
        let expected = vec![json!({
            "ruleName": RULE,
            "severity": "warning",
            "message": case.expected.message,
            "start": start,
            "end": end,
            "help": null,
            "labels": [],
            "fix": null,
        })];
        assert_eq!(actual, expected, "{}", case.id);
    }
}

#[test]
fn promise_any_extension_preserves_adjacent_return_behavior() {
    if !corsa_available() {
        return;
    }
    let source = r#"<script lang="ts">
function syncPromiseAny() {
  return {} as Promise<any>;
}
async function safeUnion() {
  return {} as Promise<object> | Promise<unknown>;
}
interface SafeAlias extends Promise<object> {
  brand: "safe";
}
declare const safeAlias: SafeAlias;
async function safeInherited() {
  return safeAlias;
}
declare const directAny: any;
function stillAny() {
  return directAny;
}
declare const anyArray: any[];
function stillAnyArray() {
  return anyArray;
}
export {};
</script>
"#;
    let result = lint_sfc_with_corsa(&linter(), source, "Controls.vue");
    let actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            (
                diagnostic.message.as_str(),
                &source[diagnostic.start as usize..diagnostic.end as usize],
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            (
                "Unsafe return of a value of type `any`.",
                "return directAny;"
            ),
            (
                "Unsafe return of a value of type `any[]`.",
                "return anyArray;"
            ),
        ]
    );
}
