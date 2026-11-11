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
        "f2e36ccf042033f883d56b872b5c1f03500cf315e0c9a27a50f0562266410ebb"
    );
    assert_eq!(
        digest(vite_runtime().unwrap().as_bytes()),
        "3b0e06470baef5b427e2d62dbe7980b42b64e252a0d9de03c9da0dacbdb3df1b"
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
fn missing_or_duplicate_owned_imports_are_refused_before_node_execution() {
    let source = super::super::VITE_RUNTIME;
    let missing = source.replacen(super::VALIDATION_IMPORT, "", 1);
    assert_eq!(
        super::assemble(&missing),
        Err("missing owned validator import in the product runtime")
    );
    let duplicate = [source, super::VALIDATION_IMPORT].concat();
    assert_eq!(
        super::assemble(&duplicate),
        Err("duplicate owned validator import in the product runtime")
    );
}
