use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn reference_path_keeps_module_declaration_and_its_side_effect_import() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "reference-path-module-side-effect",
        &[
            (
                "env.d.ts",
                "/// <reference path=\"./types/builder-env.d.ts\" />\nexport {};\n",
            ),
            (
                "types/builder-env.d.ts",
                "import \"../vendor/client/index\";\n",
            ),
            (
                "vendor/client/index.d.ts",
                "declare module \"*?raw\" { const source: string; export default source; }\n",
            ),
            (
                "src/a.ts",
                "import source from \"./data.txt?raw\";\nexport const text: string = source;\n",
            ),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "types": []
  },
  "include": ["env.d.ts", "src/**/*"]
}"#,
    )
    .expect("write the authored TypeScript configuration");
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(snapshot, Some(Vec::new()));
}
