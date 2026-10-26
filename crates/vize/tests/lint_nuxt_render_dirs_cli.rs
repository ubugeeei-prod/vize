//! `vize lint --cross-file` reads Nuxt `srcDir` / `dir.*` like the doctor (#10808).
#![cfg(test)]

use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

/// A layout provides a key that a component below a page injects.
fn project(root: &Path, nuxt_config: &str) {
    write(root, "package.json", r#"{"private":true}"#);
    write(root, "nuxt.config.ts", nuxt_config);
    write(
        root,
        "src/features/keys.ts",
        r#"import type { InjectionKey } from "vue";
export const NoticeKey: InjectionKey<{ show(text: string): void }> = Symbol("notice");
"#,
    );
    write(
        root,
        "src/nuxt/app/app.vue",
        "<template>\n  <NuxtLayout>\n    <NuxtPage />\n  </NuxtLayout>\n</template>\n",
    );
    write(
        root,
        "src/nuxt/layouts/default.vue",
        r#"<script setup lang="ts">
import { provide } from "vue";
import { NoticeKey } from "../../features/keys";

provide(NoticeKey, { show: text => console.warn(text) });
</script>

<template>
  <main><slot /></main>
</template>
"#,
    );
    write(
        root,
        "src/features/NoticeButton.vue",
        r#"<script setup lang="ts">
import { inject } from "vue";
import { NoticeKey } from "./keys";

const notice = inject(NoticeKey)!;
</script>

<template>
  <button type="button" @click="notice.show('hi')">notify</button>
</template>
"#,
    );
    write(
        root,
        "src/nuxt/pages/index.vue",
        r#"<script setup lang="ts">
import NoticeButton from "../../features/NoticeButton.vue";
</script>

<template>
  <NoticeButton />
</template>
"#,
    );
}

/// Every cross-file message as `file line:column message`.
fn cross_file_messages(root: &Path) -> Vec<Value> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["lint", "src/**/*.vue", "--no-config", "--cross-file"])
        .args(["--format", "json", "--help-level", "none"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    report
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            file["messages"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |message| {
                    json!([
                        file["file"],
                        message["line"],
                        message["column"],
                        message["message"]
                    ])
                })
        })
        .collect()
}

#[test]
fn custom_nuxt_dirs_keep_layout_provides_matched() {
    let directory = tempfile::tempdir().unwrap();
    project(
        directory.path(),
        r#"export default defineNuxtConfig({
  ssr: false,
  srcDir: "src",
  dir: { layouts: "nuxt/layouts", pages: "nuxt/pages", app: "nuxt/app" }
});
"#,
    );

    assert_eq!(cross_file_messages(directory.path()), Vec::<Value>::new());
}

#[test]
fn default_nuxt_dirs_leave_the_custom_layout_outside_the_render_tree() {
    let directory = tempfile::tempdir().unwrap();
    project(
        directory.path(),
        "export default defineNuxtConfig({ ssr: false, srcDir: \"src\" });\n",
    );

    assert_eq!(
        cross_file_messages(directory.path()),
        [
            json!([
                "src/features/NoticeButton.vue",
                5,
                16,
                "[vize:cross-file] vize:croquis/cf/unmatched-inject: **Unmatched Inject**: `inject(NoticeKey)` has no matching `provide()` in any ancestor component and returns `undefined` at runtime."
            ]),
            json!([
                "src/nuxt/layouts/default.vue",
                5,
                1,
                "[vize:cross-file] vize:croquis/cf/unused-provide: provide(NoticeKey) is not used by any descendant component"
            ]),
        ]
    );
}
