#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use std::{fs, path::PathBuf};

use crate::{PackageResolutionContext, PackageRoute, PackageRouteBinding};

use super::{VirtualProject, unique_case_dir};

#[path = "pnpm_workspace_routes.rs"]
mod pnpm_workspace_routes;

#[test]
fn workspace_build_shadow_mirrors_source_dependency_tree() {
    let project_root = unique_case_dir("workspace-build-shadow-deps");
    let package_root = project_root.parent().unwrap().join(format!(
        "{}-package",
        project_root.file_name().unwrap().to_string_lossy()
    ));
    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&package_root);
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::create_dir_all(package_root.join("src/autogen")).unwrap();
    fs::write(
        project_root.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"Bundler","strict":true}}"#,
    )
    .unwrap();
    let entry_path = project_root.join("src/entry.ts");
    fs::write(
        &entry_path,
        "import * as Misskey from 'misskey-js';\ntype File = Misskey.entities.DriveFile;\n",
    )
    .unwrap();
    let manifest = r#"{
  "type": "module",
  "name": "misskey-js",
  "exports": {
    ".": {
      "import": "./built/index.js",
      "types": "./built/index.d.ts"
    }
  }
}

"#;
    fs::write(package_root.join("package.json"), manifest).unwrap();
    let index_contents = "export * as entities from \"./entities.js\";\n";
    let entities_contents = "export type { DriveFile } from \"./autogen/models.js\";\n";
    let models_contents = "export interface DriveFile { id: string }\n";
    fs::write(package_root.join("src/index.ts"), index_contents).unwrap();
    fs::write(package_root.join("src/entities.ts"), entities_contents).unwrap();
    fs::write(package_root.join("src/autogen/models.ts"), models_contents).unwrap();

    let project_root = project_root.canonicalize().unwrap();
    let package_root = package_root.canonicalize().unwrap();
    let entry_path = entry_path.canonicalize().unwrap();
    let manifest_path = package_root.join("package.json");
    let index_path = package_root.join("src/index.ts");
    let entities_path = package_root.join("src/entities.ts");
    let models_path = package_root.join("src/autogen/models.ts");
    let route = PackageRoute {
        source_paths: vec![index_path.clone()],
        dependency_paths: vec![entities_path.clone()],
        source_targets: vec![
            crate::PackageRouteSource {
                target_path: package_root.join("built/index.js"),
                source_path: index_path.clone(),
                native_probe_path: package_root.join("built/index.ts"),
            },
            crate::PackageRouteSource {
                target_path: package_root.join("built/index.d.ts"),
                source_path: index_path.clone(),
                native_probe_path: package_root.join("built/index.ts"),
            },
        ],
        package_root: package_root.clone(),
        package_link_root: package_root.clone(),
        manifest_path: manifest_path.clone(),
        package_name: Some("misskey-js".into()),
        workspace_source: true,
        nested_routes: Vec::new(),
    };
    let mut project = VirtualProject::new(&project_root).unwrap();
    project.set_package_routes([PackageRouteBinding {
        importer_path: entry_path.clone(),
        specifier: "misskey-js".into(),
        occurrence_mode: crate::PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route: Some(route),
        invalidation_paths: vec![manifest_path.clone(), index_path.clone()],
    }]);
    project.register_path(&entry_path).unwrap();
    project.register_package_route_targets().unwrap();
    project.register_reachable_dependencies().unwrap();
    project.finalize_package_routes().unwrap();
    project.materialize().unwrap();

    let shadow_root = project
        .find_by_original(&entry_path)
        .unwrap()
        .virtual_path
        .parent()
        .unwrap()
        .join("node_modules/misskey-js");
    assert_eq!(
        fs::read_to_string(shadow_root.join("built/index.ts")).unwrap(),
        index_contents
    );
    assert_eq!(
        fs::read_to_string(shadow_root.join("built/index.d.ts")).unwrap(),
        "export * from \"./index.js\";\n"
    );
    assert_eq!(
        fs::read_to_string(shadow_root.join("built/entities.ts")).unwrap(),
        entities_contents
    );
    assert_eq!(
        fs::read_to_string(shadow_root.join("built/autogen/models.ts")).unwrap(),
        models_contents
    );
    assert_eq!(
        project
            .find_by_virtual(&shadow_root.join("built/autogen/models.ts"))
            .map(|file| file.original_path.as_path()),
        Some(models_path.as_path())
    );
    assert_eq!(
        project
            .package_routes_snapshot()
            .pop()
            .unwrap()
            .route
            .as_ref()
            .unwrap()
            .source_for_native_shadow_path(&shadow_root, &shadow_root.join("built/index.d.ts"))
            .map(PathBuf::as_path),
        Some(index_path.as_path())
    );

    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&package_root);
}

#[test]
fn nested_workspace_package_shadow_tracks_late_relative_dependencies() {
    let root = unique_case_dir("nested-workspace-shadow-deps");
    let _ = fs::remove_dir_all(&root);
    for name in ["a", "b", "c"] {
        fs::create_dir_all(root.join(format!("packages/{name}/src"))).unwrap();
        fs::write(
            root.join(format!("packages/{name}/package.json")),
            format!(r#"{{"name":"@x/{name}","type":"module","exports":"./src/index.ts"}}"#),
        )
        .unwrap();
    }
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.ts"), "export * from '@x/a';\n").unwrap();
    fs::write(
        root.join("packages/a/src/index.ts"),
        "export * from '@x/b';\n",
    )
    .unwrap();
    fs::write(
        root.join("packages/b/src/index.ts"),
        "export * from '@x/c';\n",
    )
    .unwrap();
    fs::write(
        root.join("packages/c/src/index.ts"),
        "export * from './util';\n",
    )
    .unwrap();
    fs::write(
        root.join("packages/c/src/util.ts"),
        "export { default as Btn } from './Btn.vue';\n",
    )
    .unwrap();
    fs::write(
        root.join("packages/c/src/Btn.vue"),
        "<template><button /></template>\n",
    )
    .unwrap();

    let route = |name: &str, nested_routes: Vec<PackageRoute>| {
        let package_root = root.join(format!("packages/{name}"));
        let entry = package_root.join("src/index.ts");
        PackageRoute {
            source_paths: vec![entry.clone()],
            dependency_paths: Vec::new(),
            source_targets: vec![crate::PackageRouteSource {
                target_path: entry.clone(),
                source_path: entry.clone(),
                native_probe_path: entry,
            }],
            manifest_path: package_root.join("package.json"),
            package_root: package_root.clone(),
            package_link_root: package_root,
            package_name: Some(format!("@x/{name}").into()),
            workspace_source: true,
            nested_routes,
        }
    };
    let c = route("c", Vec::new());
    let b = route("b", vec![c]);
    let a = route("a", vec![b]);
    let importer = root.join("src/main.ts");
    let mut project = VirtualProject::new(&root).unwrap();
    project.set_package_routes([PackageRouteBinding {
        importer_path: importer.clone(),
        specifier: "@x/a".into(),
        occurrence_mode: crate::PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route: Some(a),
        invalidation_paths: Vec::new(),
    }]);
    project.register_path(&importer).unwrap();
    project.register_package_route_targets().unwrap();
    project.register_reachable_dependencies().unwrap();
    project.finalize_package_routes().unwrap();

    let paths = project.topology_shadow_paths();
    for relative in ["src/util.ts", "src/Btn.vue.ts"] {
        let suffix = format!("node_modules/@x/a/node_modules/@x/b/node_modules/@x/c/{relative}");
        assert!(
            paths.iter().any(|path| path.ends_with(&suffix)),
            "nested shadow is missing {suffix}: {paths:#?}"
        );
    }
    let _ = fs::remove_dir_all(&root);
}
