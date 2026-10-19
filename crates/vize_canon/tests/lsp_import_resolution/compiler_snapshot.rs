//! Real native requests retain admitted compiler authority, including cold startup.
#![expect(
    clippy::unwrap_used,
    reason = "complete native fixture assertions panic"
)]
#![expect(
    clippy::string_slice,
    reason = "test queries an independently authored declaration"
)]
use super::*;
use vize_canon::{CompilerConfigurationSnapshot, CorsaVueVirtualDocumentOptions};

#[expect(
    clippy::panic,
    reason = "the native fixture requires the complete markdown variant"
)]
async fn assert_typed_hover(bridge: &CorsaBridge, source: &Path, text: &str, typ: &str) {
    let document = bridge
        .open_vue_virtual_document(source, text, CorsaVueVirtualDocumentOptions::default())
        .await
        .unwrap();
    let declaration = vize_l0::cstr!("const value: {typ}");
    let offset = document.code.find(declaration.as_str()).unwrap() + "const ".len();
    let prefix = &document.code[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let character = prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32;
    let hover = bridge
        .hover(document.request_uri.as_str(), line, character + 2)
        .await
        .unwrap()
        .expect("the retained native owner must serve a complete typed hover");
    match hover.contents {
        LspHoverContents::Markup(markup) => {
            assert_eq!(markup.kind, "markdown");
            assert_eq!(
                markup.value,
                // Native 7.0.2 gitHead 2bd066d87f5bafd315be9f40889d0a60b9e58e0b:
                // internal/ls/hover.go writeCode writes LF after the closing fence.
                vize_l0::cstr!("```typescript\nconst value: {typ}\n```\n").as_str()
            );
        }
        other => panic!("expected complete native markdown, received {other:?}"),
    }
    assert_eq!(
        serde_json::to_value(hover.range).unwrap(),
        serde_json::json!({"start":{"line":line,"character":character},"end":{"line":line,"character":character+5}})
    );
}

#[test]
#[cfg(unix)]
#[expect(
    clippy::disallowed_types,
    reason = "cold and warm native owners share immutable admitted compiler snapshots"
)]
fn admitted_compiler_configuration_survives_deleted_extends_before_first_native_request_and_rebuilds()
 {
    let native = resolve_test_tsgo_binary();
    if native.is_none() {
        assert_ne!(
            std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref(),
            Ok("1"),
            "this acceptance requires the actual native backend"
        );
        return;
    }
    for warmed in [false, true] {
        let root = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(
            workspace_root().join("playground/node_modules"),
            root.path().join("node_modules"),
        )
        .unwrap();
        let config = root.path().join("tsconfig.json");
        let base = root.path().join("tsconfig.base.json");
        let policy = r#"{"compilerOptions":{"strict":true,"strictNullChecks":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true}}"#;
        std::fs::write(&base, policy).unwrap();
        std::fs::write(
            &config,
            r#"{"extends":"./tsconfig.base.json","include":["*.vue"]}"#,
        )
        .unwrap();
        let source = root.path().join("TypedPanel.vue");
        let original = "<script setup lang=\"ts\">\nconst value: string = null;\n</script>\n<template>{{ value }}</template>\n";
        std::fs::write(&source, original).unwrap();
        let admitted = std::sync::Arc::new(
            CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap(),
        );
        let bridge = CorsaBridge::with_compiler_configuration(
            CorsaBridgeConfig {
                corsa_path: native.clone(),
                working_dir: Some(root.path().to_path_buf()),
                tsconfig_path: Some(config.clone()),
                ..Default::default()
            },
            Default::default(),
            Some(admitted),
        );
        corsa::runtime::block_on(async {
            if warmed {
                bridge.spawn().await.unwrap();
                assert_typed_hover(&bridge, &source, original, "string").await;
            }
            std::fs::remove_file(&base).unwrap();
            // A live capture skips the deleted base like `tsc`, reporting it.
            let live = CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap();
            assert_eq!(live.unresolved_extends().len(), 1);
            bridge.spawn().await.unwrap();
            assert_typed_hover(&bridge, &source, original, "string").await;
            let unsaved =
                original.replace("const value: string = null", "const value: number = 42");
            assert_typed_hover(&bridge, &source, &unsaved, "number").await;
            assert_eq!(std::fs::read_to_string(&source).unwrap(), original);
            bridge.shutdown().await.unwrap();
        });
        std::fs::write(
            &base,
            policy.replace("\"strictNullChecks\":true", "\"strictNullChecks\":false"),
        )
        .unwrap();
        let repaired = std::sync::Arc::new(
            CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap(),
        );
        let next = CorsaBridge::with_compiler_configuration(
            CorsaBridgeConfig {
                corsa_path: native.clone(),
                working_dir: Some(root.path().to_path_buf()),
                tsconfig_path: Some(config),
                ..Default::default()
            },
            Default::default(),
            Some(repaired),
        );
        corsa::runtime::block_on(async {
            next.spawn().await.unwrap();
            assert_typed_hover(&next, &source, original, "string").await;
            next.shutdown().await.unwrap();
        });
    }
}

#[test]
#[cfg(unix)]
fn retired_union_member_leaves_native_mirror_while_other_member_and_old_catalog_remain_valid() {
    let native = resolve_test_tsgo_binary();
    if native.is_none() {
        assert_ne!(
            std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref(),
            Ok("1"),
            "this acceptance requires the actual native backend"
        );
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let project = vize_carton::path::canonicalize_non_verbatim(root.path());
    std::os::unix::fs::symlink(
        workspace_root().join("playground/node_modules"),
        project.join("node_modules"),
    )
    .unwrap();
    let config = project.join("tsconfig.json");
    std::fs::write(&config, r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["*.vue"]}"#).unwrap();
    let a = project.join("Moved.vue");
    let b = project.join("Control.vue");
    let a_text = "<script setup lang=\"ts\">\nconst value: string = 'moved';\n</script>\n<template>{{ value }}</template>\n";
    let b_text = "<script setup lang=\"ts\">\nconst value: number = 42;\n</script>\n<template>{{ value }}</template>\n";
    std::fs::write(&a, a_text).unwrap();
    std::fs::write(&b, b_text).unwrap();
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: native,
        working_dir: Some(project.clone()),
        tsconfig_path: Some(config),
        ..Default::default()
    });
    corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let moved_packet = bridge
            .open_vue_virtual_document(&a, a_text, CorsaVueVirtualDocumentOptions::default())
            .await
            .unwrap();
        let moved = moved_packet
            .materialized_sources
            .iter()
            .find(|source| source.source_path == a)
            .expect("the moved source owns its exact materialized identity");
        let retired_path = moved.materialized_path.clone();
        assert_eq!(moved.source.as_str(), a_text);
        assert!(retired_path.is_file());
        assert_typed_hover(&bridge, &a, a_text, "string").await;
        let before = bridge
            .open_vue_virtual_document(&b, b_text, CorsaVueVirtualDocumentOptions::default())
            .await
            .unwrap();
        assert_typed_hover(&bridge, &b, b_text, "number").await;
        // A query surface can remain scoped to B. Its live union catalog,
        // rather than its local materialized-source list, must still own A.
        assert_eq!(
            before
                .source_catalog
                .get(&retired_path)
                .unwrap()
                .source
                .as_str(),
            a_text
        );
        bridge.forget_cached_vue_virtual_documents(std::slice::from_ref(&a));
        bridge.mark_disk_project_state_dirty();
        bridge.flush_disk_project_state_if_dirty().await.unwrap();
        let after = bridge
            .open_vue_virtual_document(&b, b_text, CorsaVueVirtualDocumentOptions::default())
            .await
            .unwrap();
        assert!(bridge.is_initialized());
        assert!(
            !retired_path.exists(),
            "next synchronization removes the retired materialized source"
        );
        assert!(after.source_catalog.get(&retired_path).is_none());
        assert!(
            before.source_catalog.get(&retired_path).is_some(),
            "in-flight old packets retain their original complete source catalog"
        );
        assert!(
            !before
                .source_catalog
                .shares_revision_with(&after.source_catalog)
        );
        assert_typed_hover(&bridge, &b, b_text, "number").await;
        assert_eq!(std::fs::read_to_string(&a).unwrap(), a_text);
        assert_eq!(std::fs::read_to_string(&b).unwrap(), b_text);
        bridge.shutdown().await.unwrap();
    });
}
