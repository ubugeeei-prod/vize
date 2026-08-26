use vize_canon::{
    CorsaBridge, CorsaBridgeConfig, ImportRewriter, batch::generate_vue_document_virtual_ts,
    virtual_ts::VirtualTsOptions,
};

#[test]
fn editor_references_link_generated_vue_module_exports_to_importers() {
    let Some(tsgo_path) = resolve_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    std::fs::write(
        project.path().join("tsconfig.json"),
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
    .expect("tsconfig");
    let child_path = src.join("Child.vue");
    let parent_path = src.join("Parent.vue");
    let child_source = r#"<script lang="ts">
export const shared = 1
export default {}
</script>"#;
    let parent_source = r#"<script setup lang="ts">
import { shared } from './Child.vue'
const local = shared
</script>"#;
    std::fs::write(&child_path, child_source).expect("child");
    std::fs::write(&parent_path, parent_source).expect("parent");
    let rewriter = ImportRewriter::new();
    let child = generate_vue_document_virtual_ts(
        &child_path,
        child_source,
        &VirtualTsOptions::default(),
        &rewriter,
        false,
    )
    .expect("child virtual ts");
    let parent = generate_vue_document_virtual_ts(
        &parent_path,
        parent_source,
        &VirtualTsOptions::default(),
        &rewriter,
        false,
    )
    .expect("parent virtual ts");
    let child_virtual_path = child_path.with_extension("vue.ts");
    let parent_virtual_path = parent_path.with_extension("vue.ts");
    let child_export = child
        .code
        .rfind("export const shared")
        .expect("module export")
        + "export const ".len();
    let (line, character) = position(&child.code, child_export);
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(tsgo_path),
        working_dir: Some(project.path().to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let (child_uri, parent_uri, references) = corsa::runtime::block_on(async {
        bridge.spawn().await.expect("tsgo session");
        let child_uri = bridge
            .open_or_update_virtual_document(&child_virtual_path.to_string_lossy(), &child.code)
            .await
            .expect("child open");
        let parent_uri = bridge
            .open_or_update_virtual_document(&parent_virtual_path.to_string_lossy(), &parent.code)
            .await
            .expect("parent open");
        let references = bridge
            .references(&child_uri, line, character, true)
            .await
            .expect("references");
        bridge.shutdown().await.expect("shutdown");
        (child_uri, parent_uri, references)
    });

    assert!(
        references.iter().any(|location| location.uri == child_uri),
        "declaration missing: {references:#?}",
    );
    assert!(
        references.iter().any(|location| location.uri == parent_uri),
        "importer missing: {references:#?}",
    );
    assert!(!child_virtual_path.exists());
    assert!(!parent_virtual_path.exists());
}

fn position(source: &str, offset: usize) -> (u32, u32) {
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let character = prefix[line_start..].encode_utf16().count() as u32;
    (line, character)
}

fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)?;
    vize_s0::corsa_resolver::resolve_corsa_executable(
        vize_s0::corsa_resolver::CorsaResolveRequest {
            project_root: Some(workspace_root),
            ..Default::default()
        },
    )
    .ok()
}
