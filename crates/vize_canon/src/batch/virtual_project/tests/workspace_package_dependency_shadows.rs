use std::{fs, path::PathBuf};

use crate::{PackageResolutionContext, PackageRoute, PackageRouteBinding};

use super::{VirtualProject, unique_case_dir};

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
