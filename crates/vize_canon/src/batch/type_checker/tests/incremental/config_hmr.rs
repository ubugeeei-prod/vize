use std::path::Path;

use super::super::super::{BatchTypeChecker, create_project_case, resolve_test_tsgo_binary};
use crate::batch::TypeChecker;

#[test]
fn same_mtime_base_config_condition_flip_rebuilds_package_identity_persistently() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "incremental-config-package-condition",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
import Widget from '@scope/ui'
type Props = InstanceType<typeof Widget>['$props']
const props: Props = { editorOnly: 'ok' }
void props
</script>
"#,
        )],
    );
    let app = project_root.join("src/App.vue");
    let base = project_root.join("tsconfig.base.json");
    write(
        &base,
        r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"bundler","allowArbitraryExtensions":true,"customConditions":["editor"]}}"#,
    );
    write(
        &project_root.join("tsconfig.json"),
        r#"{"extends":"./tsconfig.base.json","include":["src/**/*"]}"#,
    );
    let package = project_root.join("node_modules/@scope/ui");
    write(
        &package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{".":{"editor":"./Editor.vue","legacy":"./Legacy.vue","default":"./Editor.vue"}}}"#,
    );
    write(
        &package.join("Editor.vue"),
        "<script setup lang=\"ts\">defineProps<{ editorOnly: string }>()</script>\n",
    );
    write(
        &package.join("Legacy.vue"),
        "<script setup lang=\"ts\">defineProps<{ legacyOnly: boolean }>()</script>\n",
    );

    let mut resolver = crate::PackageRouteResolver::default();
    let lookup = resolver.lookup(
        app.parent().unwrap(),
        "@scope/ui",
        crate::PackageSourceOptions::default(),
    );
    let (route, invalidation_paths) = lookup.into_parts();
    assert!(
        route.is_some(),
        "package fixture must resolve before the flip"
    );
    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.set_package_route_resolver(resolver);
    checker.set_package_routes([crate::PackageRouteBinding {
        importer_path: app.clone(),
        specifier: "@scope/ui".into(),
        occurrence_mode: crate::PackageResolutionMode::Contextual,
        context: crate::PackageResolutionContext::default(),
        route,
        invalidation_paths,
    }]);
    checker.scan_project().unwrap();

    let initial = checker
        .check_incremental(std::slice::from_ref(&app))
        .unwrap();
    assert_no_code(&initial, 2353, "editor condition must be authoritative");

    let modified = std::fs::metadata(&base).unwrap().modified().unwrap();
    let legacy = std::fs::read_to_string(&base)
        .unwrap()
        .replace("editor", "legacy");
    std::fs::write(&base, legacy).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&base)
        .unwrap()
        .set_modified(modified)
        .unwrap();

    let flipped = checker
        .check_incremental(std::slice::from_ref(&base))
        .unwrap();
    assert_has_code(&flipped, &app, 2353, "legacy condition must replace editor");
    assert!(checker.incremental_metrics().last_full_rebuild);

    let persisted = checker
        .check_incremental(std::slice::from_ref(&app))
        .unwrap();
    assert_has_code(
        &persisted,
        &app,
        2353,
        "an unrelated notification must not restore the stale route",
    );
    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn base_config_paths_retarget_replaces_external_dependency_ownership() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "incremental-config-paths-retarget",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
import type { Contract } from '@contract'
const value: Contract = 1
void value
</script>
"#,
        )],
    );
    let app = project_root.join("src/App.vue");
    let contracts = project_root.parent().unwrap().join(format!(
        "{}-contracts",
        project_root.file_name().unwrap().to_string_lossy()
    ));
    let alpha = contracts.join("alpha.ts");
    let bravo = contracts.join("bravo.ts");
    write(&alpha, "export type Contract = number\n");
    write(&bravo, "export type Contract = string\n");
    let alpha_target = relative_target(&project_root, &alpha);
    let bravo_target = relative_target(&project_root, &bravo);
    assert_eq!(alpha_target.len(), bravo_target.len());
    let base = project_root.join("tsconfig.base.json");
    write(
        &base,
        &format!(
            r#"{{"compilerOptions":{{"strict":true,"module":"ESNext","moduleResolution":"bundler","baseUrl":".","paths":{{"@contract":[{alpha_target:?}]}}}}}}"#
        ),
    );
    write(
        &project_root.join("tsconfig.json"),
        r#"{"extends":"./tsconfig.base.json","include":["src/**/*"]}"#,
    );

    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.scan_project().unwrap();
    let initial = checker
        .check_incremental(std::slice::from_ref(&app))
        .unwrap();
    assert_no_code(&initial, 2322, "alpha contract must be selected");

    let modified = std::fs::metadata(&base).unwrap().modified().unwrap();
    let retargeted = std::fs::read_to_string(&base)
        .unwrap()
        .replace(&alpha_target, &bravo_target);
    std::fs::write(&base, retargeted).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&base)
        .unwrap()
        .set_modified(modified)
        .unwrap();

    let flipped = checker
        .check_incremental(std::slice::from_ref(&base))
        .unwrap();
    assert_has_code(&flipped, &app, 2322, "bravo contract must be selected");
    let originals = checker
        .virtual_files()
        .into_iter()
        .map(|file| file.original_path.clone())
        .collect::<Vec<_>>();
    assert!(originals.contains(&bravo.canonicalize().unwrap()));
    assert!(!originals.contains(&alpha.canonicalize().unwrap()));

    let persisted = checker
        .check_incremental(std::slice::from_ref(&app))
        .unwrap();
    assert_has_code(
        &persisted,
        &app,
        2322,
        "retargeted dependency ownership must persist",
    );
    let _ = std::fs::remove_dir_all(&project_root);
    let _ = std::fs::remove_dir_all(&contracts);
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn relative_target(project_root: &Path, path: &Path) -> String {
    let parent = project_root.parent().unwrap();
    let file = path.file_name().unwrap().to_string_lossy();
    format!(
        "../{}/{file}",
        path.parent()
            .unwrap()
            .strip_prefix(parent)
            .unwrap()
            .display()
    )
}

fn assert_has_code(result: &crate::batch::TypeCheckResult, file: &Path, code: u32, reason: &str) {
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.file == file && diagnostic.code == Some(code)),
        "{reason}: {:#?}",
        result.diagnostics
    );
}

fn assert_no_code(result: &crate::batch::TypeCheckResult, code: u32, reason: &str) {
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(code)),
        "{reason}: {:#?}",
        result.diagnostics
    );
}
