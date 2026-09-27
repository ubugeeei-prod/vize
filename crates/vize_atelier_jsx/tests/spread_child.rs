//! Spread children `{...items}` (#6888): complete compiled modules, frozen as
//! a differential corpus entry and executed against real Vue and
//! `@vue/babel-plugin-jsx` (`tests/tooling/support/jsx-spread-child-runtime.mjs`).
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
use vize_atelier_jsx::{
    JsxCompileConfig, JsxLang, SsrCompileOptions, VaporCompileOptions, compile_jsx, compile_to_ssr,
    compile_to_vapor,
};
use vize_l0::Allocator;

const FIXTURES: [(&str, JsxLang, &str); 7] = [
    (
        "values",
        JsxLang::Jsx,
        include_str!("../../../tests/_fixtures/differential/jsx/spread-child-values.jsx"),
    ),
    (
        "nested",
        JsxLang::Jsx,
        include_str!("../../../tests/_fixtures/differential/jsx/spread-child-nested.jsx"),
    ),
    (
        "fragment",
        JsxLang::Jsx,
        include_str!("../../../tests/_fixtures/differential/jsx/spread-child-fragment.jsx"),
    ),
    (
        "slots",
        JsxLang::Jsx,
        include_str!("../../../tests/_fixtures/differential/jsx/spread-child-slots.jsx"),
    ),
    (
        "conditional",
        JsxLang::Jsx,
        include_str!("../../../tests/_fixtures/differential/jsx/spread-child-conditional.jsx"),
    ),
    (
        "loop",
        JsxLang::Jsx,
        include_str!("../../../tests/_fixtures/differential/jsx/spread-child-loop.jsx"),
    ),
    (
        "cast",
        JsxLang::Tsx,
        include_str!("../../../tests/_fixtures/differential/jsx/spread-child-cast.tsx"),
    ),
];

const FROZEN: &str = "tests/_fixtures/differential/jsx/spread-child.fixed-legacy.json";

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
    let path = root().join(FROZEN);
    if std::env::var_os("VIZE_UPDATE_JSX_SPREAD").is_some() {
        let text = serde_json::to_string_pretty(&actual).expect("json");
        std::fs::write(&path, format!("{text}\n")).expect("write frozen corpus");
    }
    let frozen: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("frozen corpus"))
            .expect("frozen corpus JSON");
    assert_eq!(actual, frozen);
}

#[test]
fn spread_modules_render_update_and_hydrate_like_babel() {
    let mut child = Command::new("node")
        .arg(root().join("tests/tooling/support/jsx-spread-child-runtime.mjs"))
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
    assert_eq!(receipt["failures"], 0);
    assert_eq!(
        receipt["observations"].as_object().expect("cases").len(),
        FIXTURES.len()
    );
}

#[test]
fn vapor_and_ssr_report_the_spread_instead_of_emitting_client_blocks() {
    let source = "const A = () => <div>{...items}</div>;";
    let message = "spread children (`{...items}`) are only supported in VDOM output; Vapor and SSR stringify the value instead of spreading it";

    let allocator = Allocator::new();
    let vapor = compile_to_vapor(
        &allocator,
        source,
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
    let messages: Vec<_> = vapor
        .diagnostics
        .iter()
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(messages, [message]);
    assert!(!vapor.components[0].code.contains("BAIL"));

    let allocator = Allocator::new();
    let ssr = compile_to_ssr(
        &allocator,
        source,
        JsxLang::Jsx,
        SsrCompileOptions::default(),
    );
    let messages: Vec<_> = ssr.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert_eq!(messages, [message]);
    assert!(ssr.components[0].code.contains("_ssrInterpolate(items)"));
}

#[test]
fn jsx_inside_the_spread_argument_is_reported() {
    let allocator = Allocator::new();
    let output = compile_jsx(
        &allocator,
        "const A = () => <div>{...[<i/>]}</div>;",
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let messages: Vec<_> = output
        .diagnostics
        .iter()
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(
        messages,
        [
            "JSX inside a spread child argument (`{...[<i/>]}`) is not supported; declare the VNodes separately and spread the variable"
        ]
    );
}
