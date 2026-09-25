use super::unique_case_dir;

#[test]
fn retains_authored_external_inputs_but_drops_external_node_modules() {
    let workspace = unique_case_dir("retained-external-inputs");
    let package_root = workspace.join("packages/app");
    let app = package_root.join("src/main.ts");
    let shared = workspace.join("shared/globals.d.ts");
    let types_package = workspace.join("node_modules/@types/vize/index.d.ts");
    let mut files = vec![app.clone(), shared.clone(), types_package];

    super::super::resolve::retain_project_files(
        &mut files,
        &[app.clone(), shared.clone()],
        &package_root,
    );

    assert_eq!(files, vec![app, shared]);
}

#[test]
fn retains_explicit_tsconfig_files_under_ancestor_node_modules() {
    use crate::commands::check::tsconfig_inputs::{
        TsconfigInputCache, collect_default_check_files,
    };

    let workspace = unique_case_dir("explicit-node-modules-files");
    let _ = std::fs::remove_dir_all(&workspace);
    let package_root = workspace.join("app");
    let app = package_root.join("src/a.tsx");
    let jsx = workspace.join("node_modules/vue/jsx.d.ts");
    let ambient_types = workspace.join("node_modules/@types/ambient/index.d.ts");
    std::fs::create_dir_all(app.parent().unwrap()).unwrap();
    std::fs::create_dir_all(jsx.parent().unwrap()).unwrap();
    std::fs::write(&app, "export const view = <div />;\n").unwrap();
    std::fs::write(&jsx, "export {};\n").unwrap();
    std::fs::write(
        package_root.join("tsconfig.json"),
        r#"{"files":["../node_modules/vue/jsx.d.ts"],"include":["src/**/*"]}"#,
    )
    .unwrap();

    let inputs = collect_default_check_files(
        &package_root,
        Some(&package_root.join("tsconfig.json")),
        true,
        &mut TsconfigInputCache::default(),
    );
    assert!(
        inputs.contains(&jsx),
        "tsconfig files entry missing: {inputs:?}"
    );
    assert!(inputs.contains(&app), "source input missing: {inputs:?}");

    let mut files = inputs.clone();
    files.push(ambient_types);
    super::super::resolve::retain_project_files(&mut files, &inputs, &package_root);
    assert_eq!(files, inputs);

    let _ = std::fs::remove_dir_all(&workspace);
}
