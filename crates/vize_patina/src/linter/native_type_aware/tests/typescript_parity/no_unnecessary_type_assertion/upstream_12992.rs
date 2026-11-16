//! The typescript-eslint v8.72.2 cases #12992 lists where the rule disagreed:
//! numeric enum assertions change the type even though `number` and the enum
//! are mutually assignable, an overloaded receiver whose parameter is the same
//! in every overload still accepts the operand, and an assertion that
//! contextually types a function keeps the `any` its parameters infer.
use super::{CONTEXTUAL, UNCHANGED, lint, project, reports};
use crate::linter::native_type_aware::tests::{corsa_available, failure_context};
use serde::Deserialize;
use serde_json::Value;

const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unnecessary-type-assertion-12992/cases.json"
));
/// The same shapes where the assertion really is a no-op.
const CONTROLS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unnecessary-type-assertion-12992/Controls.ts.txt"
));

/// The compiler options #12992 ran every case with.
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
    message_ids: Vec<String>,
}

fn message(id: &str) -> &'static str {
    match id {
        "contextuallyUnnecessary" => CONTEXTUAL,
        _ => UNCHANGED,
    }
}

#[test]
fn typescript_eslint_872_cases_from_12992_match() {
    if !corsa_available() {
        return;
    }
    let corpus: Value = serde_json::from_str(CORPUS).expect("pinned #12992 oracle");
    assert_eq!(corpus["schema"], "vize.typescript-eslint.immutable-oracle");
    assert_eq!(corpus["issue"], 12992);
    assert_eq!(corpus["upstream"]["release"], "v8.72.2");
    let mut cases: Vec<Case> =
        serde_json::from_value(corpus["cases"].clone()).expect("pinned #12992 cases");
    assert_eq!(cases.len(), 14);
    cases.push(Case {
        id: "Controls.ts".into(),
        code: CONTROLS.into(),
        message_ids: vec!["unnecessaryAssertion".into(); 2],
    });

    let project = project("no-unnecessary-assertion-12992-", TSCONFIG, &[]);
    let mut mismatches = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let source = format!(
            "<script lang=\"ts\">\n{}\nexport {{}};\n</script>\n",
            case.code
        );
        let result = lint(&project, &format!("Case{index:02}.vue"), &source);
        let messages = reports(&result)
            .into_iter()
            .map(|(_, _, message)| message)
            .collect::<Vec<_>>();
        let wanted = case
            .message_ids
            .iter()
            .map(|id| message(id))
            .collect::<Vec<_>>();
        if messages != wanted {
            mismatches.push(format!("{}: {}", case.id, failure_context(&result)));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n\n"));
}
