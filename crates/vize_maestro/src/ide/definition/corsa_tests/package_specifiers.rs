use super::*;

#[test]
fn definition_with_corsa_uses_native_condition_for_package_specifier() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        let package = root.path().join("node_modules/@scope/ui");
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(
            root.path().join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowArbitraryExtensions": true,
    "customConditions": ["editor"]
  },
  "include": ["src/**/*"]
}"#,
        )
        .unwrap();
        fs::write(
            package.join("package.json"),
            r#"{
  "name": "@scope/ui",
  "exports": { ".": { "editor": "./src/Selected.vue", "types": "./src/Fallback.vue" } }
}"#,
        )
        .unwrap();
        let selected = package.join("src/Selected.vue");
        fs::write(
            &selected,
            "<script setup lang=\"ts\">defineProps<{ selected: string }>()</script>\n",
        )
        .unwrap();
        fs::write(
            package.join("src/Fallback.vue"),
            "<script setup lang=\"ts\">defineProps<{ fallback: number }>()</script>\n",
        )
        .unwrap();
        let source = r#"<script setup lang="ts">
import Widget from '@scope/ui'
void Widget
</script>
"#;
        let source_path = src.join("App.vue");
        fs::create_dir_all(&src).unwrap();
        fs::write(&source_path, source).unwrap();
        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(root.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();
        let offset = source.find("@scope/ui").unwrap() + 1;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let response = DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .expect("native package definition");
        let _ = bridge.shutdown().await;
        let location = scalar_location(response);
        assert_eq!(
            location.uri.to_file_path().unwrap().canonicalize().unwrap(),
            selected.canonicalize().unwrap(),
            "definition must agree with the native selected condition"
        );
        assert_eq!(
            location.range,
            tower_lsp::lsp_types::Range::new(
                tower_lsp::lsp_types::Position::new(0, 0),
                tower_lsp::lsp_types::Position::new(0, 0)
            ),
            "module-specifier navigation selects the authored file, not a symbol span"
        );
    });
}

#[test]
fn definition_with_corsa_maps_conditional_private_import_from_the_exact_shadow() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let root = tempfile::tempdir().unwrap();
        let package = root.path().join("node_modules/@scope/ui");
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(
            root.path().join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowArbitraryExtensions": true,
    "customConditions": ["editor"]
  }
}"#,
        )
        .unwrap();
        fs::write(
            package.join("package.json"),
            r##"{
  "name": "@scope/ui",
  "exports": { ".": "./src/Host.vue" },
  "imports": { "#component": { "editor": "./src/Selected.js", "default": "./src/Fallback.js" } }
}"##,
        )
        .unwrap();
        let selected = package.join("src/Selected.vue");
        fs::write(
            &selected,
            "<script setup lang=\"ts\">defineProps<{ selected: string }>()</script>\n",
        )
        .unwrap();
        fs::write(
            package.join("src/Fallback.vue"),
            "<script setup lang=\"ts\">defineProps<{ fallback: number }>()</script>\n",
        )
        .unwrap();
        let source = r#"<script setup lang="ts">
import Component from '#component'
void Component
</script>
"#;
        let source_path = package.join("src/Host.vue");
        fs::write(&source_path, source).unwrap();
        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(root.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();
        let offset = source.find("#component").unwrap() + 1;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let response = DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .expect("native private-import definition");
        let _ = bridge.shutdown().await;
        let location = scalar_location(response);
        assert_eq!(
            location.uri.to_file_path().unwrap().canonicalize().unwrap(),
            selected.canonicalize().unwrap(),
            "definition must use Canon's exact private-import shadow map"
        );
        assert!(!location.uri.path().contains(".vize"));
    });
}

#[test]
fn definition_with_corsa_identity_maps_an_authored_declaration_package_barrel() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        let package = root.path().join("node_modules/@scope/ui");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&package).unwrap();
        fs::write(
            root.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowArbitraryExtensions":true},"include":["src/**/*"]}"#,
        )
        .unwrap();
        fs::write(
            package.join("package.json"),
            r#"{"name":"@scope/ui","exports":{".":"./index.d.ts"}}"#,
        )
        .unwrap();
        let barrel = package.join("index.d.ts");
        fs::write(
            &barrel,
            "export { default } from './Entry.vue'\nexport declare const shared: 1\n",
        )
        .unwrap();
        fs::write(
            package.join("Entry.vue"),
            "<script setup lang=\"ts\">defineProps<{ label: string }>()</script>\n",
        )
        .unwrap();
        let source = r#"<script setup lang="ts">
import { shared } from '@scope/ui'
const value = shared
void value
</script>
"#;
        let source_path = src.join("App.vue");
        fs::write(&source_path, source).unwrap();
        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(root.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();
        let offset = source.find("shared\nvoid").unwrap() + 1;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let response = DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .expect("native declaration barrel definition");
        bridge.shutdown().await.unwrap();
        let location = scalar_location(response);
        assert_eq!(
            location.uri.to_file_path().unwrap().canonicalize().unwrap(),
            barrel.canonicalize().unwrap()
        );
        assert_ne!(
            location.range.start, location.range.end,
            "ordinary symbol definitions must keep their native authored span"
        );
        assert!(!location.uri.path().contains(".vize"));
    });
}
