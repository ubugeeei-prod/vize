use super::*;

#[test]
fn definition_with_corsa_maps_computed_template_binding_to_its_declaration() {
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
    "allowImportingTsExtensions": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
        )
        .unwrap();
        fs::write(
            src.join("vue.d.ts"),
            r#"declare module "vue" {
  export interface ComputedRef<T = unknown> { value: T }
  export function computed<T>(getter: () => T): ComputedRef<T>
}
"#,
        )
        .unwrap();

        let source = r#"<script lang="ts">
export interface RadioProps {
  id?: string
  value?: string
}
</script>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<RadioProps>()
const { forwardRef, currentElement: triggerElement } = useForwardExpose()
const isFormControl = useFormControl(triggerElement)
const ariaLabel = computed(() => props.id && triggerElement.value ? (document.querySelector(`[for="${props.id}"]`) as HTMLLabelElement)?.innerText ?? props.value : undefined)
</script>

<template>
  <Primitive
    :ref="forwardRef"
    :aria-label="ariaLabel"
    :data-form-control="isFormControl"
  />
</template>
"#;
        let source_path = src.join("Radio.vue");
        fs::write(&source_path, source).unwrap();
        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(dir.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        let offset = source.rfind("ariaLabel").unwrap();
        let sync_ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let sync_location = scalar_location(
            DefinitionService::definition(&sync_ctx).expect("synchronous definition"),
        );
        let expected_offset = source.find("const ariaLabel").unwrap() + "const ".len();
        let (line, character) = crate::ide::offset_to_position(source, expected_offset);
        assert_eq!(sync_location.range.start.line, line);
        assert_eq!(sync_location.range.start.character, character);

        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(dir.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();

        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let response = DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .unwrap();
        let _ = bridge.shutdown().await;
        let location = scalar_location(response);
        assert_eq!(location.uri, uri);
        assert_eq!(location.range.start.line, line);
        assert_eq!(location.range.start.character, character);
        assert_eq!(location.range.end.line, line);
        assert_eq!(
            location.range.end.character,
            character + "ariaLabel".len() as u32
        );
    });
}

#[test]
fn definition_with_corsa_rejects_a_deleted_component_import_alias() {
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
    "allowImportingTsExtensions": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
        )
        .unwrap();
        fs::write(
            src.join("vue.d.ts"),
            r#"declare module "vue" {
  export type DefineComponent<P = any> = { new(): { $props: P } };
}
"#,
        )
        .unwrap();

        let child_path = src.join("Child.vue");
        fs::write(&child_path, "<template><button /></template>\n").unwrap();
        let parent_source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template><Child /></template>
"#;
        let parent_path = src.join("Parent.vue");
        fs::write(&parent_path, parent_source).unwrap();

        let uri = Url::from_file_path(&parent_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(dir.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), parent_source.to_owned(), 1, "vue".to_owned());
        state.update_virtual_docs(&uri, parent_source);

        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(dir.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();

        let offset = parent_source.find("<Child").unwrap() + 1;
        let warm_ctx = IdeContext::new(&state, &uri, offset).unwrap();
        crate::ide::corsa_support::open_canonical_virtual_document(&warm_ctx, &bridge)
            .await
            .expect("open canonical document before deletion");

        fs::remove_file(&child_path).unwrap();
        bridge
            .forget_vue_virtual_documents(std::slice::from_ref(&child_path))
            .await
            .unwrap();
        bridge.invalidate_disk_project_state().await.unwrap();

        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let response = DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone())).await;
        let _ = bridge.shutdown().await;

        assert!(
            response.is_none(),
            "deleted component definition must be null, got {response:?}"
        );
    });
}
