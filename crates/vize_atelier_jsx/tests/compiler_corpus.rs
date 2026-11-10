//! Runner for the compiler corpus JSX/TSX entry cases (`jsxEntryCases` in
//! `tests/_fixtures/differential/compiler/manifest.json`, contract in
//! `tests/differential/compiler-jsx-entry.ts`). Each pinned entry is compiled per
//! registered target and its product facets must equal the pinned reference bytes.
//! `VIZE_JSX_ENTRY_CAPTURE=<dir>` writes every observation as
//! `<dir>/<root>/modules.expected.json`; a capture never accepts a reference.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "the corpus runner fails loudly on any unregistered or unpinned input"
)]

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fmt::Write,
    path::{Path, PathBuf},
};
use vize_atelier_jsx::{JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx};
use vize_l0::{Allocator, String};

const ADAPTER: &str = "compiler-jsx-entry-source-v1";

/// The facets the `compileJsx` product binding returns, in the reference's field order.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Module<'a> {
    target: &'a str,
    code: String,
    errors: Vec<String>,
    warnings: Vec<String>,
    scoped_styles: Vec<ScopedStyle>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScopedStyle {
    scope_id: String,
    css: String,
}

fn config(target: &str, compat: JsxCompatMode) -> JsxCompileConfig {
    let (default_mode, ssr) = match target {
        "vdom" => (JsxOutputMode::Vdom, false),
        "vapor" => (JsxOutputMode::Vapor, false),
        "vdom-ssr" => (JsxOutputMode::Vdom, true),
        "vapor-ssr" => (JsxOutputMode::Vapor, true),
        _ => panic!("unregistered JSX entry target: {target}"),
    };
    JsxCompileConfig {
        default_mode,
        compat,
        ssr,
        ..Default::default()
    }
}

fn observe<'a>(source: &str, lang: JsxLang, compat: JsxCompatMode, target: &'a str) -> Module<'a> {
    let allocator = Allocator::new();
    let output = compile_jsx(&allocator, source, lang, &config(target, compat));
    let messages = |error: bool| -> Vec<String> {
        output
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_error() == error)
            .map(|diagnostic| diagnostic.message.clone())
            .collect()
    };
    Module {
        target,
        code: output.module_code(),
        errors: messages(true),
        warnings: messages(false),
        scoped_styles: output
            .components
            .iter()
            .filter_map(|component| component.scoped_style())
            .map(|style| ScopedStyle {
                scope_id: style.scope_id.clone(),
                css: style.css.clone(),
            })
            .collect(),
    }
}

fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string field {field}"))
}

fn pinned(root: &Path, path: &str, pin: &Value) -> Vec<u8> {
    let bytes = std::fs::read(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"));
    let mut digest = String::default();
    for byte in Sha256::digest(&bytes) {
        write!(digest, "{byte:02x}").expect("hex digest");
    }
    assert_eq!(digest, text(pin, "sha256"), "SHA256 mismatch: {path}");
    bytes
}

#[test]
fn jsx_entry_cases_match_their_pinned_modules() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/differential/compiler");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).expect("manifest"))
            .expect("JSON compiler manifest");
    let cases = manifest["jsxEntryCases"]
        .as_array()
        .expect("jsxEntryCases is an array");
    let capture = std::env::var_os("VIZE_JSX_ENTRY_CAPTURE").map(PathBuf::from);
    let mut drifted = Vec::new();
    for case in cases {
        let id = text(case, "id");
        assert_eq!(text(case, "adapter"), ADAPTER, "{id}");
        let compat = JsxCompatMode::from_config_str(text(case, "compat"))
            .unwrap_or_else(|| panic!("{id}: unregistered compat"));
        let input_root = text(&case["inputs"], "root");
        let entry = &case["inputs"]["files"][0];
        let entry_path = Path::new(input_root).join(text(entry, "path"));
        let entry_path = entry_path.to_str().expect("UTF-8 entry path");
        let bytes = pinned(&root, entry_path, entry);
        let source = std::str::from_utf8(&bytes).expect("UTF-8 entry source");
        let lang = JsxLang::from_path(entry_path.trim_end_matches(".txt"));
        let modules: Vec<Module<'_>> = case["targets"]
            .as_array()
            .expect("targets")
            .iter()
            .map(|target| observe(source, lang, compat, target.as_str().expect("target")))
            .collect();
        let mut observed = serde_json::to_vec_pretty(&modules).expect("modules JSON");
        observed.push(b'\n');
        if let Some(capture) = &capture {
            let directory = capture.join(input_root);
            std::fs::create_dir_all(&directory).expect("capture directory");
            std::fs::write(directory.join("modules.expected.json"), &observed).expect("capture");
        }
        let reference = &case["reference"];
        if pinned(&root, text(reference, "path"), reference) != observed {
            drifted.push(id);
        }
    }
    assert!(
        drifted.is_empty(),
        "JSX entry modules differ from their pinned references (capture with VIZE_JSX_ENTRY_CAPTURE): {drifted:?}"
    );
}
