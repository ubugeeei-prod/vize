//! Reduced reproduction of the large-batch work amplification behind #4153.
//!
//! Vue Vben Admin exhausted its 600s Matrix budget because Canon anchored one
//! package shadow scope at every *importing directory*: a workspace package's
//! sources were materialized, and listed as native program roots, once per
//! directory that imported the package. These cases pin the exact counters
//! instead of wall-clock time, so they reproduce the amplification
//! deterministically on any machine.

use std::fs;
use std::path::{Path, PathBuf};

use crate::batch::virtual_project::VirtualProject;
use crate::{PackageResolutionContext, PackageResolutionMode, PackageRoute, PackageRouteBinding};

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(vize_carton::cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

/// Authored `.vue` sources inside the shared workspace package.
const PACKAGE_SOURCES: usize = 4;
/// Directories importing that package with the same bare specifier.
const IMPORTER_DIRS: usize = 6;

struct Monorepo {
    root: PathBuf,
    package_root: PathBuf,
    manifest_path: PathBuf,
    component_paths: Vec<PathBuf>,
    importer_paths: Vec<PathBuf>,
}

fn build_monorepo(name: &str, importer_dirs: usize) -> Monorepo {
    let root = unique_case_dir(name);
    let _ = fs::remove_dir_all(&root);
    let package_root = root.join("packages/ui");
    fs::create_dir_all(package_root.join("src")).unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"moduleResolution":"Bundler","strict":true}}"#,
    )
    .unwrap();
    // The workspace root manifest is the importers' resolution scope; without
    // it the nearest `package.json` above the fixture would be, and the mirror
    // would carry an unrelated external scope entry.
    fs::write(root.join("package.json"), r#"{"name":"w","private":true}"#).unwrap();
    let manifest_path = package_root.join("package.json");
    fs::write(
        &manifest_path,
        r#"{"name":"@w/ui","version":"1.0.0","types":"./src/index.ts"}"#,
    )
    .unwrap();

    let mut component_paths = Vec::new();
    for index in 0..PACKAGE_SOURCES {
        let component_path =
            package_root.join(vize_carton::cstr!("src/Widget{index}.vue").as_str());
        fs::write(
            &component_path,
            vize_carton::cstr!(
                "<script setup lang=\"ts\">defineProps<{{ label{index}: string }}>()</script>\n"
            )
            .as_str(),
        )
        .unwrap();
        component_paths.push(component_path);
    }
    let index_path = package_root.join("src/index.ts");
    let mut barrel = vize_carton::String::default();
    for index in 0..PACKAGE_SOURCES {
        barrel.push_str(
            vize_carton::cstr!(
                "export {{ default as Widget{index} }} from \"./Widget{index}.vue\";\n"
            )
            .as_str(),
        );
    }
    fs::write(&index_path, barrel.as_str()).unwrap();
    component_paths.push(index_path);

    let mut importer_paths = Vec::new();
    for app in 0..importer_dirs {
        let importer_dir = root.join(vize_carton::cstr!("apps/app{app}/src").as_str());
        fs::create_dir_all(&importer_dir).unwrap();
        let importer_path = importer_dir.join("View.vue");
        fs::write(
            &importer_path,
            "<script setup lang=\"ts\">import { Widget0 } from \"@w/ui\";\nvoid Widget0;\n</script>\n",
        )
        .unwrap();
        importer_paths.push(importer_path);
    }

    Monorepo {
        root,
        package_root,
        manifest_path,
        component_paths,
        importer_paths,
    }
}

fn route(monorepo: &Monorepo) -> PackageRoute {
    PackageRoute {
        source_paths: monorepo.component_paths.clone(),
        dependency_paths: Vec::new(),
        source_targets: monorepo
            .component_paths
            .iter()
            .map(|source_path| crate::PackageRouteSource {
                target_path: source_path.clone(),
                source_path: source_path.clone(),
                native_probe_path: native_probe(source_path),
            })
            .collect(),
        package_root: monorepo.package_root.clone(),
        package_link_root: monorepo.package_root.clone(),
        manifest_path: monorepo.manifest_path.clone(),
        package_name: Some("@w/ui".into()),
        workspace_source: true,
        nested_routes: Vec::new(),
    }
}

fn native_probe(source_path: &Path) -> PathBuf {
    if source_path.extension().is_some_and(|ext| ext == "vue") {
        source_path.with_extension("d.vue.ts")
    } else {
        source_path.to_path_buf()
    }
}

fn scanned_project(monorepo: &Monorepo) -> VirtualProject {
    let mut project = VirtualProject::new(&monorepo.root).unwrap();
    let shared_route = route(monorepo);
    project.set_package_routes(monorepo.importer_paths.iter().map(|importer_path| {
        PackageRouteBinding {
            importer_path: importer_path.clone(),
            specifier: "@w/ui".into(),
            occurrence_mode: PackageResolutionMode::Import,
            context: PackageResolutionContext::default(),
            route: Some(shared_route.clone()),
            invalidation_paths: vec![monorepo.manifest_path.clone()],
        }
    }));
    let mut roots = monorepo.importer_paths.clone();
    roots.extend(monorepo.component_paths.iter().cloned());
    project.set_declaration_roots(&roots);
    project.register_paths(&roots).unwrap();
    project.register_package_route_targets().unwrap();
    project.finalize_package_routes().unwrap();
    project
}

#[test]
fn one_physical_package_materializes_exactly_one_shadow_scope() {
    let monorepo = build_monorepo("shadow-fan-out", IMPORTER_DIRS);
    let project = scanned_project(&monorepo);
    let metrics = project.topology_metrics();

    // 6 importers + 4 components + the barrel.
    assert_eq!(metrics.scan_roots, IMPORTER_DIRS + PACKAGE_SOURCES + 1);
    assert_eq!(metrics.virtual_files, IMPORTER_DIRS + PACKAGE_SOURCES + 1);
    assert_eq!(metrics.package_route_bindings, IMPORTER_DIRS);
    assert_eq!(metrics.resolved_package_routes, IMPORTER_DIRS);

    // One scope for the canonical package root and one shared scope for all
    // six importers. Anchoring per importing directory produced one scope per
    // `apps/appN/src` instead.
    assert_eq!(
        project.topology_shadow_manifest_scopes(),
        // The mirror root is the importers' package scope; the other two are
        // the canonical package root and the one shared importer scope.
        ["", "apps/node_modules/@w/ui", "packages/ui"],
    );
    assert_eq!(metrics.package_shadow_scopes, 3);
    // Each scope holds `Widget*.vue.ts` + `Widget*.d.vue.ts` per component plus
    // the barrel: 2 * (2 * PACKAGE_SOURCES + 1).
    assert_eq!(
        metrics.package_shadow_files,
        2 * (2 * PACKAGE_SOURCES + 1),
        "one physical package must not be copied once per importing directory"
    );
}

#[test]
fn native_program_lists_each_authored_source_a_bounded_number_of_times() {
    let monorepo = build_monorepo("shadow-fan-out-program", IMPORTER_DIRS);
    let project = scanned_project(&monorepo);
    let metrics = project.topology_metrics();

    // Program roots: every registered virtual file, plus both shadow scopes'
    // copies of the package's declaration roots, plus the ambient stubs.
    assert_eq!(
        project.topology_program_files(),
        [
            "__vize_helpers.d.ts",
            "__vize_vue_modules.d.ts",
            "apps/app0/src/View.vue.ts",
            "apps/app1/src/View.vue.ts",
            "apps/app2/src/View.vue.ts",
            "apps/app3/src/View.vue.ts",
            "apps/app4/src/View.vue.ts",
            "apps/app5/src/View.vue.ts",
            "apps/node_modules/@w/ui/src/Widget0.d.vue.ts",
            "apps/node_modules/@w/ui/src/Widget0.vue.ts",
            "apps/node_modules/@w/ui/src/Widget1.d.vue.ts",
            "apps/node_modules/@w/ui/src/Widget1.vue.ts",
            "apps/node_modules/@w/ui/src/Widget2.d.vue.ts",
            "apps/node_modules/@w/ui/src/Widget2.vue.ts",
            "apps/node_modules/@w/ui/src/Widget3.d.vue.ts",
            "apps/node_modules/@w/ui/src/Widget3.vue.ts",
            "apps/node_modules/@w/ui/src/index.ts",
            "packages/ui/src/Widget0.d.vue.ts",
            "packages/ui/src/Widget0.vue.ts",
            "packages/ui/src/Widget1.d.vue.ts",
            "packages/ui/src/Widget1.vue.ts",
            "packages/ui/src/Widget2.d.vue.ts",
            "packages/ui/src/Widget2.vue.ts",
            "packages/ui/src/Widget3.d.vue.ts",
            "packages/ui/src/Widget3.vue.ts",
            "packages/ui/src/index.ts",
        ],
    );
    assert!(
        metrics.program_files_per_virtual_file() <= 4.0,
        "program grew to {} roots for {} virtual files",
        metrics.native_program_files,
        metrics.virtual_files
    );
}

#[test]
fn the_program_does_not_grow_when_more_directories_import_the_same_package() {
    let narrow = scanned_project(&build_monorepo("shadow-fan-out-narrow", 2));
    let wide = scanned_project(&build_monorepo("shadow-fan-out-wide", 2 * IMPORTER_DIRS));
    let narrow_metrics = narrow.topology_metrics();
    let wide_metrics = wide.topology_metrics();

    assert_eq!(
        narrow_metrics.package_shadow_files, wide_metrics.package_shadow_files,
        "shadow materialization must not scale with importing directories"
    );
    assert_eq!(
        narrow_metrics.package_shadow_scopes,
        wide_metrics.package_shadow_scopes
    );
    assert_eq!(
        wide_metrics.native_program_files - narrow_metrics.native_program_files,
        wide_metrics.virtual_files - narrow_metrics.virtual_files,
        "extra importers may only add their own program roots"
    );
}

#[path = "package_shadow_identity_scopes.rs"]
mod identity_scopes;
