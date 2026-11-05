//! Native diagnostics across host edits that keep, add and remove imports.

use std::path::{Path, PathBuf};

use vize_carton::corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable};

use super::{CorsaBridge, CorsaBridgeConfig, CorsaVueVirtualDocumentOptions};

const TSCONFIG: &str = r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true,"skipLibCheck":true},"include":["src/**/*"]}"#;

fn host(script: &str) -> vize_carton::String {
    vize_carton::cstr!(
        "<script setup lang=\"ts\">\n{script}\n</script>\n<template>{{{{ shown }}}}</template>\n"
    )
}

/// Errors an author can cause here; generated helpers against the facade
/// report others on every revision alike.
const AUTHORED: [i64; 4] = [2304, 2305, 2307, 2322];

/// Each authored error as its code and the identifier its message names.
async fn errors(bridge: &CorsaBridge, path: &Path, source: &str) -> Vec<(i64, &'static str)> {
    // The editor's own buffer is one of its overlays.
    let document = bridge
        .open_vue_virtual_document_with_borrowed_overlays_and_options(
            path,
            source,
            CorsaVueVirtualDocumentOptions::default(),
            &[(path.to_path_buf(), source)],
            &Default::default(),
        )
        .await
        .unwrap();
    let mut errors = bridge
        .get_diagnostics(&document.request_uri)
        .await
        .unwrap()
        .into_iter()
        .filter_map(|diagnostic| {
            let code = diagnostic.code?.as_i64()?;
            AUTHORED.contains(&code).then(|| {
                let named = ["absent", "extra", "./nope", "type 'number'"]
                    .into_iter()
                    .find(|name| diagnostic.message.contains(name))
                    .unwrap_or("");
                (code, named)
            })
        })
        .collect::<Vec<_>>();
    errors.sort_unstable();
    errors
}

#[test]
fn host_edits_that_keep_add_and_remove_imports_keep_native_diagnostics_exact() {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return;
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let explicit = std::env::var_os("VIZE_TEST_TSGO_PATH").map(PathBuf::from);
    let corsa = resolve_corsa_executable(CorsaResolveRequest {
        explicit_path: explicit.as_deref(),
        project_root: Some(workspace),
    });
    assert!(corsa.is_ok() || std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none());
    let Ok(corsa) = corsa else { return };
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();
    crate::batch::write_vue_facade(&root.join("node_modules")).unwrap();
    std::fs::write(root.join("tsconfig.json"), TSCONFIG).unwrap();
    std::fs::write(src.join("util.ts"), "export const label: string = 'ok';\n").unwrap();
    std::fs::write(src.join("extra.ts"), "export const extra: string = 'e';\n").unwrap();
    let path = src.join("Host.vue");
    let clean = host("import { label } from './util';\nconst shown: string = label;");
    std::fs::write(&path, clean.as_str()).unwrap();
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa),
        working_dir: Some(root.clone()),
        timeout_ms: 30_000,
        ..Default::default()
    });
    corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        assert_eq!(errors(&bridge, &path, &clean).await, Vec::new());
        // Same imports, new body: the error comes from the new buffer.
        let wrong = host("import { label } from './util';\nconst shown: number = label;");
        assert_eq!(
            errors(&bridge, &path, &wrong).await,
            [(2322, "type 'number'")]
        );
        // Same module specifier, a name it does not export.
        let missing =
            host("import { label, absent } from './util';\nconst shown: string = label + absent;");
        assert_eq!(errors(&bridge, &path, &missing).await, [(2305, "absent")]);
        // A new import's types reach the host.
        let added = host(
            "import { label } from './util';\nimport { extra } from './extra';\nconst shown: number = label.length + extra;",
        );
        assert_eq!(
            errors(&bridge, &path, &added).await,
            [(2322, "type 'number'")]
        );
        // Removing it leaves the use unresolved.
        let removed = host("import { label } from './util';\nconst shown: string = label + extra;");
        assert_eq!(errors(&bridge, &path, &removed).await, [(2304, "extra")]);
        let unresolved = host(
            "import { label } from './util';\nimport { nope } from './nope';\nconst shown: string = label + nope;",
        );
        assert_eq!(
            errors(&bridge, &path, &unresolved).await,
            [(2307, "./nope")]
        );
        assert_eq!(errors(&bridge, &path, &clean).await, Vec::new());
        bridge.shutdown().await.unwrap();
    });
}
