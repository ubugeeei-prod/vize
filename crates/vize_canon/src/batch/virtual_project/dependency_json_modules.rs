//! An external package source that the app already mirrors imports a relative
//! JSON module. Materializing the app must write that JSON beside the shadowed
//! source, the same way a root JSON input is written (#10350).

use std::fs;
use std::path::{Path, PathBuf};

use vize_carton::cstr;

use super::VirtualProject;
use crate::{PackageResolutionContext, PackageRoute, PackageRouteBinding};

fn unique_case_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("dependency-json-{name}-{}", std::process::id()).as_str())
}

#[test]
fn materialize_writes_dependency_package_json_module() {
    let project_root = unique_case_dir("widget-app");
    let mut package_name = project_root.file_name().unwrap().to_os_string();
    package_name.push("-widget-lib");
    let package_root = project_root.parent().unwrap().join(package_name);
    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&package_root);
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::create_dir_all(package_root.join("src/tokens/source")).unwrap();
    fs::write(
        project_root.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"Bundler","resolveJsonModule":true,"strict":true},"include":["src/**/*.ts","src/**/*.vue","src/**/*.d.ts"]}"#,
    )
    .unwrap();
    let entry_path = project_root.join("src/main.ts");
    fs::write(
        &entry_path,
        "import { Widget } from \"widget-lib\";\nvoid Widget;\n",
    )
    .unwrap();
    let manifest = r#"{
  "name": "widget-lib",
  "type": "module",
  "exports": { ".": "./src/index.ts" }
}
"#;
    let index_contents = "export { colors } from \"./tokens.js\";\n";
    let tokens_contents =
        "import colors from \"./tokens/source/colors.tokens.json\";\nexport { colors };\n";
    let json_contents = "{\"primary\":\"#0057ff\"}\n";
    fs::write(package_root.join("package.json"), manifest).unwrap();
    fs::write(package_root.join("src/index.ts"), index_contents).unwrap();
    fs::write(package_root.join("src/tokens.ts"), tokens_contents).unwrap();
    fs::write(
        package_root.join("src/tokens/source/colors.tokens.json"),
        json_contents,
    )
    .unwrap();

    let project_root = project_root.canonicalize().unwrap();
    let package_root = package_root.canonicalize().unwrap();
    let entry_path = entry_path.canonicalize().unwrap();
    let manifest_path = package_root.join("package.json");
    let index_path = package_root.join("src/index.ts");
    let tokens_path = package_root.join("src/tokens.ts");
    let route = PackageRoute {
        source_paths: vec![index_path.clone()],
        dependency_paths: vec![tokens_path.clone()],
        source_targets: vec![crate::PackageRouteSource {
            target_path: index_path.clone(),
            source_path: index_path.clone(),
            native_probe_path: index_path.clone(),
        }],
        package_root: package_root.clone(),
        package_link_root: package_root.clone(),
        manifest_path: manifest_path.clone(),
        package_name: Some("widget-lib".into()),
        workspace_source: true,
        nested_routes: Vec::new(),
    };
    let mut project = VirtualProject::new(&project_root).unwrap();
    project.set_package_routes([PackageRouteBinding {
        importer_path: entry_path.clone(),
        specifier: "widget-lib".into(),
        occurrence_mode: crate::PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route: Some(route),
        invalidation_paths: vec![manifest_path, index_path, tokens_path],
        shared_invalidation_paths: Vec::new(),
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
        .join("node_modules/widget-lib");
    let tokens_shadow = shadow_root.join("src/tokens.ts");
    let json_shadow = shadow_root.join("src/tokens/source/colors.tokens.json");
    let tokens_text = fs::read_to_string(&tokens_shadow).unwrap_or_default();
    let kept_json_import = tokens_text
        .split("./tokens/source/colors.tokens.json")
        .nth(1)
        .is_some();
    assert!(
        kept_json_import,
        "shadowed dependency should keep the relative JSON import, got {tokens_text:?}"
    );
    assert!(
        json_shadow.is_file(),
        "dependency JSON was not materialized at {}",
        json_shadow.display()
    );
    assert_eq!(fs::read_to_string(&json_shadow).unwrap(), json_contents);

    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&package_root);
}
