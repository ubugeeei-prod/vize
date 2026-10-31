//! A declaration-only Canon stub must not hide the project's real Vite+ loader.
#![expect(
    clippy::disallowed_types,
    reason = "complete public CLI fixture packets"
)]

use super::support::{run, write};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[test]
fn lint_keeps_its_complete_report_when_a_types_only_vite_stub_is_present() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let config = "import {settings} from './settings';\nexport default {vue:settings};\n";
    let settings = "export const settings = {linter:{preset:'essential',rules:{'a11y/alt-text':'error'}}} satisfies Record<string, unknown>;\n";
    let component = include_str!("../fixtures/lint-execution/Warning.vue");
    for (name, source) in [
        ("package.json", "{\"private\":true,\"type\":\"module\"}"),
        ("vite.config.ts", config),
        ("settings.ts", settings),
        ("CardView.vue", component),
    ] {
        write(root, name, source);
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let installed = workspace
        .join("node_modules/vite-plus")
        .canonicalize()
        .unwrap();
    let modules = root.join("node_modules");
    fs::create_dir_all(&modules).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&installed, modules.join("vite-plus")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&installed, modules.join("vite-plus")).unwrap();
    let expected = json!([{
        "file":"CardView.vue", "errorCount":1, "warningCount":0,
        "messages":[{
            "ruleId":"a11y/alt-text", "severity":2,
            "message":"[vize:a11y/alt-text] <img> elements must have an alt attribute",
            "line":2,"column":3,"endLine":2,"endColumn":28,
            "help":"Add an alt attribute: <img alt=\"Description\"> or <img alt=\"\"> for decorative images",
            "ruleDocsPath":"docs/content/rules/accessibility.md"
        }]
    }]);
    let lint = || {
        let output = run(root, &["lint", "CardView.vue", "--format", "json"]);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report, expected);
    };
    lint();
    let stub = modules.join("vite");
    fs::create_dir(&stub).unwrap();
    let manifest = "{\"name\":\"vite\",\"types\":\"client.d.ts\"}";
    let declaration = "interface ImportMetaEnv { readonly [key: string]: string | boolean | undefined; }\ninterface ImportMeta { readonly env: ImportMetaEnv; }\nexport {};\n";
    fs::write(stub.join("package.json"), manifest).unwrap();
    fs::write(stub.join("client.d.ts"), declaration).unwrap();
    lint();
    assert_eq!(
        fs::read_to_string(stub.join("package.json")).unwrap(),
        manifest
    );
    assert_eq!(
        fs::read_to_string(stub.join("client.d.ts")).unwrap(),
        declaration
    );
    assert_eq!(fs::read_link(modules.join("vite-plus")).unwrap(), installed);
    for (name, source) in [
        ("vite.config.ts", config),
        ("settings.ts", settings),
        ("CardView.vue", component),
    ] {
        assert_eq!(fs::read_to_string(root.join(name)).unwrap(), source);
    }
}
