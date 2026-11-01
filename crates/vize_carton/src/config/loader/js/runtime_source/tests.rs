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
        "515047133aeba1ec3d34c80b9346a532d00bd0d47e3ce21d9cd0a6fa94a1dd82"
    );
    assert_eq!(
        digest(super::super::VITE_RUNTIME.as_bytes()),
        "8cbd863757317f245d08daf8e0e171be44a36948fc875d1f9ab7f38a7f6fc048"
    );
    assert_eq!(
        digest(vite_runtime().unwrap().as_bytes()),
        "a0682c22fe27a45b314b89dc3bb3ec441bb6cff173dac2dcd788ed575663c014"
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
