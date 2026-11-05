use super::*;

#[test]
fn an_imported_overlay_edit_refreshes_affected_open_roots_for_native_navigation() {
    const CONTRACT: &str = "<script lang='ts'>export interface Contract { n: number }</script><script setup lang='ts'>const token = 1;</script><template>{{ token }}</template>";
    const BETA: &str = "import type { Contract } from './Host.vue'; export class Beta implements Contract { n = 1; }";
    let workspace = Workspace::new(true);
    std::fs::write(&workspace.host, CONTRACT).unwrap();
    let beta = workspace.other.with_file_name("Beta.tsx");
    std::fs::write(&beta, BETA).unwrap();
    let mut overlays =
        FxHashMap::from_iter([(workspace.host.clone(), CONTRACT), (beta.clone(), BETA)]);
    drop(workspace.open(&workspace.host, CONTRACT, &overlays, &[]));
    let implementation = workspace.open(&beta, BETA, &overlays, &[]);
    let beta_path = implementation.mirror_virtual_path(&beta).unwrap();
    drop(implementation);
    let changed = CONTRACT.replace("token = 1", "token = 2");
    overlays.insert(workspace.host.clone(), &changed);
    let current = workspace.open(&workspace.host, &changed, &overlays, &[]);
    assert_eq!(
        current
            .source_catalog
            .get(&beta_path)
            .unwrap()
            .source
            .as_str(),
        BETA
    );
    assert!(beta_path.exists());
    let expected = native_documents(&current);
    assert!(
        expected
            .iter()
            .any(|(uri, _)| uri == &crate::file_uri::path_to_file_uri(&beta_path))
    );
    workspace.session.clear();
    drop(workspace.open(&workspace.host, &changed, &overlays, &[]));
    drop(workspace.open(&beta, BETA, &overlays, &[]));
    let cold = workspace.open(&workspace.host, &changed, &overlays, &[]);
    assert_eq!(native_documents(&cold), expected);
}

#[test]
fn a_new_overlay_at_a_failed_resolution_probe_invalidates_the_graph() {
    let workspace = Workspace::new(false);
    let missing = workspace.child.with_file_name("Missing.vue");
    let host = HOST.replace("./Child.vue", "./Missing.vue");
    let mut overlays = FxHashMap::from_iter([(workspace.host.clone(), host.as_str())]);
    let old = workspace.open(&workspace.host, &host, &overlays, &[]);
    overlays.insert(missing, CHILD);
    let updated = workspace.open(&workspace.host, &host, &overlays, &[]);
    assert!(!Arc::ptr_eq(&old.context, &updated.context));
    let expected = updated
        .mirror
        .as_ref()
        .unwrap()
        .preparation_import_facts_snapshot();
    workspace.session.clear();
    let cold = workspace.open(&workspace.host, &host, &overlays, &[]);
    assert_eq!(
        cold.mirror
            .as_ref()
            .unwrap()
            .preparation_import_facts_snapshot(),
        expected
    );
}

#[test]
fn workspace_implementations_keep_an_unchanged_tsx_root_when_a_vue_class_is_removed() {
    const CONTRACT: &str = "<script lang='ts'>\r\nexport interface Formatter { format(value: string): string }\r\nexport default {}\r\n</script>\r\n<template><span /></template>";
    const ALPHA: &str = "<script lang='ts'>\r\nimport type { Formatter } from './Host.vue'\r\nexport class Alpha implements Formatter {\r\n  /* 😀 */ format(value: string): string { return value.toUpperCase() }\r\n}\r\nexport default {}\r\n</script>\r\n<template><span /></template>";
    const BETA: &str = "import type { Formatter } from './Host.vue'\r\nexport class Beta implements Formatter {\r\n  /* 😀 */ format(value: string): string { return value.toUpperCase() }\r\n}\r\nexport const view = <span>implementation</span>\r\n";
    for beta_is_own_root in [false, true] {
        let workspace = Workspace::new(true);
        let beta = workspace.child.with_file_name("Beta.tsx");
        for (path, source) in [
            (&workspace.host, CONTRACT),
            (&workspace.other, ALPHA),
            (&beta, BETA),
        ] {
            std::fs::write(path, source).unwrap();
        }
        let open = |path: &Path,
                    content: &str,
                    overlays: &FxHashMap<PathBuf, &str>,
                    requested: &[(PathBuf, &str)]| {
            AliasContext::for_hosts_cached(
                path,
                content,
                overlays,
                requested,
                crate::corsa_bridge::CorsaVueVirtualDocumentOptions {
                    options_api: true,
                    jsx_typecheck: true,
                    ..Default::default()
                },
                CorsaProjectEnvironment {
                    virtual_ts_options: &Default::default(),
                    package_routes: &Default::default(),
                    project_root: None,
                    tsconfig_path: None,
                    editor_session: &workspace.session,
                },
            )
            .unwrap()
        };
        let mut overlays = FxHashMap::from_iter([
            (workspace.host.clone(), CONTRACT),
            (workspace.other.clone(), ALPHA),
            (beta.clone(), BETA),
        ]);
        drop(open(&workspace.host, CONTRACT, &overlays, &[]));
        drop(open(&workspace.other, ALPHA, &overlays, &[]));
        if beta_is_own_root {
            drop(open(&beta, BETA, &overlays, &[]));
        }
        let requested = [(workspace.other.clone(), ALPHA), (beta.clone(), BETA)];
        let initial = open(&workspace.host, CONTRACT, &overlays, &requested);
        let beta_path = initial.mirror_virtual_path(&beta).unwrap();
        let previous_beta = initial.source_catalog.get(&beta_path).unwrap();
        let initial_config =
            std::fs::read_to_string(initial.mirror.as_ref().unwrap().generated_tsconfig_path())
                .unwrap();
        drop(initial);
        let changed = ALPHA.replace("/* 😀 */", "/* 雪😀 changed */");
        let removed = "<script lang='ts'>\r\nimport type { Formatter } from './Host.vue'\r\nexport const removed = true\r\nexport default {}\r\n</script>\r\n<template><span /></template>";
        let mut last_native = Vec::new();
        for source in [changed.as_str(), removed] {
            overlays.insert(workspace.other.clone(), source);
            drop(open(&workspace.other, source, &overlays, &[]));
            // Implementation opens the primary and reverse SFC importer;
            // it does not request the full workspace again after every edit.
            drop(open(&workspace.host, CONTRACT, &overlays, &[]));
            let current = open(&workspace.other, source, &overlays, &[]);
            let retained = current
                .source_catalog
                .get(&beta_path)
                .expect("workspace-only TSX implementation remains live");
            assert_eq!(
                (
                    &retained.source_path,
                    &retained.source,
                    &retained.code,
                    &retained.mapping,
                    &retained.import_source_map,
                    retained.mapping_kind
                ),
                (
                    &previous_beta.source_path,
                    &previous_beta.source,
                    &previous_beta.code,
                    &previous_beta.mapping,
                    &previous_beta.import_source_map,
                    previous_beta.mapping_kind
                )
            );
            last_native = native_documents(&current);
            assert!(last_native.iter().any(|(uri, code)| uri
                == &crate::file_uri::path_to_file_uri(&beta_path)
                && code == &previous_beta.code));
            let config =
                std::fs::read_to_string(current.mirror.as_ref().unwrap().generated_tsconfig_path())
                    .unwrap();
            assert_eq!(config, initial_config);
        }
        workspace.session.clear();
        let requested = [(workspace.other.clone(), removed), (beta.clone(), BETA)];
        let cold = open(&workspace.host, CONTRACT, &overlays, &requested);
        assert_eq!(last_native, native_documents(&cold));
    }
}
