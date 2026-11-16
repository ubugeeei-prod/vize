use super::{VALIDATION, vite_runtime};
use sha2::{Digest, Sha256};

const CONTROLS: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/runtime-controls.original.json"
);

fn digest(bytes: &[u8]) -> vize_l0::String {
    let mut hex = vize_l0::String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        hex.push_str(crate::cstr!("{byte:02x}").as_str());
    }
    hex
}

#[test]
fn complete_product_sources_and_native_assembly_keep_their_exact_postimages() {
    assert_eq!(
        digest(VALIDATION.as_bytes()),
        "e7cd857464b449579d288fd20e75549875404389200fa40a1bd9517ec4e2ab83"
    );
    assert_eq!(
        digest(super::super::VITE_RUNTIME.as_bytes()),
        "7f8d8264168123d70c8429a998b0c3e9931ba7abf4cf28b2677293e028816f45"
    );
    assert_eq!(
        digest(vite_runtime().unwrap().as_bytes()),
        "c5aef161453268dd936d2b49fbb7ce5134861bff941a1bcec76e8bfcb0a3659f"
    );
}

#[test]
fn private_native_module_executes_the_same_sixteen_complete_original_controls() {
    let controls: serde_json::Value = serde_json::from_str(CONTROLS).unwrap();
    assert_eq!(controls["rows"].as_array().unwrap().len(), 16);
    let mut script = vite_runtime().unwrap();
    script.push_str("\nconst controls = ");
    script.push_str(CONTROLS);
    script.push_str(r#";
const rows = [];
for (const row of controls.rows) {
  let outcome;
  try { outcome = {kind: "returned", value: await projectViteConfigSource(structuredClone(row.input))}; }
  catch (error) { outcome = {kind: "threw", error: {name: error.name, message: error.message}}; }
  rows.push({...row, outcome});
}

process.stdout.write(JSON.stringify(rows));
"#);
    let project = tempfile::tempdir().unwrap();
    let runtime = super::super::runtime_file::NodeRuntimeFile::new(&script).unwrap();
    let output = runtime
        .command(&project.path().join("vite.config.mjs"))
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stderr, Vec::<u8>::new());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        controls["rows"]
    );
}

#[test]
fn owned_modules_share_one_scope_and_the_plugin_host_loads_lazily() {
    let assembled = vite_runtime().unwrap();
    assert_eq!(assembled.matches(" from \"./").count(), 0);
    assert_eq!(assembled.matches("import(\"./").count(), 0);
    assert_eq!(
        assembled
            .matches("import(\"data:text/javascript;base64,")
            .count(),
        1
    );
    assert_eq!(
        super::assemble("import { x } from \"./unowned.js\";\nexport const y = x;\n"),
        Err("unowned relative import in the product runtime")
    );
    assert_eq!(
        super::assemble("import { x } from \"./vite-runtime-validation.js\"\n"),
        Err("unterminated import in the product runtime")
    );
    let builtin = "import { resolve } from \"node:path\";\nexport { resolve };\n";
    assert_eq!(
        super::assemble(builtin).unwrap(),
        crate::cstr!("{}{builtin}", super::assemble("").unwrap())
    );
    let mut encoded = vize_l0::String::default();
    super::base64(b"vize\xff\x00", &mut encoded);
    assert_eq!(encoded, "dml6Zf8A");
}

#[test]
fn the_lazy_plugin_host_resolves_and_inspects_inside_the_private_module() {
    let mut script = vite_runtime().unwrap();
    script.push_str(
        r#"
const rows = [];
try { await projectViteConfigSource({ vue: { plugins: [{ name: "x", version: "1" }] } }); }
catch (error) { rows.push(error.message); }
rows.push(JSON.stringify(await projectViteConfigSource({ vue: { plugins: [false, null] } })));
const inspection = await inspectViteConfigPlugins({ vue: {} });
rows.push(inspection.order.length, inspection.digest.length, inspection.admission.lint.status);
process.stdout.write(JSON.stringify(rows));
"#,
    );
    let project = tempfile::tempdir().unwrap();
    let runtime = super::super::runtime_file::NodeRuntimeFile::new(&script).unwrap();
    let output = runtime
        .command(&project.path().join("vite.config.mjs"))
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::json!([
            "[plugin/entry-invalid] vue.plugins[0]: expected a defineVizePlugin entry or a v1 definePlugin rule plugin; other v1 hooks keep their native APIs",
            "{\"typeChecker\":{\"jsxTypecheck\":true}}",
            0,
            64,
            "unused"
        ])
    );
}
