use std::path::{Path, PathBuf};

use crate::server::ServerState;
use tower_lsp::lsp_types::{Diagnostic, Url};

pub(super) struct SpeakerFixture {
    pub(super) vue_path: PathBuf,
    pub(super) source: String,
}

pub(super) fn state_for_fixture(root: &Path, uri: &Url, source: &str) -> ServerState {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "lint": false,
        "ecosystem": false,
        "typecheck": true
    })));
    state.set_workspace_root(root.to_path_buf());
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state
}

pub(super) fn write_speaker_fixture(root: &Path) -> SpeakerFixture {
    let src = root.join("src");
    let components = src.join("components");
    let utils = src.join("utils");
    let types = src.join("types");
    std::fs::create_dir_all(&components).expect("components dir");
    std::fs::create_dir_all(&utils).expect("utils dir");
    std::fs::create_dir_all(&types).expect("types dir");
    write_vue_test_package(root);
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
    let source = r#"<script setup lang="ts">
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
  <div></div>
</template>
"#
    .to_string();
    std::fs::write(&vue_path, &source).expect("vue");
    SpeakerFixture { vue_path, source }
}

pub(super) fn write_vue_import_fixture(root: &Path) {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("src dir");
    write_vue_test_package(root);
    write_typecheck_tsconfig(root);
    std::fs::write(
        src.join("Child.vue"),
        r#"<script setup lang="ts">
defineProps<{ label?: string }>();
</script>

<template>
  <span>{{ label }}</span>
</template>
"#,
    )
    .expect("child vue");
    std::fs::write(
        src.join("Parent.vue"),
        r#"<script setup lang="ts">
import Child from "./Child.vue";

const selected = Child;
</script>

<template>
  <Child label="ready" />
</template>
"#,
    )
    .expect("parent vue");
}

pub(super) fn write_art_vue_import_fixture(root: &Path) {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("src dir");
    write_vue_test_package(root);
    write_typecheck_tsconfig(root);
    std::fs::write(
        src.join("Button.vue"),
        r#"<script setup lang="ts">
defineProps<{ label?: string }>();
</script>

<template>
  <button>{{ label }}</button>
</template>
"#,
    )
    .expect("button vue");
    std::fs::write(
        src.join("Button.art.vue"),
        r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="Primary" default>
    <Button :label="123" />
  </variant>
</art>
"#,
    )
    .expect("button art vue");
}

pub(super) fn write_corsa_config(root: &Path, corsa_path: &Path) {
    std::fs::write(
        root.join("vize.config.json"),
        serde_json::json!({
            "typeChecker": {
                "corsaPath": corsa_path.to_string_lossy()
            }
        })
        .to_string(),
    )
    .expect("vize config");
}

pub(super) fn assert_no_import_or_unknown_record_diagnostics(diagnostics: &[Diagnostic]) {
    assert!(
        diagnostics.iter().all(
            |diagnostic| !diagnostic.message.contains("Cannot find module")
                && !diagnostic.message.contains("'record' is of type 'unknown'")
        ),
        "unexpected import/type false positive: {diagnostics:#?}",
    );
}

pub(super) fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    if let Some(path) = std::env::var_os("VIZE_TEST_TSGO_PATH") {
        return Some(PathBuf::from(path));
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache);
    }
    let upstream_cache = workspace_root
        .parent()?
        .join("corsa-bind/ref/corsa-upstream/.cache/tsgo");
    if upstream_cache.exists() {
        return Some(upstream_cache);
    }

    vize_s0::corsa_resolver::discover_corsa_in_ancestors(workspace_root)
}

fn write_typecheck_tsconfig(root: &Path) {
    std::fs::write(
        root.join("tsconfig.json"),
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
}

pub(super) fn write_vue_test_package(root: &Path) {
    let vue_dir = root.join("node_modules/vue");
    std::fs::create_dir_all(&vue_dir).expect("vue package dir");
    std::fs::write(
        vue_dir.join("package.json"),
        r#"{"name":"vue","version":"3.0.0","types":"index.d.ts"}"#,
    )
    .expect("vue package json");
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export type DefineComponent<P = any, _B = any, _D = any> = { new(): { $props: P } };
export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
export interface Ref<T = unknown, _Raw = T> { value: T }
export interface ComputedRef<T = unknown> extends Ref<T> {}
export interface ShallowRef<T = unknown> extends Ref<T> {}
export function computed<T>(getter: () => T): ComputedRef<T>;
"#,
    )
    .expect("vue types");
}
