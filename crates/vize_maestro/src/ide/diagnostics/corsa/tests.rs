use super::mapping::{line_character_to_byte_offset, source_offset_to_position};

#[test]
fn line_character_to_byte_offset_counts_utf16_code_units() {
    let source = "const icon = \"😀\";\nconst message = icon";

    assert_eq!(
        line_character_to_byte_offset(source, 0, 16),
        Some("const icon = \"😀".len())
    );
    assert_eq!(
        line_character_to_byte_offset(source, 1, 6),
        Some(source.find("message").unwrap())
    );
}

#[test]
fn line_character_to_byte_offset_rejects_surrogate_pair_interior() {
    let source = "a😀b";

    assert_eq!(line_character_to_byte_offset(source, 0, 2), None);
}

#[test]
fn source_offset_to_position_counts_utf16_code_units() {
    let source = "const icon = \"😀\"; missing";
    let offset = source.find("missing").unwrap();

    assert_eq!(source_offset_to_position(source, offset), (0, 19));
}

/// Issue #752: editor-side virtual TS generation must rewrite `.vue`
/// import specifiers to `.vue.ts` so the Corsa session can resolve
/// siblings via the virtual mirror — alias *and* relative specifiers
/// both get rewritten, mirroring the batch pipeline.
#[test]
fn editor_virtual_ts_rewrites_dot_vue_imports() {
    use crate::DiagnosticService;
    use tower_lsp::lsp_types::Url;

    let uri = Url::parse("file:///tmp/Host.vue").expect("parse uri");
    let content = "<script setup lang=\"ts\">\n\
                   import App from './app.vue'\n\
                   import Sibling from '../shared/Sib.vue'\n\
                   import Aliased from '@/Alias.vue'\n\
                   import { ref } from 'vue'\n\
                   const _u = App\n\
                   const _v = Sibling\n\
                   const _w = Aliased\n\
                   const _r = ref(0)\n\
                   </script>\n\
                   <template><div></div></template>";

    let result = DiagnosticService::generate_virtual_ts(&uri, content, false, false)
        .expect("virtual ts generated");

    assert!(
        !result.code.contains("'./app.vue'"),
        "expected relative .vue import to be rewritten, got:\n{}",
        result.code,
    );
    assert!(
        result.code.contains("'./app.vue.ts'"),
        "expected rewritten relative specifier, got:\n{}",
        result.code,
    );
    assert!(
        result.code.contains("'../shared/Sib.vue.ts'"),
        "expected rewritten parent-path specifier, got:\n{}",
        result.code,
    );
    assert!(
        result.code.contains("'@/Alias.vue.ts'"),
        "expected rewritten alias specifier, got:\n{}",
        result.code,
    );
}

#[test]
fn editor_virtual_ts_for_inline_art_binds_self_to_host_props() {
    use crate::DiagnosticService;
    use tower_lsp::lsp_types::Url;

    let uri = Url::parse("file:///tmp/Button.vue").expect("parse uri");
    let content = r#"<script setup lang="ts">
defineProps<{ variant?: "primary" | "secondary" }>();
</script>

<template><button /></template>

<art>
  <variant name="Primary" default>
    <Self :variant="123" />
  </variant>
</art>"#;

    let results =
        DiagnosticService::generate_virtual_ts_for_inline_art_variants(&uri, content, false, false);

    assert_eq!(results.len(), 1);
    let (_, result) = &results[0];
    assert!(
        result
            .code
            .contains("declare const Self: { new (): { $props: Props } };"),
        "expected Self component binding, got:\n{}",
        result.code,
    );
    assert!(
        result.code.contains("type __Self_Props_0 = typeof Self"),
        "expected Self prop checks, got:\n{}",
        result.code,
    );
}

#[test]
fn editor_virtual_ts_for_art_imports_define_art_target_component() {
    use crate::DiagnosticService;
    use tower_lsp::lsp_types::Url;

    let uri = Url::parse("file:///tmp/Button.art.vue").expect("parse uri");
    let content = r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="Primary" default>
    <Button :variant="123" />
  </variant>
</art>"#;

    let result = DiagnosticService::generate_virtual_ts_for_art(&uri, content)
        .expect("virtual TS generated");

    assert!(
        result
            .code
            .contains("import __VizeArtTarget_Button from \"./Button.vue.ts\";"),
        "expected defineArt component import, got:\n{}",
        result.code,
    );
    assert!(
        result
            .code
            .contains("type __Button_Props_0 = typeof Button"),
        "expected Button prop checks, got:\n{}",
        result.code,
    );
}

#[test]
fn editor_virtual_ts_preserves_computed_map_value_types() {
    use std::path::{Path, PathBuf};

    use crate::DiagnosticService;
    use tower_lsp::lsp_types::Url;
    use vize_canon::{CorsaBridge, CorsaBridgeConfig};

    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().expect("temp project");
    let root = project.path();
    let src = root.join("src");
    let components = src.join("components");
    let utils = src.join("utils");
    let types = src.join("types");
    std::fs::create_dir_all(&components).expect("components dir");
    std::fs::create_dir_all(&utils).expect("utils dir");
    std::fs::create_dir_all(&types).expect("types dir");

    std::fs::write(
        root.join("tsconfig.json"),
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
    .expect("tsconfig");
    std::fs::write(
        src.join("vue.d.ts"),
        r#"declare module "vue" {
  export interface Ref<T = unknown, _Raw = T> { value: T }
  export interface ComputedRef<T = unknown> extends Ref<T> {}
  export interface ShallowRef<T = unknown, _Raw = T> extends Ref<T> {}
  export interface ComponentPublicInstance {
    $attrs: Record<string, unknown>;
    $slots: Record<string, (...args: any[]) => any>;
    $refs: Record<string, any>;
    $emit: (...args: any[]) => void;
  }
  export function computed<T>(getter: () => T): ComputedRef<T>;
}
"#,
    )
    .expect("vue shim");
    std::fs::write(
        types.join("index.ts"),
        "export interface SpeakerWithYear { name: string; year: number; title: string }\n",
    )
    .expect("types");
    std::fs::write(
        utils.join("speakerMap.ts"),
        r#"import type { SpeakerWithYear } from "../types";

export interface SpeakerRecord {
  name: string;
  talks: SpeakerWithYear[];
}

export function buildSpeakerMap(allSpeakers: SpeakerWithYear[]): Map<string, SpeakerRecord> {
  const map = new Map<string, SpeakerRecord>();
  for (const speaker of allSpeakers) {
    const record = map.get(speaker.name) ?? { name: speaker.name, talks: [] };
    record.talks.push(speaker);
    map.set(speaker.name, record);
  }
  return map;
}
"#,
    )
    .expect("speaker map");

    let vue_path = components.join("DirectoryView.vue");
    let content = r#"<script setup lang="ts">
import { computed } from "vue";
import type { SpeakerWithYear } from "../types";
import { buildSpeakerMap } from "../utils/speakerMap";

const props = defineProps<{ allSpeakers: SpeakerWithYear[] }>();
const speakerMap = computed(() => buildSpeakerMap(props.allSpeakers));
const allRecords = computed(() => Array.from(speakerMap.value.values()));
const speakerOptions = computed(() =>
  allRecords.value.map((record) => ({
    label: `${record.name} (${record.talks.length})`,
    value: record.name,
  })),
);
</script>

<template>
  <div />
</template>
"#;
    std::fs::write(&vue_path, content).expect("vue file");

    let uri = Url::from_file_path(&vue_path).expect("file uri");
    let virtual_result = DiagnosticService::generate_virtual_ts(&uri, content, false, false)
        .expect("virtual TS generated");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let diagnostics = crate::runtime::block_on(async {
        if bridge.spawn().await.is_err() {
            return None;
        }
        let virtual_name = format!("{}.ts", vue_path.display());
        let open_result = bridge
            .open_or_update_virtual_document(&virtual_name, &virtual_result.code)
            .await;
        if open_result.is_err() {
            let _ = bridge.shutdown().await;
            return None;
        }
        let diagnostics = bridge.get_diagnostics(&virtual_name).await.ok();
        let _ = bridge.shutdown().await;
        diagnostics
    });

    let Some(diagnostics) = diagnostics else {
        return;
    };
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("'record' is of type 'unknown'")),
        "unexpected record unknown diagnostic: {diagnostics:#?}\nvirtual TS:\n{}",
        virtual_result.code,
    );
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:#?}\nvirtual TS:\n{}",
        virtual_result.code,
    );

    fn resolve_test_tsgo_binary() -> Option<PathBuf> {
        if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
            return None;
        }

        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)?;
        let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
        if sibling_cache.exists() {
            return Some(sibling_cache);
        }

        vize_carton::corsa_resolver::discover_corsa_in_ancestors(workspace_root)
    }
}
