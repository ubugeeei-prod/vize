use std::fs;

use super::{VirtualProject, unique_case_dir};

#[test]
fn late_registered_declaration_import_uses_the_mirror() {
    let root = unique_case_dir("late-declaration-import");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("types")).unwrap();
    fs::create_dir_all(root.join("vendor/client")).unwrap();
    let env = root.join("env.d.ts");
    let builder = root.join("types/builder-env.d.ts");
    let vendor = root.join("vendor/client/index.d.ts");
    fs::write(
        &env,
        "/// <reference path=\"./types/builder-env.d.ts\" />\nexport {};\n",
    )
    .unwrap();
    fs::write(&builder, "import \"../vendor/client/index\";\n").unwrap();
    fs::write(
        &vendor,
        "declare module \"*?vize-reference-fixture\" { const source: string; export default source; }\n",
    )
    .unwrap();

    let mut project = VirtualProject::new(&root).unwrap();
    project.register_path(&env).unwrap();
    project.register_reachable_dependencies().unwrap();
    project.finalize_package_routes().unwrap();
    project.materialize().unwrap();

    let actual = fs::read_to_string(project.virtual_root().join("types/builder-env.d.ts")).unwrap();
    let expected = project.virtual_root().join("vendor/client/index");
    assert!(actual.contains(expected.to_str().unwrap()), "{actual}");
    assert!(
        project
            .virtual_root()
            .join("vendor/client/index.d.ts")
            .is_file()
    );
    let _ = fs::remove_dir_all(&root);
}
