use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn check_options_api_props_spread_with_instance_members_has_no_ts_parse_errors() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/OptionsApiPropsSpread.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let diagnostics = json["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["diagnostics"].as_array().cloned().unwrap_or_default())
        .filter_map(|diagnostic| diagnostic.as_str().map(str::to_owned))
        .collect::<Vec<_>>();

    let parse_error_codes = diagnostics
        .iter()
        .filter_map(|diagnostic| ts_diagnostic_code(diagnostic))
        .filter(|code| matches!(code, 1109 | 1128 | 1131))
        .collect::<Vec<_>>();
    assert_eq!(
        parse_error_codes,
        Vec::<u32>::new(),
        "Options API props spread must not generate virtual-TS syntax errors; got {diagnostics:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[cfg(feature = "legacy")]
#[test]
fn check_legacy_vue2_options_api_prop_type_matrix_cli() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_prop_type_matrix_project();
    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/App.vue",
            "--config",
            "vize.config.json",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], 0);
    assert_eq!(json["fileCount"], 1);
    let files = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["file"].as_str().unwrap().to_owned(),
                file["diagnostics"].as_array().unwrap().clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(files, vec![("src/App.vue".to_owned(), Vec::new())]);

    let _ = std::fs::remove_dir_all(&project_root);
}

fn ts_diagnostic_code(diagnostic: &str) -> Option<u32> {
    let marker = diagnostic.find("[TS")?;
    let start = marker + 3;
    let end = diagnostic[start..].find(']')? + start;
    diagnostic[start..end].parse().ok()
}

fn create_cli_project() -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(format!("options-api-props-spread-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_node_modules(&project_root);
    std::fs::write(
        project_root.join("tsconfig.json"),
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
    std::fs::write(
        project_root.join("src/OptionsApiPropsSpread.vue"),
        r#"<script lang="ts">
import { defineComponent } from 'vue'

const sharedProps = {
  meta: { type: Object, required: true as const },
}
function useFakeStore() {
  return { cached: (s: string, _b: boolean) => s }
}
export default defineComponent({
  props: { ...sharedProps },
  setup() {
    const store = useFakeStore()
    return { store }
  },
  data() {
    return { missing: false }
  },
  computed: {
    url() {
      return this.store.cached('x', false)
    },
  },
  methods: {
    onError(_a: unknown) {
      this.missing = true
    },
  },
})
</script>
"#,
    )
    .unwrap();
    project_root
}

#[cfg(feature = "legacy")]
fn create_prop_type_matrix_project() -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(format!(
            "options-api-prop-type-matrix-{}",
            std::process::id()
        ));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_node_modules(&project_root);
    std::fs::write(
        project_root.join("vize.config.json"),
        r#"{
  "vue": { "version": "2.7" },
  "typeChecker": {
    "legacyVue2": true
  }
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
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
    std::fs::write(
        project_root.join("src/types.ts"),
        r#"export type ImportedItem = { id: string; count: number }
export type ImportedStatus = "ready" | "draft"
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script lang="ts">
import { defineComponent, type PropType as VuePropType } from 'vue'
import type { ImportedItem, ImportedStatus } from './types'

type LocalItem = ImportedItem & { local: boolean }
type LocalPropType<T> = VuePropType<T>
type NestedShape = { nested: { id: string; status: ImportedStatus } }

const nestedObjectProp = {
  type: Object as LocalPropType<NestedShape>,
  required: true,
}

export default defineComponent({
  props: {
    status: { type: String as VuePropType<ImportedStatus | "archived">, required: true },
    selected: { type: Object as VuePropType<ImportedItem & { enabled: boolean }>, required: true },
    items: { type: Array as LocalPropType<ReadonlyArray<LocalItem>>, required: true },
    readonlyItems: { type: Array as VuePropType<readonly ImportedItem[]>, required: true },
    formatter: { type: Function as VuePropType<(item: ImportedItem) => string>, required: true },
    nestedObject: nestedObjectProp,
  },
  setup(props) {
    const formatted = props.formatter(props.selected)
    const firstId = props.items[0]?.id
    const readonlyCount = props.readonlyItems[0]?.count ?? 0
    const nestedId = props.nestedObject?.nested.id ?? ""
    return { formatted, firstId, readonlyCount, nestedId }
  },
})
</script>

<template>
  <div>
    {{ status }}
    {{ selected.id }}
    {{ items[0]?.local }}
    {{ readonlyItems[0]?.count }}
    {{ nestedObject?.nested.status }}
    {{ formatted }}
    {{ firstId }}
    {{ readonlyCount }}
    {{ nestedId }}
  </div>
</template>
"#,
    )
    .unwrap();
    project_root
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn link_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    if source.exists() {
        symlink_path(&source, &project_root.join("node_modules")).unwrap();
    }
}

fn resolve_test_corsa_path() -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path.display().to_string());
        }
    }
    let workspace_root = workspace_root();
    [workspace_root.join("node_modules/.bin/tsgo")]
        .into_iter()
        .find(|candidate| candidate.exists())
        .map(|candidate| candidate.display().to_string())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
