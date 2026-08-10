use super::*;

#[test]
fn hover_with_corsa_ranges_bare_define_props_template_binding() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            dir.path().join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
        )
        .unwrap();
        fs::write(
            src.join("vue.d.ts"),
            r#"declare module "vue" {
  export type DefineComponent<P = any, _B = any, _D = any> = { new(): { $props: P } };
}
"#,
        )
        .unwrap();

        let source = r#"<script setup lang="ts">
defineProps<{ describedBy: string }>()
</script>
<template><div :aria-describedby="describedBy" /></template>
"#;
        let source_path = src.join("Message.vue");
        fs::write(&source_path, source).unwrap();
        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(dir.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(dir.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();

        let start = source.rfind("describedBy").unwrap();
        let ctx = IdeContext::new(&state, &uri, start).unwrap();
        let hover = HoverService::hover_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .unwrap();
        let _ = bridge.shutdown().await;
        let (start_line, start_character) = crate::ide::offset_to_position(source, start);
        let (end_line, end_character) =
            crate::ide::offset_to_position(source, start + "describedBy".len());

        assert_eq!(
            hover.range,
            Some(Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            )),
        );
        let value = hover_markdown(hover);
        assert!(value.contains("const describedBy: string"), "{value}");
    });
}
