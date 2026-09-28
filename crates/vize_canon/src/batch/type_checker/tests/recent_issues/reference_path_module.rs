use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use crate::BatchTypeChecker;

#[test]
fn reference_path_keeps_module_declaration_and_its_side_effect_import() {
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
    let mut checker = BatchTypeChecker::new(&project_root).expect("create project checker");
    checker.scan_project().expect("scan declaration graph");
    let projections = checker
        .virtual_files()
        .into_iter()
        .filter(|file| file.original_path.to_string_lossy().ends_with(".d.ts"))
        .map(|file| {
            (
                file.original_path.clone(),
                file.virtual_path.clone(),
                file.content.clone(),
            )
        })
        .collect::<Vec<_>>();
    let builder = projections
        .iter()
        .find(|(original, _, _)| original == &project_root.join("types/builder-env.d.ts"))
        .expect("the referenced declaration must be mirrored");
    let vendor = projections
        .iter()
        .find(|(original, _, _)| original == &project_root.join("vendor/client/index.d.ts"))
        .expect("the transitive declaration must be mirrored");
    let vendor_module = vendor.1.to_string_lossy().replace(".d.ts", "");
    assert!(
        builder.2.contains(vendor_module.as_str()),
        "{projections:#?}"
    );
    if resolve_test_tsgo_binary().is_none() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(snapshot, Some(Vec::new()), "{projections:#?}");
}
