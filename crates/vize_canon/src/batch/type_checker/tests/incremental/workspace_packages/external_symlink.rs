use super::super::super::super::{create_project_case, resolve_test_tsgo_binary};
use crate::batch::{BatchTypeChecker, TypeChecker};
use crate::{PackageResolutionContext, PackageRouteBinding};

#[test]
#[cfg(unix)]
fn external_symlink_manifest_lifecycle_updates_every_dependent_and_reverse_index() {
    use std::os::unix::fs::symlink;

    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "incremental-external-package-symlink",
        &[
            (
                "src/first.ts",
                "import Widget from '@scope/workspace-vue'\ntype IsAny<T> = 0 extends 1 & T ? true : false\nconst typed: IsAny<typeof Widget> = false\nvoid typed\n",
            ),
            (
                "src/second.ts",
                "import Widget from '@scope/workspace-vue'\ntype IsAny<T> = 0 extends 1 & T ? true : false\nconst typed: IsAny<typeof Widget> = false\nvoid typed\n",
            ),
        ],
    );
    let external = tempfile::tempdir().unwrap();
    let first_root = external.path().join("first");
    let second_root = external.path().join("second");
    let clean = "<script setup lang=\"ts\">\nconst count: number = 1\n</script>\n<template>{{ count }}</template>\n";
    for (root, file) in [(&first_root, "Root.vue"), (&second_root, "Next.vue")] {
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("package.json"),
            format!("{{\"name\":\"@scope/workspace-vue\",\"exports\":\"./src/{file}\"}}\n"),
        )
        .unwrap();
        std::fs::write(root.join("src").join(file), clean).unwrap();
    }
    let link = project_root.join("node_modules/@scope/workspace-vue");
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    symlink(&first_root, &link).unwrap();

    let mut resolver = crate::PackageRouteResolver::default();
    let bindings = ["first.ts", "second.ts"]
        .into_iter()
        .map(|name| {
            let importer = project_root.join("src").join(name);
            let lookup = resolver.lookup(
                importer.parent().unwrap(),
                "@scope/workspace-vue",
                crate::PackageSourceOptions::default(),
            );
            let (route, invalidation_paths) = lookup.into_parts();
            PackageRouteBinding {
                importer_path: importer,
                specifier: "@scope/workspace-vue".into(),
                occurrence_mode: crate::PackageResolutionMode::Import,
                context: PackageResolutionContext::default(),
                route,
                invalidation_paths,
            }
        })
        .collect::<Vec<_>>();
    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.set_package_routes(bindings);
    checker.scan_project().unwrap();
    assert!(checker.check_project().unwrap().diagnostics.is_empty());

    let old_source = first_root.join("src/Root.vue");
    let renamed_source = first_root.join("src/Renamed.vue");
    std::fs::rename(&old_source, &renamed_source).unwrap();
    std::fs::write(
        first_root.join("package.json"),
        "{\"name\":\"@scope/workspace-vue\",\"exports\":\"./src/Renamed.vue\"}\n",
    )
    .unwrap();
    assert!(
        checker
            .check_incremental(&[
                old_source,
                renamed_source.clone(),
                first_root.join("package.json"),
            ])
            .unwrap()
            .diagnostics
            .is_empty()
    );
    assert_eq!(
        checker
            .package_route_metrics()
            .last_refresh_considered_routes,
        2
    );

    std::fs::write(&renamed_source, clean.replace("= 1", "= 'broken'")).unwrap();
    let broken = checker
        .check_incremental(std::slice::from_ref(&renamed_source))
        .unwrap();
    assert!(
        broken
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(2322))
    );
    std::fs::write(&renamed_source, clean).unwrap();

    let manifest = first_root.join("package.json");
    std::fs::remove_file(&manifest).unwrap();
    let deleted = checker
        .check_incremental(std::slice::from_ref(&manifest))
        .unwrap();
    assert!(
        deleted
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(2307))
    );
    std::fs::write(
        &manifest,
        "{\"name\":\"@scope/workspace-vue\",\"exports\":\"./src/Renamed.vue\"}\n",
    )
    .unwrap();
    assert!(
        checker
            .check_incremental(std::slice::from_ref(&manifest))
            .unwrap()
            .diagnostics
            .is_empty()
    );

    std::fs::remove_file(&link).unwrap();
    symlink(&second_root, &link).unwrap();
    assert!(
        checker
            .check_incremental(std::slice::from_ref(&link))
            .unwrap()
            .diagnostics
            .is_empty()
    );
    let retargeted = second_root.join("src/Next.vue");
    std::fs::write(&retargeted, clean.replace("= 1", "= 'broken'")).unwrap();
    let broken_after_retarget = checker
        .check_incremental(std::slice::from_ref(&retargeted))
        .unwrap();
    assert!(
        broken_after_retarget
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(2322))
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
