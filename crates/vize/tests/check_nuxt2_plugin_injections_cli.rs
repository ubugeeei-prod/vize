use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn check_nuxt2_use_context_sees_plugin_injections() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project("nuxt2-plugin-injections");

    write_file(&project_root, "nuxt.config.ts", "export default {}\n");
    write_file(
        &project_root,
        "tsconfig.json",
        r##"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "noEmit": true
  },
  "include": ["pages/**/*.vue", "plugins/**/*.ts", "types/**/*.d.ts"]
}"##,
    );
    write_file(
        &project_root,
        "types/nuxt.d.ts",
        r##"declare module "@nuxt/types" {
  export interface Context {}
  export interface NuxtAppOptions {}
}

declare module "@nuxtjs/composition-api" {
  export interface UseContextReturn
    extends Omit<import("@nuxt/types").Context, "route" | "query" | "from" | "params"> {}
  export function useContext(): UseContextReturn;
}

declare module "#app" {
  export interface NuxtApp {}
}
"##,
    );
    write_file(
        &project_root,
        "plugins/logger.ts",
        r#"export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("logger", {
    info(message: string) {
      return message.length;
    },
  });
};
"#,
    );
    write_file(
        &project_root,
        "pages/index.vue",
        r#"<script setup lang="ts">
import { useContext } from "@nuxtjs/composition-api";

const context = useContext();
context.$logger.info("ready");
</script>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "pages",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "Nuxt2 useContext plugin injections should type-check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[cfg(feature = "legacy")]
#[test]
fn check_nuxt2_false_positive_fixture_matrix() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project("nuxt2-false-positive-matrix");

    write_file(
        &project_root,
        "vize.config.json",
        r#"{
  "vue": { "version": "2.7" },
  "typeChecker": { "legacyVue2": true }
}"#,
    );
    write_file(&project_root, "nuxt.config.ts", "export default {}\n");
    write_file(
        &project_root,
        "tsconfig.json",
        r##"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "paths": { "~/*": ["*"] },
    "noEmit": true
  },
  "include": ["components/**/*.vue", "pages/**/*.vue", "plugins/**/*.ts", "types/**/*.d.ts"]
}"##,
    );
    write_file(
        &project_root,
        "types/nuxt.d.ts",
        r##"declare module "@nuxt/types" {
  export interface Context {}
  export interface NuxtAppOptions {}
}

declare module "@nuxtjs/composition-api" {
  export interface UseContextReturn
    extends Omit<import("@nuxt/types").Context, "route" | "query" | "from" | "params"> {}
  export function useContext(): UseContextReturn;
}

declare module "#app" {
  export interface NuxtApp {}
}
"##,
    );
    write_file(
        &project_root,
        "plugins/logger.ts",
        r#"export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("logger", {
    info(message: string) {
      return message.length;
    },
  });
};
"#,
    );
    write_file(
        &project_root,
        "plugins/auth.ts",
        r#"export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("auth", {
    loggedIn: true,
    userName() {
      return "Ada";
    },
  });
};
"#,
    );
    write_file(
        &project_root,
        "plugins/gtm.ts",
        r#"export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("gtm", {
    push(event: { name: string }) {
      return event.name;
    },
  });
};
"#,
    );
    write_file(
        &project_root,
        "plugins/repository.ts",
        r#"export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("accountRepository", {
    find(id: string) {
      return { id, label: id.toUpperCase() };
    },
  });
};
"#,
    );
    write_file(
        &project_root,
        "components/KeyboardPanel.vue",
        r#"<script lang="ts">
import { defineComponent, type PropType } from 'vue'

export type Folder = { id: string; label: string }

export default defineComponent({
  props: {
    folders: {
      type: Array as PropType<Folder[]>,
      required: true,
    },
  },
  emits: {
    'click-folder': (_folder: Folder) => true,
    'input:math-key': (_key: string) => true,
  },
  methods: {
    clickFirst() {
      this.$emit('click-folder', this.folders[0])
    },
    inputKey() {
      this.$emit('input:math-key', 'plus')
    },
  },
})
</script>

<template>
  <button @click="clickFirst">{{ folders[0].label }}</button>
</template>
"#,
    );
    write_file(
        &project_root,
        "components/OverlayPanel.vue",
        r#"<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  emits: {
    'update:is-opened-overlay-loading': (_value: boolean) => true,
  },
  methods: {
    close() {
      this.$emit('update:is-opened-overlay-loading', false)
    },
  },
})
</script>

<template>
  <button @click="close">Close</button>
</template>
"#,
    );
    write_file(
        &project_root,
        "pages/index.vue",
        r#"<script lang="ts">
import { defineComponent, type PropType } from 'vue'
import { useContext } from '@nuxtjs/composition-api'
import KeyboardPanel, { type Folder } from '~/components/KeyboardPanel.vue'
import OverlayPanel from '~/components/OverlayPanel.vue'

const componentProps = {
  folders: {
    type: Array as PropType<Folder[]>,
    required: true,
  },
}

function usePanelState() {
  return {
    panelTitle: 'Folders',
    selectedId: 'root',
  }
}

export default defineComponent({
  components: { KeyboardPanel, OverlayPanel },
  props: componentProps,
  setup(props, { emit }) {
    const context = useContext()
    const repoItem = context.$accountRepository.find(props.folders[0].id)
    context.$logger.info(context.$auth.userName())
    context.$gtm.push({ name: repoItem.label })

    const onClickFolder = (folder: Folder) => {
      emit('click-folder', folder)
      context.$logger.info(folder.label)
    }
    const onInputMathKey = (key: string) => {
      emit('input:math-key', key)
    }
    const onUpdateIsOpenedOverlayLoading = (value: boolean) => {
      emit('update:is-opened-overlay-loading', value)
    }

    return {
      ...usePanelState(),
      repoLabel: repoItem.label,
      onClickFolder,
      onInputMathKey,
      onUpdateIsOpenedOverlayLoading,
    }
  },
})
</script>

<template>
  <section>
    <h1>{{ panelTitle }} {{ selectedId }} {{ repoLabel }}</h1>
    <KeyboardPanel
      :folders="folders"
      @click-folder="onClickFolder"
      @input:math-key="onInputMathKey"
    />
    <OverlayPanel
      @update:is-opened-overlay-loading="onUpdateIsOpenedOverlayLoading"
    />
  </section>
</template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "components",
            "pages",
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
        "Nuxt2 false-positive matrix should type-check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], 0);
    assert_eq!(json["fileCount"], 3);
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
    assert_eq!(
        files,
        vec![
            ("components/KeyboardPanel.vue".to_owned(), Vec::new()),
            ("components/OverlayPanel.vue".to_owned(), Vec::new()),
            ("pages/index.vue".to_owned(), Vec::new()),
        ]
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project(name: &str) -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root);
    project_root
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
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
