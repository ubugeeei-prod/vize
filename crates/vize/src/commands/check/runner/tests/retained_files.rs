use super::unique_case_dir;

#[test]
fn retains_authored_external_inputs_but_drops_external_node_modules() {
    let workspace = unique_case_dir("retained-external-inputs");
    let package_root = workspace.join("packages/app");
    let app = package_root.join("src/main.ts");
    let shared = workspace.join("shared/globals.d.ts");
    let types_package = workspace.join("node_modules/@types/vize/index.d.ts");
    let mut files = vec![app.clone(), shared.clone(), types_package];

    super::super::resolve::retain_project_files(&mut files, &package_root);

    assert_eq!(files, vec![app, shared]);
}
