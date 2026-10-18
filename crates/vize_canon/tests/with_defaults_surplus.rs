//! #9499: type-check complete native mapper modules with the declared TS host.
#![cfg(feature = "native")]

use std::{path::Path, process::Command};

use serde_json::{Value, json};
use vize_canon::batch::generate_vue_content_mapper_transform;
use vize_l0::{ToCompactString, cstr};

const CASES: &str =
    include_str!("../../../tests/_fixtures/content-mapper/with-defaults-surplus.json");
const HOST: &str = r#"
const fs = require('node:fs');
const path = require('node:path');
const { createRequire } = require('node:module');
const fromTests = createRequire(path.join(process.argv[1], 'tests/package.json'));
const ts = fromTests('typescript');
const cases = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const options = { strict: true, skipLibCheck: true, noEmit: true, types: [],
  target: ts.ScriptTarget.ESNext, jsx: ts.JsxEmit.Preserve,
  module: ts.ModuleKind.ESNext, moduleResolution: ts.ModuleResolutionKind.Bundler };
const rows = cases.map(({ name, directory, files }) => {
  const program = ts.createProgram(files.map(file => path.join(directory, file)), options);
  const diagnostics = ts.getPreEmitDiagnostics(program).map(d => ({
    code: d.code, category: d.category, start: d.start, length: d.length,
    file: d.file && path.relative(directory, d.file.fileName),
    message: ts.flattenDiagnosticMessageText(d.messageText, '\n')
  }));
  return { name, diagnostics };
});
process.stdout.write(JSON.stringify({ typescript: ts.version, rows }));
"#;

#[test]
fn complete_native_mapper_modules_accept_surplus_defaults_and_keep_declared_type_errors() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "pr".into());
    let evidence = workspace
        .join("target/nextest")
        .join(profile)
        .join("with-defaults-surplus");
    std::fs::create_dir_all(&evidence).unwrap();
    let project = tempfile::tempdir().unwrap();
    let fixture: Value = serde_json::from_str(CASES).unwrap();
    let mut requests = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let directory = project.path().join(name);
        let capture = evidence.join(name);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::create_dir_all(&capture).unwrap();
        let dependency_link = directory.join("node_modules");
        if !dependency_link.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(workspace.join("tests/node_modules"), &dependency_link)
                .unwrap();
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(
                workspace.join("tests/node_modules"),
                &dependency_link,
            )
            .unwrap();
        }
        let mut files = Vec::new();
        for (name, source) in case["files"].as_object().unwrap() {
            let source = source.as_str().unwrap();
            std::fs::write(directory.join(name), source).unwrap();
            std::fs::write(capture.join(name), source).unwrap();
            if name.ends_with(".vue") {
                let transform = generate_vue_content_mapper_transform(Path::new(name), source)
                    .expect("complete actual native mapper transform");
                std::fs::write(
                    capture.join("transform.json"),
                    serde_json::to_vec_pretty(&transform).unwrap(),
                )
                .unwrap();
                assert_eq!(transform.diagnostics, vec![], "{name}: {transform:#?}");
                let generated = cstr!("{name}{}", transform.extension);
                std::fs::write(directory.join(&generated), transform.text.as_str()).unwrap();
                std::fs::write(capture.join(&generated), transform.text.as_str()).unwrap();
                files.push(generated);
            } else if name.ends_with(".ts") {
                files.push(name.to_compact_string());
            }
        }
        requests.push(json!({ "name": name, "directory": directory, "files": files }));
    }
    let request = evidence.join("requests.json");
    std::fs::write(&request, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    let output = Command::new("node")
        .arg("-e")
        .arg(HOST)
        .arg(workspace)
        .arg(&request)
        .output()
        .expect("run the already declared official TypeScript dependency");
    // Keep every complete module, mapping vector and host stream before judging.
    std::fs::write(evidence.join("stdout.bin"), &output.stdout).unwrap();
    std::fs::write(evidence.join("stderr.bin"), &output.stderr).unwrap();
    std::fs::write(
        evidence.join("process.json"),
        serde_json::to_vec_pretty(&json!({
            "sourceSha": std::env::var("GITHUB_SHA").ok(),
            "host": HOST, "requests": requests,
            "exitCode": output.status.code(), "success": output.status.success(),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stderr, b"", "{output:?}");
    let observed: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(observed["typescript"], "6.0.3");
    let rows = observed["rows"].as_array().unwrap();
    assert_eq!(rows.len(), fixture["cases"].as_array().unwrap().len());
    for (actual, expected) in rows.iter().zip(fixture["cases"].as_array().unwrap()) {
        assert_eq!(actual["name"], expected["name"]);
        let diagnostics = actual["diagnostics"].as_array().unwrap();
        let codes = diagnostics
            .iter()
            .map(|d| d["code"].clone())
            .collect::<Vec<_>>();
        assert_eq!(json!(codes), expected["codes"], "{actual:#}");
        for diagnostic in diagnostics {
            assert_eq!(diagnostic["file"], "App.vue.tsx", "{actual:#}");
            assert_eq!(diagnostic["category"], 1, "{actual:#}");
            let generated = std::fs::read_to_string(
                evidence
                    .join(expected["name"].as_str().unwrap())
                    .join("App.vue.tsx"),
            )
            .unwrap();
            let start = diagnostic["start"].as_u64().unwrap() as usize;
            let end = start + diagnostic["length"].as_u64().unwrap() as usize;
            assert_eq!(generated.get(start..end), Some("size"), "{actual:#}");
        }
    }
}
