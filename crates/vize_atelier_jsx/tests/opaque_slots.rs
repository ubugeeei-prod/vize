//! Slot objects Vize cannot expand (#9075): complete compiled modules, frozen as
//! a differential corpus entry and mounted beside `@vue/babel-plugin-jsx`
//! output (`tests/tooling/support/jsx-opaque-slots-runtime.ts`).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "integration fixtures execute actual compiler output through Node"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, compile_jsx};
use vize_l0::Allocator;

const FIXTURES: [(&str, JsxLang, &str); 4] = [
    (
        "reference",
        JsxLang::Jsx,
        include_str!("../../../tests/_fixtures/differential/jsx/opaque-slots-reference.jsx"),
    ),
    (
        "child",
        JsxLang::Jsx,
        include_str!("../../../tests/_fixtures/differential/jsx/opaque-slots-child.jsx"),
    ),
    (
        "spread",
        JsxLang::Jsx,
        include_str!("../../../tests/_fixtures/differential/jsx/opaque-slots-spread.jsx"),
    ),
    (
        "argument",
        JsxLang::Tsx,
        include_str!("../../../tests/_fixtures/differential/jsx/opaque-slots-argument.tsx"),
    ),
];

const FROZEN: &str = "tests/_fixtures/differential/jsx/opaque-slots.fixed-legacy.json";

fn compile_fixtures() -> Value {
    let mut fixtures = serde_json::Map::new();
    for (id, lang, source) in FIXTURES {
        let allocator = Allocator::new();
        let output = compile_jsx(&allocator, source, lang, &JsxCompileConfig::default());
        fixtures.insert(
            id.into(),
            json!({
                "source": source,
                "lang": if lang == JsxLang::Tsx { "tsx" } else { "jsx" },
                "code": output.module_code(),
                "diagnostics": output.diagnostics.iter().map(|d| d.message.as_str()).collect::<Vec<_>>(),
            }),
        );
    }
    Value::Object(fixtures)
}

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn fixed_legacy_output_matches_the_frozen_corpus() {
    let actual = compile_fixtures();
    for (id, fixture) in actual.as_object().expect("fixtures") {
        assert_eq!(fixture["diagnostics"], json!([]), "{id}");
    }
    let path = root().join(FROZEN);
    if std::env::var_os("VIZE_UPDATE_JSX_OPAQUE_SLOTS").is_some() {
        let text = serde_json::to_string_pretty(&actual).expect("json");
        std::fs::write(&path, format!("{text}\n")).expect("write frozen corpus");
    }
    let frozen: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("frozen corpus"))
            .expect("frozen corpus JSON");
    assert_eq!(actual, frozen);
}

#[test]
fn opaque_slot_modules_render_like_babel() {
    let mut child = Command::new("node")
        .arg(root().join("tests/tooling/support/jsx-opaque-slots-runtime.ts"))
        .current_dir(root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("node with the installed Vue/Babel dependencies");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(&serde_json::to_vec(&compile_fixtures()).expect("payload"))
        .expect("send compiler output");
    let output = child.wait_with_output().expect("runtime process");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("runtime receipt");
    assert_eq!(
        receipt["rendered"].as_object().expect("cases").len(),
        FIXTURES.len()
    );
}
