use super::{BatchTypeChecker, TypeChecker, create_project_case, resolve_test_tsgo_binary};
use crate::{PackageResolutionContext, PackageRouteBinding, PackageRouteResolver};

#[test]
fn cold_negative_package_route_activates_after_manifest_creation() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "incremental-negative-package-create",
        &[(
            "src/entry.ts",
            "import Widget from 'late-ui'\nvoid Widget\n",
        )],
    );
    let importer = project_root.join("src/entry.ts");
    let package = project_root.join("node_modules/late-ui");
    let manifest = package.join("package.json");
    let component = package.join("src/Widget.vue");
    let mut resolver = PackageRouteResolver::default();
    let lookup = resolver.lookup(
        importer.parent().unwrap(),
        "late-ui",
        crate::PackageSourceOptions::default(),
    );
    let (route, invalidation_paths) = lookup.into_parts();
    assert!(route.is_none(), "the fixture must begin as a cold negative");

    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.set_package_route_resolver(resolver);
    checker.set_package_routes([PackageRouteBinding {
        importer_path: importer.clone(),
        specifier: "late-ui".into(),
        occurrence_mode: crate::PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route,
        invalidation_paths,
    }]);
    checker.scan_project().unwrap();
    let missing = checker
        .check_incremental(std::slice::from_ref(&importer))
        .unwrap();
    assert!(
        missing
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(2307)),
        "the initial session did not observe the missing package: {:#?}",
        missing.diagnostics
    );

    std::fs::create_dir_all(component.parent().unwrap()).unwrap();
    std::fs::write(
        &manifest,
        "{\"name\":\"late-ui\",\"exports\":\"./src/Widget.vue\"}\n",
    )
    .unwrap();
    std::fs::write(
        &component,
        "<script setup lang=\"ts\">defineProps<{ count: number }>()</script>\n",
    )
    .unwrap();
    let created = checker
        .check_incremental(std::slice::from_ref(&manifest))
        .unwrap();
    assert!(
        created
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(2307)),
        "package creation retained TS2307: {:#?}",
        created.diagnostics
    );
    let metrics = checker.package_route_metrics();
    assert_eq!(metrics.last_refresh_total_routes, 1);
    assert_eq!(metrics.last_refresh_considered_routes, 1);
    assert_eq!(metrics.last_refresh_affected_routes, 1);
    assert!(checker.incremental_metrics().last_session_reused);

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn importer_edits_add_retarget_and_remove_package_routes_without_restart() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "incremental-package-import-edit",
        &[
            ("src/entry.ts", "export const ready = true\n"),
            ("src/unrelated.ts", "export const revision = 0\n"),
        ],
    );
    for (package, prop, ty) in [
        ("first-ui", "firstOnly", "string"),
        ("second-ui", "secondOnly", "number"),
    ] {
        let root = project_root.join("node_modules").join(package);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("package.json"),
            format!("{{\"name\":\"{package}\",\"exports\":\"./src/Widget.vue\"}}\n"),
        )
        .unwrap();
        std::fs::write(
            root.join("src/Widget.vue"),
            format!("<script setup lang=\"ts\">defineProps<{{ {prop}: {ty} }}>()</script>\n"),
        )
        .unwrap();
    }
    let importer = project_root.join("src/entry.ts");
    let unrelated = project_root.join("src/unrelated.ts");
    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.scan_project().unwrap();

    std::fs::write(
        &importer,
        "import Widget from 'first-ui'\ntype Props = InstanceType<typeof Widget>['$props']\nexport const props: Props = { firstOnly: 'added' }\n",
    )
    .unwrap();
    let added = checker
        .check_incremental(std::slice::from_ref(&importer))
        .unwrap();
    assert!(added.diagnostics.is_empty(), "{:#?}", added.diagnostics);
    std::fs::write(&unrelated, "export const revision = 1\n").unwrap();
    let after_add = checker
        .check_incremental(std::slice::from_ref(&unrelated))
        .unwrap();
    assert!(
        after_add.diagnostics.is_empty(),
        "new route was not persisted: {:#?}",
        after_add.diagnostics
    );

    std::fs::write(
        &importer,
        "import Widget from 'second-ui'\ntype Props = InstanceType<typeof Widget>['$props']\nexport const props: Props = { secondOnly: 2 }\n",
    )
    .unwrap();
    let retargeted = checker
        .check_incremental(std::slice::from_ref(&importer))
        .unwrap();
    assert!(
        retargeted.diagnostics.is_empty(),
        "{:#?}",
        retargeted.diagnostics
    );
    std::fs::write(&unrelated, "export const revision = 2\n").unwrap();
    let after_retarget = checker
        .check_incremental(std::slice::from_ref(&unrelated))
        .unwrap();
    assert!(
        after_retarget.diagnostics.is_empty(),
        "retargeted route was not persisted: {:#?}",
        after_retarget.diagnostics
    );

    let stale_source = project_root.join("node_modules/first-ui/src/Widget.vue");
    std::fs::write(
        &stale_source,
        "<script setup lang=\"ts\">const stale: number = 'must stay unreachable'</script>\n",
    )
    .unwrap();
    std::fs::write(&unrelated, "export const revision = 3\n").unwrap();
    let after_stale_source_edit = checker
        .check_incremental(std::slice::from_ref(&unrelated))
        .unwrap();
    assert!(
        after_stale_source_edit
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.file != stale_source),
        "the old package target stayed a program root after retarget: {:#?}",
        after_stale_source_edit.diagnostics
    );

    std::fs::write(&importer, "export const ready = true\n").unwrap();
    let removed = checker
        .check_incremental(std::slice::from_ref(&importer))
        .unwrap();
    assert!(removed.diagnostics.is_empty(), "{:#?}", removed.diagnostics);
    assert_eq!(checker.incremental_metrics().session_starts, 1);
    assert_eq!(checker.incremental_metrics().session_reuses, 5);

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn newly_discovered_external_alias_dependency_remains_a_known_watcher_input() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "incremental-new-external-alias",
        &[
            ("src/entry.ts", "export const ready = true\n"),
            ("src/unrelated.ts", "export const revision = 0\n"),
        ],
    );
    let external = tempfile::tempdir().unwrap();
    let component = external.path().join("Widget.vue");
    std::fs::write(
        &component,
        "<script setup lang=\"ts\">const count: number = 1</script>\n",
    )
    .unwrap();
    let canonical_component = component.canonicalize().unwrap();
    let external_pattern = external
        .path()
        .canonicalize()
        .unwrap()
        .join("*")
        .to_string_lossy()
        .replace('\\', "/");
    std::fs::write(
        project_root.join("tsconfig.json"),
        format!(
            "{{\"compilerOptions\":{{\"strict\":true,\"moduleResolution\":\"bundler\",\"allowArbitraryExtensions\":true,\"baseUrl\":\".\",\"paths\":{{\"@external/*\":[{external_pattern:?}]}}}}}}\n"
        ),
    )
    .unwrap();

    let importer = project_root.join("src/entry.ts");
    let unrelated = project_root.join("src/unrelated.ts");
    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.scan_project().unwrap();
    std::fs::write(
        &importer,
        "import Widget from '@external/Widget'\nvoid Widget\n",
    )
    .unwrap();
    let discovered = checker
        .check_incremental(std::slice::from_ref(&importer))
        .unwrap();
    assert!(
        discovered.diagnostics.is_empty(),
        "{:#?}",
        discovered.diagnostics
    );

    std::fs::write(
        &component,
        "<script setup lang=\"ts\">const count: number = 'broken'</script>\n",
    )
    .unwrap();
    let external_edit = checker
        .check_incremental(std::slice::from_ref(&component))
        .expect("the newly discovered external source must remain a known watcher input");
    assert!(
        external_edit.diagnostics.iter().any(|diagnostic| {
            diagnostic.file == canonical_component && diagnostic.code == Some(2322)
        }),
        "external dependency edit was not checked: {:#?}",
        external_edit.diagnostics
    );

    std::fs::write(&unrelated, "export const revision = 1\n").unwrap();
    let persisted = checker
        .check_incremental(std::slice::from_ref(&unrelated))
        .unwrap();
    assert!(
        persisted.diagnostics.iter().any(|diagnostic| {
            diagnostic.file == canonical_component && diagnostic.code == Some(2322)
        }),
        "external dependency membership was not committed: {:#?}",
        persisted.diagnostics
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
