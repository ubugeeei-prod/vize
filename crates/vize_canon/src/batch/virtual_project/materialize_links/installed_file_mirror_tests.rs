//! A program file that is itself an installed dependency file (#13044).

use std::fs;
use std::path::{Path, PathBuf};

use vize_carton::cstr;

use super::super::VirtualProject;

fn case_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("installed-file-mirror")
        .join(cstr!("{name}-{}", std::process::id()).as_str());
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Nuxt's generated declarations name `../node_modules/<pkg>/…` by path, so
/// the mirror rewrites that one file of the package. Its `package.json` and
/// sibling declarations must stay linked, or a bare `import "pkg"` from any
/// source reports `TS2307` (Mealie's `vuetify` and `@nuxtjs/i18n`).
#[test]
fn a_mirrored_installed_file_keeps_its_package_siblings_linked() {
    let root = case_dir("siblings");
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
    let _ = fs::remove_dir_all(&root);
}
