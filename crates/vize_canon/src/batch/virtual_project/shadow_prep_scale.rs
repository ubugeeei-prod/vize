//! Package shadow preparation must not canonicalize or test membership once
//! per file/package pair (#10359).

use std::fs;
use std::path::PathBuf;

use vize_carton::FxHashSet;

use super::VirtualProject;
use super::canonical_paths::{prep_counters, reset_prep_counters};
use crate::{PackageResolutionMode, PackageRoute, PackageRouteBinding, PackageRouteSource};

const PACKAGES: usize = 8;
const EXTRA_FILES: usize = 24;

struct PackageFixture {
    root: PathBuf,
    manifest: PathBuf,
    sources: Vec<PathBuf>,
    importer: PathBuf,
}

fn write_workspace() -> (tempfile::TempDir, Vec<PackageFixture>, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.json"), r#"{"name":"ws","private":true}"#).unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"moduleResolution":"Bundler","strict":true}}"#,
    )
    .unwrap();
    let app = root.join("src/App.vue");
    fs::write(
        &app,
        "<script setup lang=\"ts\">import { value0 } from \"@ws/pkg-0\";\nvoid value0;\n</script>\n",
    )
    .unwrap();

    let mut packages = Vec::with_capacity(PACKAGES);
    for index in 0..PACKAGES {
        let package_root = root.join(vize_carton::cstr!("packages/pkg-{index}").as_str());
        fs::create_dir_all(package_root.join("src")).unwrap();
        fs::create_dir_all(package_root.join("node_modules")).unwrap();
        let manifest = package_root.join("package.json");
        fs::write(
            &manifest,
            vize_carton::cstr!(
                "{{\"name\":\"@ws/pkg-{index}\",\"version\":\"1.0.0\",\"type\":\"module\",\"exports\":{{\".\":\"./src/index.ts\"}}}}"
            )
            .as_str(),
        )
        .unwrap();
        let importer = package_root.join("src/index.ts");
        let mut source = vize_carton::String::default();
        for dependency in 0..PACKAGES {
            if dependency == index {
                continue;
            }
            source.push_str(
                vize_carton::cstr!(
                    "import {{ value{dependency} }} from \"@ws/pkg-{dependency}\";\n"
                )
                .as_str(),
            );
        }
        source.push_str(vize_carton::cstr!("export const value{index} = {index};\n").as_str());
        fs::write(&importer, source.as_str()).unwrap();
        let widget = package_root.join("src/Widget.vue");
        fs::write(
            &widget,
            "<script setup lang=\"ts\">defineProps<{ label: string }>();</script>\n<template><span>{{ label }}</span></template>\n",
        )
        .unwrap();
        let mut sources = vec![importer.clone(), widget];
        for file in 0..EXTRA_FILES {
            let path = package_root.join(vize_carton::cstr!("src/file{file}.ts").as_str());
            fs::write(
                &path,
                vize_carton::cstr!("export const file{index}_{file} = {file};\n").as_str(),
            )
            .unwrap();
            sources.push(path);
        }
        packages.push(PackageFixture {
            root: package_root,
            manifest,
            sources,
            importer,
        });
    }
    (temp, packages, app)
}

fn route(package: &PackageFixture, index: usize) -> PackageRoute {
    PackageRoute {
        source_paths: package.sources.clone(),
        dependency_paths: Vec::new(),
        source_targets: package
            .sources
            .iter()
            .map(|source_path| PackageRouteSource {
                target_path: source_path.clone(),
                source_path: source_path.clone(),
                native_probe_path: if source_path.extension().is_some_and(|ext| ext == "vue") {
                    source_path.with_extension("d.vue.ts")
                } else {
                    source_path.clone()
                },
            })
            .collect(),
        package_root: package.root.clone(),
        package_link_root: package.root.clone(),
        manifest_path: package.manifest.clone(),
        package_name: Some(vize_carton::cstr!("@ws/pkg-{index}").as_str().into()),
        workspace_source: true,
        nested_routes: Vec::new(),
    }
}

#[test]
fn many_files_do_not_canonicalize_or_probe_once_per_package_pair() {
    let (_temp, packages, app) = write_workspace();
    let mut project = VirtualProject::new(packages[0].root.parent().unwrap().parent().unwrap())
        .expect("workspace root");
    let routes: Vec<PackageRoute> = packages
        .iter()
        .enumerate()
        .map(|(index, package)| route(package, index))
        .collect();
    let mut bindings = Vec::new();
    for (importer_index, package) in packages.iter().enumerate() {
        for (dependency, route) in routes.iter().enumerate() {
            if dependency == importer_index {
                continue;
            }
            bindings.push(PackageRouteBinding {
                importer_path: package.importer.clone(),
                specifier: vize_carton::cstr!("@ws/pkg-{dependency}").as_str().into(),
                occurrence_mode: PackageResolutionMode::Import,
                context: crate::PackageResolutionContext::default(),
                route: Some(route.clone()),
                invalidation_paths: vec![package.manifest.clone()],
                shared_invalidation_paths: Vec::new(),
            });
        }
    }
    bindings.push(PackageRouteBinding {
        importer_path: app.clone(),
        specifier: "@ws/pkg-0".into(),
        occurrence_mode: PackageResolutionMode::Import,
        context: crate::PackageResolutionContext::default(),
        route: Some(routes[0].clone()),
        invalidation_paths: vec![packages[0].manifest.clone()],
        shared_invalidation_paths: Vec::new(),
    });
    project.set_package_routes(bindings);
    let mut registered = vec![app];
    for package in &packages {
        registered.extend(package.sources.iter().cloned());
    }
    project.set_declaration_roots(&registered);
    project.register_paths(&registered).unwrap();
    project.register_package_route_targets().unwrap();

    reset_prep_counters();
    project.finalize_package_routes().unwrap();
    let _ = project.validate_workspace_alias_targets(&FxHashSet::default());
    let (canonicalizations, membership_probes) = prep_counters();
    let files = project.expected_materialized_files().len();
    let pairs = files * PACKAGES;
    println!(
        "canonicalizations {canonicalizations} membership_probes {membership_probes} files {files} pairs {pairs}"
    );
    assert!(
        files > PACKAGES * EXTRA_FILES,
        "expected many materialized files, got {files}"
    );
    assert!(
        canonicalizations > 0 && canonicalizations < pairs / 4,
        "canonicalizations {canonicalizations} probes {membership_probes} files {files} pairs {pairs}"
    );
    assert!(
        membership_probes > 0 && membership_probes < pairs / 4,
        "membership probes {membership_probes} canonicalizations {canonicalizations} files {files} pairs {pairs}"
    );
}
