//! Art variants retain native dependency types and unsaved dependency edits.

use tower_lsp::lsp_types::{NumberOrString, Url};

use super::DiagnosticService;
use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_art_vue_import_fixture, write_corsa_config,
};

#[test]
fn art_variants_resolve_aliases_and_track_unsaved_transitive_types() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::tempdir().unwrap();
    let root = project.path().join("仕様 with spaces");
    std::fs::create_dir_all(&root).unwrap();
    write_art_vue_import_fixture(&root);
    write_corsa_config(&root, &corsa_path);
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
        "compilerOptions": { "strict": true, "module": "ESNext", "moduleResolution": "Bundler",
            "noEmit": true, "paths": { "@ui/*": ["./src/*"] } }, "include": ["src/**/*"]
    }"#,
    )
    .unwrap();
    let contract = root.join("src/contract.ts");
    std::fs::write(&contract, "export type Label = string;\n").unwrap();
    std::fs::write(
        root.join("src/Button.vue"),
        r#"<script setup lang="ts">
import type { Label } from './contract';
defineProps<{ label: Label }>();
</script><template><button>{{ label }}</button></template>"#,
    )
    .unwrap();
    let source = r#"<script setup lang="ts">
defineArt("@ui/Button.vue", { title: "Button" });
</script>
<art>
  <variant name="First" default>
    <Button :label="123" />
  </variant>
  <variant name="Second">
    <Button :label="456" />
  </variant>
</art>"#;
    let art_path = root.join("src/Button.art.vue");
    std::fs::write(&art_path, source).unwrap();
    let uri = Url::from_file_path(&art_path).unwrap();
    let state = state_for_fixture(&root, &uri, source);
    state.load_workspace_config(&root);
    let collect = || crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
    let errors = collect();
    assert_eq!(errors.len(), 2, "{errors:#?}");
    for (error, line) in errors.iter().zip([5, 8]) {
        assert_eq!(error.code, Some(NumberOrString::Number(2322)));
        assert_eq!(error.range.start.line, line);
    }
    let contract_uri = Url::from_file_path(&contract).unwrap();
    state.documents.open(
        contract_uri.clone(),
        "export type Label = number;\n".into(),
        1,
        "typescript".into(),
    );
    assert_eq!(
        collect(),
        Vec::new(),
        "unsaved transitive declaration must update both variants"
    );
    state.documents.open(
        contract_uri,
        "export type Label = string;\n".into(),
        2,
        "typescript".into(),
    );
    assert_eq!(
        collect(),
        errors,
        "restoring the declaration restores the exact authored diagnostics"
    );
}
