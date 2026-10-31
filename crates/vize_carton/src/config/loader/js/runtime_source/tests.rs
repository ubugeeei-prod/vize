use super::{VALIDATION, vite_runtime};
use sha2::{Digest, Sha256};

const CONTROLS: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/runtime-controls.original.json"
);

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .into_iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn complete_product_sources_and_native_assembly_keep_their_exact_postimages() {
    assert_eq!(
        digest(VALIDATION.as_bytes()),
        "515047133aeba1ec3d34c80b9346a532d00bd0d47e3ce21d9cd0a6fa94a1dd82"
    );
    assert_eq!(
        digest(super::super::VITE_RUNTIME.as_bytes()),
        "b6be222c3d0f68112e7e3d52e801cf757827b12ad6465d5cb1c754f998f0a98f"
    );
    assert_eq!(
        digest(vite_runtime().unwrap().as_bytes()),
        "c8dad1baecba626f18b08eacd89236a7667e19c214dddb7b4dc7d559269efca5"
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

#[test]
fn missing_or_duplicate_owned_imports_are_refused_before_node_execution() {
    let source = super::super::VITE_RUNTIME;
    let missing = source.replacen(super::VALIDATION_IMPORT, "", 1);
    assert_eq!(super::assemble(&missing),Err("missing owned validator import in the product runtime"));
    let duplicate = [source,super::VALIDATION_IMPORT].concat();
    assert_eq!(super::assemble(&duplicate),Err("duplicate owned validator import in the product runtime"));
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
