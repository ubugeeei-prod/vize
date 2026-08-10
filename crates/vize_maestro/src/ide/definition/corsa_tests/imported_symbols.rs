use super::*;

#[test]
fn definition_with_corsa_resolves_template_call_to_imported_typescript() {
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
        let dependency =
            "export function format(value: string): string { return value.toUpperCase() }\n";
        let dependency_path = src.join("format.ts");
        fs::write(&dependency_path, dependency).unwrap();
        let source = r#"<script setup lang="ts">
import { format } from './format'
import * as formatter from './format'
const values = ['local']
</script>

<template>
  {{ format('hello') }} {{ formatter.format('world') }}
  <span v-for="format in values">{{ format }}</span>
</template>
"#;
        let path = src.join("App.vue");
        fs::write(&path, source).unwrap();

        let uri = Url::from_file_path(&path).unwrap();
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

        let named_offset = source.find("format('hello')").unwrap() + 1;
        let namespace_offset =
            source.find("formatter.format('world')").unwrap() + "formatter.".len() + 1;
        let mut responses = Vec::new();
        for offset in [named_offset, namespace_offset] {
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            responses.push(
                DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone()))
                    .await
                    .expect("definition for imported template callable"),
            );
        }
        let expected = dependency.find("format").unwrap();
        let (line, character) = crate::ide::offset_to_position(dependency, expected);
        for (index, response) in responses.into_iter().enumerate() {
            let location = scalar_location(response);
            assert_eq!(
                location.uri.to_file_path().unwrap().canonicalize().unwrap(),
                dependency_path.canonicalize().unwrap(),
                "template import case {index}: {location:#?}"
            );
            assert_eq!(
                location.range.start,
                tower_lsp::lsp_types::Position::new(line, character)
            );
        }

        let shadowed_offset = source.rfind("{{ format }}").unwrap() + "{{ ".len() + 1;
        let shadowed_ctx = IdeContext::new(&state, &uri, shadowed_offset).unwrap();
        let shadowed =
            DefinitionService::definition_with_corsa(&shadowed_ctx, Some(bridge.clone()))
                .await
                .expect("definition for shadowing template binding");
        let shadowed_location = scalar_location(shadowed);
        assert_eq!(shadowed_location.uri, uri);
        let shadowed_expected = source.find("v-for=\"format").unwrap() + "v-for=\"".len();
        let (line, character) = crate::ide::offset_to_position(source, shadowed_expected);
        assert_eq!(
            shadowed_location.range.start,
            tower_lsp::lsp_types::Position::new(line, character)
        );

        let art_source = r#"<script setup lang="ts">
import { format } from './format'
function decorate(value: string): string { return `[${value}]` }
</script>

<art title="Definition">
  <variant name="Secondary">
    <p>{{ decorate('art') }} {{ format('world') }}</p>
  </variant>
</art>
"#;
        let art_path = src.join("Definition.art.vue");
        fs::write(&art_path, art_source).unwrap();
        let art_uri = Url::from_file_path(&art_path).unwrap();
        state.documents.open(
            art_uri.clone(),
            art_source.to_string(),
            1,
            "art-vue".to_string(),
        );
        state.update_virtual_docs(&art_uri, art_source);

        for (marker, expected_uri, expected_offset) in [
            (
                "decorate('art')",
                art_uri.clone(),
                art_source.find("function decorate").unwrap() + "function ".len(),
            ),
            (
                "format('world')",
                Url::from_file_path(&dependency_path).unwrap(),
                dependency.find("format").unwrap(),
            ),
        ] {
            let offset = art_source.rfind(marker).unwrap() + 1;
            let ctx = IdeContext::new(&state, &art_uri, offset).unwrap();
            let response = DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .expect("definition for art variant callable");
            let location = scalar_location(response);
            assert_eq!(location.uri, expected_uri);
            let (line, character) = crate::ide::offset_to_position(
                if expected_uri == art_uri {
                    art_source
                } else {
                    dependency
                },
                expected_offset,
            );
            assert_eq!(
                location.range.start,
                tower_lsp::lsp_types::Position::new(line, character)
            );
        }
        bridge.shutdown().await.unwrap();
    });
}
