//! Native editor type-checking regressions.

use std::path::{Path, PathBuf};

use super::{DiagnosticService, sources};
use crate::server::ServerState;
use tower_lsp::lsp_types::Url;

struct SpeakerFixture {
    vue_path: PathBuf,
    source: String,
}

#[test]
fn sync_collect_does_not_surface_legacy_type_false_positives() {
    let project = tempfile::TempDir::new().expect("temp project");
    let fixture = write_speaker_fixture(project.path());
    let uri = Url::from_file_path(&fixture.vue_path).expect("file uri");
    let state = state_for_fixture(project.path(), &uri, &fixture.source);

    let diagnostics = DiagnosticService::collect(&state, &uri);

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.source.as_deref() != Some(sources::TYPE_CHECKER)),
        "sync native diagnostics must not run the legacy type checker: {diagnostics:#?}",
    );
    assert_no_import_or_unknown_record_diagnostics(&diagnostics);
}

#[test]
fn async_collect_preserves_imported_computed_callback_types() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let fixture = write_speaker_fixture(project.path());
    write_corsa_config(project.path(), &corsa_path);

    let uri = Url::from_file_path(&fixture.vue_path).expect("file uri");
    let state = state_for_fixture(project.path(), &uri, &fixture.source);
    state.load_workspace_config(project.path());

    let diagnostics = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));

    assert_no_import_or_unknown_record_diagnostics(&diagnostics);
    assert!(
        diagnostics.is_empty(),
        "expected clean editor diagnostics, got: {diagnostics:#?}",
    );
}

#[test]
fn async_collect_resolves_relative_vue_imports_in_script_setup() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    write_vue_import_fixture(project.path());
    write_corsa_config(project.path(), &corsa_path);

    let parent_path = project.path().join("src/Parent.vue");
    let source = std::fs::read_to_string(&parent_path).expect("parent source");
    let uri = Url::from_file_path(&parent_path).expect("file uri");
    let state = state_for_fixture(project.path(), &uri, &source);
    state.load_workspace_config(project.path());

    let diagnostics = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("Cannot find module")),
        "relative .vue imports must resolve via editor virtual mirrors: {diagnostics:#?}",
    );
    assert!(
        diagnostics.is_empty(),
        "expected clean editor diagnostics, got: {diagnostics:#?}",
    );
}

#[test]
fn virtual_ts_generates_template_less_sfc_mirror() {
    let uri = Url::parse("file:///tmp/SpeakerFilterBar.vue").expect("parse uri");
    let source = r#"<script setup lang="ts">
defineProps<{ selected: string }>();
</script>
"#;

    let result = DiagnosticService::generate_virtual_ts(&uri, source, false, false)
        .expect("virtual TS generated for template-less SFC");

    assert!(
        result.code.contains("export default __vize_component__;"),
        "expected a component module mirror, got:\n{}",
        result.code,
    );
}

fn state_for_fixture(root: &Path, uri: &Url, source: &str) -> ServerState {
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

fn write_speaker_fixture(root: &Path) -> SpeakerFixture {
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

fn write_vue_import_fixture(root: &Path) {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("src dir");
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

fn write_corsa_config(root: &Path, corsa_path: &Path) {
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

fn assert_no_import_or_unknown_record_diagnostics(
    diagnostics: &[tower_lsp::lsp_types::Diagnostic],
) {
    assert!(
        diagnostics.iter().all(
            |diagnostic| !diagnostic.message.contains("Cannot find module")
                && !diagnostic.message.contains("'record' is of type 'unknown'")
        ),
        "unexpected import/type false positive: {diagnostics:#?}",
    );
}

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
