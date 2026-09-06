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
        assert_eq!(
            route.unambiguous_source_path().unwrap().clone(),
            source.canonicalize().unwrap()
        );
    }
}

#[test]
fn extensionless_and_directory_targets_retain_the_native_probe_path() {
    for (target, source, expected_probe) in [
        ("./dist/Widget", "dist/Widget.vue", "dist/Widget.ts"),
        (
            "./dist/feature",
            "dist/feature/index.vue",
            "dist/feature/index.ts",
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        let importer = root.path().join("src");
        let package = root.path().join("node_modules/runtime-package");
        let source_path = package.join(source);
        std::fs::create_dir_all(&importer).unwrap();
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(&source_path, "<script setup lang=\"ts\" />\n").unwrap();
        std::fs::write(
            package.join("package.json"),
            format!(r#"{{"exports":{{".":"{target}"}}}}"#),
        )
        .unwrap();

        let route = PackageRouteResolver::default()
            .resolve(
                &importer,
                "runtime-package",
                PackageSourceOptions::default(),
            )
            .unwrap();
        assert!(route.source_targets.iter().any(|route_source| {
            route_source.source_path == source_path.canonicalize().unwrap()
                && route_source.native_probe_relative_path(&route.package_root)
                    == Some(expected_probe.into())
        }));
    }
}

#[test]
fn declaration_and_authored_twins_are_both_retained_for_native_selection() {
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
    let mut expected = vec![
        declaration.canonicalize().unwrap(),
        package.join("dist/index.ts").canonicalize().unwrap(),
    ];
    expected.sort();
    assert_eq!(route.source_paths, expected);
    assert_eq!(route.unambiguous_source_path(), None);
    let mut probes = route
        .source_targets
        .iter()
        .filter_map(|source| source.native_probe_relative_path(&route.package_root))
        .collect::<Vec<_>>();
    probes.sort();
    probes.dedup();
    assert_eq!(
        probes,
        ["dist/index.d.ts", "dist/index.ts"]
            .into_iter()
            .map(std::path::PathBuf::from)
            .collect::<Vec<_>>()
    );
}

#[test]
fn native_shadow_reverse_mapping_matches_materialized_typescript_twin_priority() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("src");
    let package = root.path().join("node_modules/runtime-package");
    let typescript = package.join("dist/index.ts");
    let vue = package.join("dist/index.vue");
    std::fs::create_dir_all(&importer).unwrap();
    std::fs::create_dir_all(typescript.parent().unwrap()).unwrap();
    std::fs::write(&typescript, "export const native = true\n").unwrap();
    std::fs::write(&vue, "<script setup lang=\"ts\" />\n").unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"exports":{".":"./dist/index.js"}}"#,
    )
    .unwrap();

    let route = PackageRouteResolver::default()
        .resolve(
            &importer,
            "runtime-package",
            PackageSourceOptions::default(),
        )
        .unwrap();
    let shadow_root = root.path().join("virtual/node_modules/runtime-package");
    assert_eq!(
        route.source_for_native_shadow_path(&shadow_root, &shadow_root.join("dist/index.ts")),
        Some(&typescript.canonicalize().unwrap())
    );
    assert_eq!(
        route.source_for_native_shadow_path(
            &shadow_root,
            &root.path().join("unrelated/dist/index.ts")
        ),
        None
    );
}

#[test]
#[cfg(unix)]
fn symlinked_workspace_build_export_falls_back_to_src_entry() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let importer = app.join("src");
    let package = root.path().join("packages/misskey-js");
    let link = app.join("node_modules/misskey-js");
    let source = package.join("src/index.ts");
    std::fs::create_dir_all(&importer).unwrap();
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::fs::write(&source, "export const packed: string = 'ok'\n").unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{
  "type": "module",
  "name": "misskey-js",
  "main": "./built/index.js",
  "types": "./built/index.d.ts",
  "exports": {
    ".": {
      "import": "./built/index.js",
      "types": "./built/index.d.ts",
      "default": "./built/index.js"
    }
  }
}
"#,
    )
    .unwrap();
    symlink(&package, &link).unwrap();

    let route = PackageRouteResolver::default()
        .resolve(&importer, "misskey-js", PackageSourceOptions::default())
        .unwrap();
    assert!(route.workspace_source);
    assert!(route.requires_workspace_source_shadow());
    assert_eq!(
        route.unambiguous_source_path().unwrap().clone(),
        source.canonicalize().unwrap()
    );
    assert!(route.source_targets.iter().any(|target| {
        target.source_path == source.canonicalize().unwrap()
            && target.native_probe_relative_path(&route.package_root)
                == Some("built/index.ts".into())
    }));
}
