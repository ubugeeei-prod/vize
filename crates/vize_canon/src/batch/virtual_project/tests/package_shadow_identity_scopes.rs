//! Identity-scoped placement cases for #4153.
//!
//! These pin the two boundaries of the shared-scope rule: a scope never lands
//! where the real tree owns an install, and a package name that resolves to two
//! physical packages keeps the importer-scoped shadows #4002/#4137 require.

use std::fs;

use crate::batch::virtual_project::VirtualProject;
use crate::{PackageResolutionContext, PackageResolutionMode, PackageRoute, PackageRouteBinding};

use super::{IMPORTER_DIRS, build_monorepo, route};

#[test]
fn a_scope_never_lands_where_the_real_tree_owns_an_install() {
    // Importers spanning `apps/` and `packages/` share the mirror root, but the
    // workspace root owns a real `node_modules`; hoisting there would let the
    // package-link pass bridge the shadow into the real install, so the scope
    // splits by top-level directory instead.
    let monorepo = build_monorepo("shadow-fan-out-install", IMPORTER_DIRS);
    fs::create_dir_all(monorepo.root.join("node_modules")).unwrap();
    let host_dir = monorepo.root.join("packages/host/src");
    fs::create_dir_all(&host_dir).unwrap();
    let host_path = host_dir.join("Host.vue");
    fs::write(
        &host_path,
        "<script setup lang=\"ts\">import { Widget0 } from \"@w/ui\";\nvoid Widget0;\n</script>\n",
    )
    .unwrap();

    let mut project = VirtualProject::new(&monorepo.root).unwrap();
    let shared_route = route(&monorepo);
    let mut importer_paths = monorepo.importer_paths.clone();
    importer_paths.push(host_path.clone());
    project.set_package_routes(
        importer_paths
            .iter()
            .map(|importer_path| PackageRouteBinding {
                importer_path: importer_path.clone(),
                specifier: "@w/ui".into(),
                occurrence_mode: PackageResolutionMode::Import,
                context: PackageResolutionContext::default(),
                route: Some(shared_route.clone()),
                invalidation_paths: vec![monorepo.manifest_path.clone()],
            }),
    );
    let mut roots = importer_paths.clone();
    roots.extend(monorepo.component_paths.iter().cloned());
    project.set_declaration_roots(&roots);
    project.register_paths(&roots).unwrap();
    project.register_package_route_targets().unwrap();
    project.finalize_package_routes().unwrap();

    assert_eq!(
        project.topology_shadow_manifest_scopes(),
        [
            "",
            "apps/node_modules/@w/ui",
            // The `packages/` group has one importing directory, so it keeps
            // the importer-scoped placement it already had.
            "packages/host/src/node_modules/@w/ui",
            "packages/ui",
        ],
    );
}

#[test]
fn divergent_identities_keep_their_own_importer_scoped_shadows() {
    let monorepo = build_monorepo("shadow-fan-out-divergent", 2);
    let forked_root = monorepo.root.join("vendor/ui");
    fs::create_dir_all(forked_root.join("src")).unwrap();
    let forked_manifest = forked_root.join("package.json");
    fs::write(
        &forked_manifest,
        r#"{"name":"@w/ui","version":"2.0.0","types":"./src/index.ts"}"#,
    )
    .unwrap();
    let forked_component = forked_root.join("src/Widget0.vue");
    fs::write(
        &forked_component,
        "<script setup lang=\"ts\">defineProps<{ label0: string }>()</script>\n",
    )
    .unwrap();

    let mut project = VirtualProject::new(&monorepo.root).unwrap();
    let shared_route = route(&monorepo);
    let forked_route = PackageRoute {
        source_paths: vec![forked_component.clone()],
        dependency_paths: Vec::new(),
        source_targets: vec![crate::PackageRouteSource {
            target_path: forked_component.clone(),
            source_path: forked_component.clone(),
            native_probe_path: forked_component.with_extension("d.vue.ts"),
        }],
        package_root: forked_root.clone(),
        package_link_root: forked_root.clone(),
        manifest_path: forked_manifest.clone(),
        package_name: Some("@w/ui".into()),
        workspace_source: true,
        nested_routes: Vec::new(),
    };
    project.set_package_routes([
        PackageRouteBinding {
            importer_path: monorepo.importer_paths[0].clone(),
            specifier: "@w/ui".into(),
            occurrence_mode: PackageResolutionMode::Import,
            context: PackageResolutionContext::default(),
            route: Some(shared_route),
            invalidation_paths: vec![monorepo.manifest_path.clone()],
        },
        PackageRouteBinding {
            importer_path: monorepo.importer_paths[1].clone(),
            specifier: "@w/ui".into(),
            occurrence_mode: PackageResolutionMode::Import,
            context: PackageResolutionContext::default(),
            route: Some(forked_route),
            invalidation_paths: vec![forked_manifest.clone()],
        },
    ]);
    let mut roots = monorepo.importer_paths.clone();
    roots.extend(monorepo.component_paths.iter().cloned());
    roots.push(forked_component.clone());
    project.set_declaration_roots(&roots);
    project.register_paths(&roots).unwrap();
    project.register_package_route_targets().unwrap();
    project.finalize_package_routes().unwrap();

    for importer_path in &monorepo.importer_paths {
        let importer_dir = project
            .find_by_original(importer_path)
            .unwrap()
            .virtual_path
            .parent()
            .unwrap()
            .to_path_buf();
        assert!(
            project
                .topology_shadow_paths()
                .iter()
                .any(|path| path.starts_with(importer_dir.join("node_modules/@w/ui"))),
            "a name with two identities must keep importer-scoped shadows"
        );
    }
}
