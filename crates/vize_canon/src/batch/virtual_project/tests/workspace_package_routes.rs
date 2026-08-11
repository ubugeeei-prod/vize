use std::fs;

use crate::{PackageResolutionContext, PackageRoute, PackageRouteBinding};

use super::{VirtualProject, unique_case_dir};

#[test]
fn package_shadow_preserves_user_paths_and_raw_manifest() {
    let project_root = unique_case_dir("workspace-package-shadow");
    let package_root = project_root.parent().unwrap().join(
        vize_carton::cstr!(
            "{}-package",
            project_root.file_name().unwrap().to_string_lossy()
        )
        .as_str(),
    );
    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&package_root);
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::create_dir_all(package_root.join("src")).unwrap();
    fs::write(
        project_root.join("tsconfig.json"),
        r#"{"compilerOptions":{"moduleResolution":"Bundler","customConditions":["browser"],"paths":{"@scope/workspace-vue":["./user-fallback"]}}}"#,
    )
    .unwrap();
    let entry_path = project_root.join("src/entry.ts");
    fs::write(
        &entry_path,
        "import Widget from '@scope/workspace-vue'\nvoid Widget\n",
    )
    .unwrap();
    let manifest = r#"{
  "name": "@scope/workspace-vue",
  "exports": { ".": { "browser": null, "types": "./src/Root.vue" } }
}

"#;
    let manifest_path = package_root.join("package.json");
    fs::write(&manifest_path, manifest).unwrap();
    let component_path = package_root.join("src/Root.vue");
    fs::write(
        &component_path,
        "<script setup lang=\"ts\">defineProps<{ count: number }>()</script>\n",
    )
    .unwrap();

    let route = PackageRoute {
        source_paths: vec![component_path.clone()],
        dependency_paths: Vec::new(),
        source_targets: vec![crate::PackageRouteSource {
            target_path: component_path.clone(),
            source_path: component_path.clone(),
            native_probe_path: component_path.with_extension("d.vue.ts"),
        }],
        package_root: package_root.clone(),
        package_link_root: package_root.clone(),
        manifest_path: manifest_path.clone(),
        package_name: Some("@scope/workspace-vue".into()),
        workspace_source: true,
        nested_routes: Vec::new(),
    };
    let mut project = VirtualProject::new(&project_root).unwrap();
    project.set_package_routes([PackageRouteBinding {
        importer_path: entry_path.clone(),
        specifier: "@scope/workspace-vue".into(),
        occurrence_mode: crate::PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route: Some(route),
        invalidation_paths: vec![manifest_path.clone(), component_path.clone()],
    }]);
    let context = &project.package_routes_snapshot()[0].context;
    assert_eq!(context.module_resolution.as_deref(), Some("bundler"));
    assert_eq!(context.active_conditions, ["browser"]);
    project.register_path(&entry_path).unwrap();
    project.register_package_route_targets().unwrap();
    project.finalize_package_routes().unwrap();
    project.materialize().unwrap();

    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(project.virtual_root().join("tsconfig.json")).unwrap(),
    )
    .unwrap();
    let paths = config["compilerOptions"]["paths"]["@scope/workspace-vue"]
        .as_array()
        .unwrap();
    assert!(
        paths
            .iter()
            .filter_map(serde_json::Value::as_str)
            .all(|path| path.contains("user-fallback")),
        "package topology must not overwrite user-authored paths: {paths:#?}"
    );
    assert_eq!(config["compilerOptions"]["allowArbitraryExtensions"], true);

    let shadow_root = project
        .find_by_original(&entry_path)
        .unwrap()
        .virtual_path
        .parent()
        .unwrap()
        .join("node_modules/@scope/workspace-vue");
    assert_eq!(
        fs::read_to_string(shadow_root.join("package.json")).unwrap(),
        manifest
    );
    assert!(shadow_root.join("src/Root.vue.ts").is_file());
    assert!(shadow_root.join("src/Root.d.vue.ts").is_file());
    let canonical_package_root = project.virtual_root().join("__vize_external__").join(
        package_root
            .strip_prefix(std::path::Path::new("/"))
            .unwrap_or(&package_root),
    );
    assert_eq!(
        fs::read_to_string(canonical_package_root.join("package.json")).unwrap(),
        manifest,
        "declaration roots must retain the same native package boundary"
    );
    assert!(canonical_package_root.join("src/Root.vue.ts").is_file());
    assert!(canonical_package_root.join("src/Root.d.vue.ts").is_file());
    assert!(
        project
            .find_by_diagnostic_virtual(&shadow_root.join("src/Root.vue.ts"))
            .is_some(),
        "byte-identical package companions may reuse the canonical source map"
    );
    assert!(
        project
            .find_by_diagnostic_virtual(&shadow_root.join("src/Root.d.vue.ts"))
            .is_none(),
        "synthetic forwarding companions must not reuse canonical coordinates"
    );
    assert!(
        !project
            .find_by_original(&entry_path)
            .unwrap()
            .content
            .contains("__vize")
    );

    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&package_root);
}

#[test]
fn warm_package_patch_work_is_independent_of_unrelated_route_count() {
    let small = measured_warm_package_patch(16);
    let large = measured_warm_package_patch(512);
    assert_eq!(small, large, "warm work must follow the affected closure");
    assert_eq!(small.source_nodes_rebuilt, 1);
    assert_eq!(small.dependency_nodes_reconciled, 1);
    assert_eq!(small.shadow_bindings_rebuilt, 1);
    assert_eq!(small.tree_entries_scanned, 0);
    assert!(!small.full_topology_rebuild);
    assert!(small.materialized_entries_considered <= 8, "{small:?}");
}

#[derive(Debug, Eq, PartialEq)]
struct WarmWork {
    source_nodes_rebuilt: usize,
    dependency_nodes_reconciled: usize,
    shadow_bindings_rebuilt: usize,
    materialized_entries_considered: usize,
    tree_entries_scanned: usize,
    full_topology_rebuild: bool,
}

fn measured_warm_package_patch(route_count: usize) -> WarmWork {
    let project_root = unique_case_dir(&format!("package-route-scale-{route_count}"));
    let _ = fs::remove_dir_all(&project_root);
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(
        project_root.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"Bundler","strict":true}}"#,
    )
    .unwrap();
    let importer = project_root.join("src/entry.ts");
    fs::write(&importer, "export const ready = true\n").unwrap();

    let mut bindings = Vec::with_capacity(route_count);
    for index in 0..route_count {
        let name = format!("route-{index}");
        let root = project_root.join("node_modules").join(&name);
        let source = root.join("src/Root.vue");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            root.join("package.json"),
            format!("{{\"name\":\"{name}\",\"exports\":\"./src/Root.vue\"}}\n"),
        )
        .unwrap();
        fs::write(&source, "<script setup lang=\"ts\">const n = 1</script>\n").unwrap();
        bindings.push(PackageRouteBinding {
            importer_path: importer.clone(),
            specifier: name.clone().into(),
            occurrence_mode: crate::PackageResolutionMode::Import,
            context: PackageResolutionContext::default(),
            route: Some(PackageRoute {
                source_paths: vec![source.clone()],
                dependency_paths: Vec::new(),
                source_targets: vec![crate::PackageRouteSource {
                    target_path: source.clone(),
                    source_path: source.clone(),
                    native_probe_path: source.with_extension("d.vue.ts"),
                }],
                package_root: root.clone(),
                package_link_root: root.clone(),
                manifest_path: root.join("package.json"),
                package_name: Some(name.into()),
                workspace_source: false,
                nested_routes: Vec::new(),
            }),
            invalidation_paths: vec![source, root.join("package.json")],
        });
    }

    let changed = project_root.join("node_modules/route-0/src/Root.vue");
    let mut project = VirtualProject::new(&project_root).unwrap();
    project.set_package_routes(bindings);
    project.register_path(&importer).unwrap();
    project.register_package_route_targets().unwrap();
    project.register_reachable_dependencies().unwrap();
    project.finalize_package_routes().unwrap();
    project.materialize().unwrap();
    project.capture_materialized_package_links();
    project.discard_incremental_materialization();

    fs::write(
        &changed,
        "<script setup lang=\"ts\">const n: number = 2</script>\n",
    )
    .unwrap();
    project.register_path(&changed).unwrap();
    let keys = project.package_route_keys_for_changes(std::slice::from_ref(&changed));
    project.refresh_package_route_keys(keys);
    project
        .register_reachable_dependencies_from(std::slice::from_ref(&changed))
        .unwrap();
    project.finalize_package_routes().unwrap();
    let materialized = project.materialize_incremental_delta().unwrap();
    let route_metrics = project.package_route_metrics();
    assert_eq!(route_metrics.last_refresh_total_routes, route_count as u64);
    assert_eq!(route_metrics.last_refresh_considered_routes, 1);

    let work = WarmWork {
        source_nodes_rebuilt: materialized.source_nodes_rebuilt,
        dependency_nodes_reconciled: materialized.dependency_nodes_reconciled,
        shadow_bindings_rebuilt: materialized.shadow_bindings_rebuilt,
        materialized_entries_considered: materialized.considered,
        tree_entries_scanned: 0,
        full_topology_rebuild: materialized.full_topology_rebuild,
    };
    let _ = fs::remove_dir_all(&project_root);
    work
}
