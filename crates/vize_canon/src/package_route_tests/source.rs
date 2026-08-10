#![allow(clippy::disallowed_methods)]

use super::super::{PackageRouteResolver, PackageSourceOptions};

/// A source checkout of a workspace package never emits `dist/index.js`, so a
/// manifest naming the runtime target must land on its authored twin instead
/// of probing `dist/index.js.ts`.
#[test]
fn a_runtime_export_falls_back_to_its_typescript_twin() {
    for (target, authored) in [
        ("./dist/index.js", "dist/index.ts"),
        ("./dist/index.jsx", "dist/index.tsx"),
        ("./dist/index.mjs", "dist/index.mts"),
        ("./dist/index.cjs", "dist/index.cts"),
        ("./dist/index.js", "dist/index.vue"),
    ] {
        let root = tempfile::tempdir().unwrap();
        let importer = root.path().join("src");
        let package = root.path().join("node_modules/runtime-package");
        let source = package.join(authored);
        std::fs::create_dir_all(&importer).unwrap();
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "export const value = 1\n").unwrap();
        std::fs::write(
            package.join("package.json"),
            format!(r#"{{"exports":{{".":{{"import":"{target}"}}}}}}"#),
        )
        .unwrap();

        let route = PackageRouteResolver::default()
            .resolve(
                &importer,
                "runtime-package",
                PackageSourceOptions::default(),
            )
            .unwrap_or_else(|| panic!("{target} did not resolve to {authored}"));
        assert_eq!(route.source_path, source.canonicalize().unwrap());
    }
}

#[test]
fn a_declaration_sidecar_still_wins_over_a_typescript_twin() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("src");
    let package = root.path().join("node_modules/runtime-package");
    let declaration = package.join("dist/index.d.ts");
    std::fs::create_dir_all(&importer).unwrap();
    std::fs::create_dir_all(declaration.parent().unwrap()).unwrap();
    std::fs::write(&declaration, "export declare const value: number\n").unwrap();
    std::fs::write(package.join("dist/index.ts"), "export const value = 1\n").unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"exports":{".":{"import":"./dist/index.js"}}}"#,
    )
    .unwrap();

    let route = PackageRouteResolver::default()
        .resolve(
            &importer,
            "runtime-package",
            PackageSourceOptions::default(),
        )
        .unwrap();
    assert_eq!(route.source_path, declaration.canonicalize().unwrap());
}
