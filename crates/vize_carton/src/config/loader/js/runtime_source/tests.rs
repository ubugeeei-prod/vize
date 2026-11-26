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
        "866f0e42210ce4264432883dcff21f855af86a04577945ef1a0167b0b57462c9"
    );
    assert_eq!(
        digest(super::super::VITE_RUNTIME.as_bytes()),
        "b905dd0eb14d9e75528a79258f3f6a830ad517fdf4f8302c1f249662788833be"
    );
    assert_eq!(
        digest(vite_runtime().unwrap().as_bytes()),
        "56c5ffd1955ffaaba1a0809b3e6fdab11e1349e79971f632400f6a21ec0de0e4"
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
fn owned_modules_share_one_scope_and_plugin_modules_load_lazily() {
    let config = vite_runtime().unwrap();
    let tools = super::vite_tools_runtime().unwrap();
    let lazy = "import(\"data:text/javascript;charset=utf-8,\" + encodeURIComponent(";
    for runtime in [&config, &tools] {
        assert_eq!(runtime.matches(" from \"./").count(), 0);
        assert_eq!(runtime.matches("import(\"./").count(), 0);
    }
    assert_eq!(config.matches(lazy).count(), 1);
    assert_eq!(config.matches(super::TOOLS_UNAVAILABLE).count(), 1);
    assert_eq!(tools.matches(lazy).count(), 2);
    assert_eq!(
        super::assemble(
            "import { x } from \"./unowned.js\";\nexport const y = x;\n",
            false
        ),
        Err("unowned relative import in the product runtime")
    );
    assert_eq!(
        super::assemble(
            "import { x } from \"./vite-runtime-validation.js\"\n",
            false
        ),
        Err("unterminated import in the product runtime")
    );
    let builtin = "import { resolve } from \"node:path\";\nexport { resolve };\n";
    assert_eq!(
        super::assemble(builtin, false).unwrap(),
        crate::cstr!("{}{builtin}", super::assemble("", false).unwrap())
    );
}

#[test]
fn windows_crlf_product_runtime_assembles_like_the_lf_source() {
    let lf = super::super::VITE_RUNTIME;
    let crlf = lf.replace('\n', "\r\n");
    assert_eq!(
        super::assemble(&crlf, false).unwrap().replace("\r\n", "\n"),
        super::assemble(lf, false).unwrap()
    );
}

fn run(script: &str) -> serde_json::Value {
    let project = tempfile::tempdir().unwrap();
    let runtime = super::super::runtime_file::NodeRuntimeFile::new(script).unwrap();
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
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn lazy_plugin_modules_resolve_and_inspect_inside_the_private_modules() {
    let mut config = vite_runtime().unwrap();
    config.push_str(
        r#"
const rows = [];
try { await projectViteConfigSource({ vue: { plugins: [{ name: "x", version: "1" }] } }); }
catch (error) { rows.push(error.message); }
rows.push(JSON.stringify(await projectViteConfigSource({ vue: { plugins: [false, null] } })));
try { await inspectViteConfigPlugins({ vue: {} }); } catch (error) { rows.push(error.message); }
process.stdout.write(JSON.stringify(rows));
"#,
    );
    assert_eq!(
        run(&config),
        serde_json::json!([
            "[plugin/entry-invalid] vue.plugins[0]: expected a defineVizePlugin entry or a v1 definePlugin rule plugin; other v1 hooks keep their native APIs",
            "{\"typeChecker\":{\"jsxTypecheck\":true}}",
            "plugin tools are not part of the config runtime"
        ])
    );
    let mut tools = super::vite_tools_runtime().unwrap();
    tools.push_str(
        r#"
const inspection = await inspectViteConfigPlugins({ vue: {} });
process.stdout.write(JSON.stringify([inspection.order.length, inspection.digest.length, inspection.admission.lint.status]));
"#,
    );
    assert_eq!(run(&tools), serde_json::json!([0, 64, "unused"]));
}
