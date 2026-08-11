use std::path::{Path, PathBuf};
use std::sync::Arc;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::{SignatureHelpService, SignatureHelpStage};
use crate::ide::{IdeContext, JsxService};
use crate::server::ServerState;

#[test]
fn signature_help_maps_sfc_script_and_template_with_crlf_and_utf16() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let fixture = Fixture::new(&corsa_path);
        let source = "<script lang=\"ts\">\r\nfunction regularFormat(value: string, precision: number): string { return value.repeat(precision) }\r\nregularFormat('🦀', );\r\n</script>\r\n<script setup lang=\"ts\">\r\nconst emoji = '🦀';\r\nfunction format(value: string, precision: number): string { return value.repeat(precision) }\r\nformat(emoji, );\r\n</script>\r\n\r\n<template>\r\n  <p>{{ format('🦀', ) }}</p>\r\n</template>\r\n";
        let (state, uri) = fixture.vue("App.vue", source);
        let bridge = fixture.bridge();
        bridge.spawn().await.unwrap();

        for (marker, name) in [
            ("regularFormat('🦀', ", "regularFormat"),
            ("format(emoji, ", "format"),
            ("format('🦀', ", "format"),
        ] {
            let offset = source.find(marker).unwrap() + marker.len();
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            let help = SignatureHelpService::signature_help_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .expect("signature help at authored SFC call");
            assert_signature(help, name, 1);
        }

        let offset = source.find("const emoji").unwrap() + "const".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        assert!(
            SignatureHelpService::signature_help_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .is_none(),
            "signature help must not leak outside a call expression"
        );

        bridge.shutdown().await.unwrap();
    });
}

#[test]
fn signature_help_maps_art_variant_template() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let fixture = Fixture::new(&corsa_path);
        let source = "<script setup lang=\"ts\">\nfunction format(value: string, precision: number): string { return value.repeat(precision) }\nformat('script', )\n</script>\n\n<art title=\"Button\" component=\"./Button.vue\">\n  <variant name=\"Empty\">\n    <p>Nothing to call</p>\n  </variant>\n  <variant name=\"Primary\">\n    <p>{{ format('art', ) }}</p>\n  </variant>\n</art>\n";
        let (state, uri) = fixture.vue("Button.art.vue", source);
        let bridge = fixture.bridge();
        bridge.spawn().await.unwrap();

        let script_marker = "format('script', ";
        let script_offset = source.find(script_marker).unwrap() + script_marker.len();
        let script_ctx = IdeContext::new(&state, &uri, script_offset).unwrap();
        let (help, stages) = SignatureHelpService::signature_help_with_corsa_traced(
            &script_ctx,
            Some(bridge.clone()),
        )
        .await;
        let help = help.unwrap_or_else(|| panic!("signature help in art script setup: {stages:?}"));
        assert_eq!(
            stages,
            [
                SignatureHelpStage::VirtualOpened,
                SignatureHelpStage::RequestSome,
            ]
        );
        assert_signature(help, "format", 1);

        let marker = "format('art', ";
        let offset = source.find(marker).unwrap() + marker.len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let info = match ctx.block_type {
            Some(crate::virtual_code::BlockType::Art(
                crate::virtual_code::ArtCursorPosition::VariantTemplate(info),
            )) => info,
            other => panic!("expected art variant context, got {other:?}"),
        };
        let template = ctx
            .virtual_docs
            .as_ref()
            .and_then(|docs| docs.art_template(info.variant_index))
            .expect("art variant virtual document");
        assert!(
            template.content.contains("format('art', )"),
            "generated art template lost the authored call:\n{}",
            template.content
        );
        assert!(
            template.content.contains("function format"),
            "generated art template lost the callable declaration:\n{}",
            template.content
        );
        let generated_offset = template
            .source_map
            .to_generated_for(offset as u32, |features| features.signature_help)
            .expect("art signature-help mapping") as usize;
        let expected_generated_offset = template.content.find(marker).unwrap() + marker.len();
        assert_eq!(
            generated_offset, expected_generated_offset,
            "art cursor mapped to the wrong virtual offset:\n{}",
            template.content
        );
        let (help, stages) =
            SignatureHelpService::signature_help_with_corsa_traced(&ctx, Some(bridge.clone()))
                .await;
        let help = help.unwrap_or_else(|| panic!("signature help in art variant: {stages:?}"));
        assert_eq!(
            stages,
            [
                SignatureHelpStage::VirtualOpened,
                SignatureHelpStage::RequestSome,
            ]
        );
        assert_signature(help, "format", 1);

        bridge.shutdown().await.unwrap();
    });
}

#[test]
fn signature_help_maps_tsx_calls() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let fixture = Fixture::new(&corsa_path);
        let source = "function format(value: string, precision: number): string { return value.repeat(precision) }\nexport default () => <p>{format('tsx', )}</p>;\n";
        let (state, uri) = fixture.tsx("Component.tsx", source);
        let bridge = fixture.bridge();
        bridge.spawn().await.unwrap();

        let marker = "format('tsx', ";
        let offset = source.find(marker).unwrap() + marker.len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let help = JsxService::signature_help(&ctx, Some(bridge.clone()))
            .await
            .expect("signature help in TSX");
        assert_signature(help, "format", 1);

        bridge.shutdown().await.unwrap();
    });
}

#[test]
fn signature_help_tracks_imported_callable_edits() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let fixture = Fixture::new(&corsa_path);
        let dependency_path = fixture.root.path().join("src/format.ts");
        std::fs::write(
            &dependency_path,
            "export declare function format(value: string, precision: number): string;\n",
        )
        .unwrap();
        let source = "<script setup lang=\"ts\">\nimport { format } from './format';\n</script>\n<template>{{ format('imported', ) }}</template>\n";
        let (state, uri) = fixture.vue("Imported.vue", source);
        let bridge = fixture.bridge();
        bridge.spawn().await.unwrap();

        let marker = "format('imported', ";
        let offset = source.find(marker).unwrap() + marker.len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let help = SignatureHelpService::signature_help_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .expect("signature help from imported callable");
        assert_signature_type(&help, "precision: number");

        let updated = "export declare function format(value: string, precision: bigint): string;\n";
        std::fs::write(&dependency_path, updated).unwrap();
        bridge
            .open_or_update_virtual_document(dependency_path.to_str().unwrap(), updated)
            .await
            .unwrap();

        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let help = SignatureHelpService::signature_help_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .expect("signature help after imported callable edit");
        assert_signature_type(&help, "precision: bigint");

        bridge.shutdown().await.unwrap();
    });
}

fn assert_signature(help: tower_lsp::lsp_types::SignatureHelp, name: &str, active_parameter: u32) {
    assert_eq!(help.active_signature, Some(0));
    assert_eq!(help.active_parameter, Some(active_parameter));
    assert_eq!(help.signatures.len(), 1);
    let signature = &help.signatures[0];
    assert!(signature.label.contains(name), "{}", signature.label);
    assert!(
        signature.label.contains("value: string"),
        "{}",
        signature.label
    );
    assert!(
        signature.label.contains("precision: number"),
        "{}",
        signature.label
    );
    assert_eq!(
        signature.parameters.as_ref().map(Vec::len),
        Some(2),
        "{}",
        signature.label
    );
}

fn assert_signature_type(help: &tower_lsp::lsp_types::SignatureHelp, expected: &str) {
    assert_eq!(help.signatures.len(), 1);
    assert!(
        help.signatures[0].label.contains(expected),
        "{}",
        help.signatures[0].label
    );
}

struct Fixture {
    root: tempfile::TempDir,
    corsa_path: PathBuf,
}

impl Fixture {
    fn new(corsa_path: &Path) -> Self {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("src")).unwrap();
        write_project(root.path());
        Self {
            root,
            corsa_path: corsa_path.to_path_buf(),
        }
    }

    fn vue(&self, name: &str, source: &str) -> (ServerState, Url) {
        self.open(name, source, "vue")
    }

    fn tsx(&self, name: &str, source: &str) -> (ServerState, Url) {
        self.open(name, source, "typescriptreact")
    }

    fn open(&self, name: &str, source: &str, language_id: &str) -> (ServerState, Url) {
        let path = self.root.path().join("src").join(name);
        std::fs::write(&path, source).unwrap();
        let uri = Url::from_file_path(path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(self.root.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, language_id.to_string());
        state.update_virtual_docs(&uri, source);
        (state, uri)
    }

    fn bridge(&self) -> Arc<CorsaBridge> {
        Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(self.corsa_path.clone()),
            working_dir: Some(self.root.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }))
    }
}

fn write_project(root: &Path) {
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    let vue = root.join("node_modules/vue");
    std::fs::create_dir_all(&vue).unwrap();
    std::fs::write(
        vue.join("package.json"),
        r#"{"name":"vue","version":"3.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        vue.join("index.d.ts"),
        r#"export type DefineComponent<P = any, _B = any, _D = any> = { new(): { $props: P } };
export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
export interface Ref<T = unknown, _Raw = T> { value: T }
export interface ShallowRef<T = unknown> extends Ref<T> {}
"#,
    )
    .unwrap();
}

fn resolve_tsgo_binary() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    [
        workspace_root.parent()?.join("corsa-bind/.cache/tsgo"),
        workspace_root
            .parent()?
            .join("corsa-bind/ref/corsa-upstream/.cache/tsgo"),
        workspace_root.join("node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
    .or_else(|| vize_carton::corsa_resolver::discover_corsa_in_ancestors(workspace_root))
}
