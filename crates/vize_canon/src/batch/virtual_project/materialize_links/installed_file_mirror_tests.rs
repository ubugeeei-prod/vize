//! A program file that is itself an installed dependency file (#13044).

use std::fs;

use super::super::VirtualProject;

#[test]
fn external_declaration_snapshots_keep_importer_local_dependencies_cold_and_warm() {
    let workspace = tempfile::tempdir().unwrap();
    let workspace = fs::canonicalize(workspace.path()).unwrap();
    let root = workspace.join("app");
    let package = workspace.join("linked-package");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(package.join("node_modules/vue")).unwrap();
    fs::write(
        package.join("node_modules/vue/index.d.ts"),
        "export declare const local: 77;",
    )
    .unwrap();
    let source = "import type { local } from 'vue'; export declare const value: typeof local;";
    let declaration = package.join("index.d.ts");
    fs::write(&declaration, source).unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project
        .register_declaration_file(&declaration, source)
        .unwrap();
    let mirrored = project
        .find_by_original(&declaration)
        .unwrap()
        .virtual_path
        .clone();
    let expected = (
        mirrored.parent().unwrap().join("node_modules"),
        package.join("node_modules"),
    );
    let plan = project.workspace_alias_plan();
    let cold = project.desired_package_links(&plan);
    assert_eq!(cold.get(&expected.0), Some(&expected.1));
    let warm = project.prepare_incremental_package_link_patch(&plan);
    assert_eq!(warm.desired.get(&expected.0), Some(&expected.1));
    project.materialize().unwrap();
    assert_eq!(
        fs::read_to_string(expected.0.join("vue/index.d.ts")).unwrap(),
        "export declare const local: 77;"
    );
    assert_eq!(fs::read_to_string(declaration).unwrap(), source);
}

/// Nuxt's generated declarations name `../node_modules/<pkg>/…` by path, so
/// the mirror rewrites that one file of the package. Its `package.json` and
/// sibling declarations must stay linked, or a bare `import "pkg"` from any
/// source reports `TS2307` (Mealie's `vuetify` and `@nuxtjs/i18n`).
#[test]
fn a_mirrored_installed_file_keeps_its_package_siblings_linked() {
    let workspace = tempfile::tempdir().unwrap();
    let root = workspace.path().to_path_buf();
    let package = root.join("node_modules").join("pkg");
    fs::create_dir_all(package.join("lib")).unwrap();
    fs::write(root.join("package.json"), r#"{ "name": "app" }"#).unwrap();
    fs::write(
        package.join("package.json"),
        r#"{ "name": "pkg", "types": "./lib/index.d.ts" }"#,
    )
    .unwrap();
    fs::write(
        package.join("lib/index.d.ts"),
        "export declare const a: 1;\n",
    )
    .unwrap();
    fs::write(
        package.join("lib/extra.d.ts"),
        "export declare const b: 2;\n",
    )
    .unwrap();
    let root = fs::canonicalize(&root).unwrap();
    let package = root.join("node_modules").join("pkg");

    let mut project = VirtualProject::new(&root).unwrap();
    project
        .register_declaration_file(
            &package.join("lib/extra.d.ts"),
            "export declare const b: 2;\n",
        )
        .unwrap();
    let virtual_root = project.virtual_root.clone();
    let virtual_package = virtual_root.join("node_modules").join("pkg");
    let mirrored = project
        .virtual_files
        .keys()
        .filter(|path| path.starts_with(&virtual_package))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        mirrored,
        vec![virtual_package.join("lib/extra.d.ts")],
        "the installed file is mirrored at its own coordinates",
    );

    let plan = project.workspace_alias_plan();
    let mut links = project
        .desired_package_links(&plan)
        .into_iter()
        .filter(|(path, _)| path.starts_with(&virtual_package))
        .collect::<Vec<_>>();
    links.sort();
    assert_eq!(
        links,
        vec![
            (
                virtual_package.join("lib/index.d.ts"),
                package.join("lib/index.d.ts"),
            ),
            (
                virtual_package.join("package.json"),
                package.join("package.json"),
            ),
        ],
        "only the mirrored file is owned; its manifest and siblings stay linked",
    );
}
