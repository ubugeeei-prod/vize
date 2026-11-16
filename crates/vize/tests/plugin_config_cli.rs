//! `vue.plugins` resolves once into the canonical plugin plan (#13074). An
//! empty plan leaves product output byte-identical; capabilities that no
//! product implements yet fail closed but stay inspectable.
#![expect(clippy::disallowed_types, reason = "complete CLI process packets")]

use serde_json::Value;
use std::{fs, path::Path, process::Command};

const APP: &str = "<template>\n  <div v-for=\"item in items\">{{ item }}</div>\n</template>\n";

fn sdk() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../npm/plugin-sdk/v2.js")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn project(plugins: &str, setup: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("vite.config.mjs"),
        vize_l0::cstr!(
            "import {{ pathToFileURL }} from \"node:url\";\n\
             const {{ defineVizePlugin }} = await import(pathToFileURL(process.env.VIZE_TEST_PLUGIN_SDK).href);\n\
             const plugin = (name, setup) => defineVizePlugin({{ apiVersion: 2, stability: \"experimental\", name, version: \"1.0.0\", setup }});\n\
             void plugin;\n{setup}\n\
             export default {{ vue: {{ linter: {{ preset: \"essential\" }}{plugins} }} }};\n"
        )
        .as_str(),
    )
    .unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/App.vue"), APP).unwrap();
    dir
}

fn vize(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("VIZE_TEST_PLUGIN_SDK", sdk())
        .args(args)
        .output()
        .unwrap()
}

fn plugin_error(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .find_map(|line| line.trim().strip_prefix("VizePluginPlanError: "))
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn empty_plugin_lists_keep_lint_output_and_exit_status_identical() {
    let packets: Vec<(Option<i32>, Vec<u8>, Vec<u8>)> =
        ["", ", plugins: []", ", plugins: [false, null, undefined]"]
            .into_iter()
            .map(|plugins| {
                let dir = project(plugins, "");
                let output = vize(dir.path(), &["lint", "src", "--format", "json"]);
                let root = dir.path().canonicalize().unwrap();
                let normalize = |bytes: Vec<u8>| {
                    String::from_utf8(bytes)
                        .unwrap()
                        .replace(root.to_str().unwrap(), "<root>")
                        .into_bytes()
                };
                (
                    output.status.code(),
                    normalize(output.stdout),
                    normalize(output.stderr),
                )
            })
            .collect();
    assert_eq!(packets[1], packets[0]);
    assert_eq!(packets[2], packets[0]);
    assert_eq!(
        packets[0].0,
        Some(1),
        "the fixture reports a missing v-for key"
    );
}

#[test]
fn unimplemented_capabilities_fail_closed_and_stay_inspectable() {
    let dir = project(
        ", plugins: [design({ strict: true })]",
        "const design = (options) => plugin(\"@acme/design\", (api) => {\n  void options;\n  api.lint.rule({ id: \"no-raw-color\", meta: { messages: { raw: \"Use a token.\" } }, visit: [\"ui.bind\"], run() {} });\n})(options);",
    );
    let lint = vize(dir.path(), &["lint", "src"]);
    assert_eq!(lint.status.code(), Some(2));
    assert_eq!(
        plugin_error(&lint),
        "[plugin/capability-unadmitted] @acme/design: no Vize product implements lint.rule `@acme/design/no-raw-color` yet; run `vize plugin inspect` for the plan"
    );
    let inspect = vize(dir.path(), &["plugin", "inspect"]);
    assert_eq!(
        inspect.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let inspection: Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(inspection["order"], serde_json::json!(["@acme/design"]));
    assert_eq!(inspection["planSchema"], 1);
    assert_eq!(inspection["admission"]["lint"]["status"], "refused");
    assert_eq!(inspection["admission"]["compiler"]["status"], "unused");
    assert_eq!(inspection["digest"].as_str().map(str::len), Some(64));
    assert_eq!(
        inspection["plugins"][0]["capabilities"][0]["key"],
        "@acme/design/no-raw-color"
    );
    assert_eq!(
        inspection["plugins"][0]["options"],
        serde_json::json!({ "strict": true })
    );
}

#[test]
fn plan_resolution_errors_fail_every_entrypoint() {
    let dir = project(
        ", plugins: [plugin(\"@acme/routes\", (api) => api.sfc.block(\"route\"))(), plugin(\"@other/routes\", (api) => api.sfc.block(\"route\"))()]",
        "",
    );
    let message = "[plugin/owner-conflict] sfc.custom-block:route: claimed by both @acme/routes and @other/routes";
    let lint = vize(dir.path(), &["lint", "src"]);
    assert_eq!(lint.status.code(), Some(2));
    assert_eq!(plugin_error(&lint), message);
    let inspect = vize(dir.path(), &["plugin", "inspect"]);
    assert_eq!(inspect.status.code(), Some(1));
    assert_eq!(plugin_error(&inspect), message);
}
