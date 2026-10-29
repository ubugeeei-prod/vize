use super::{build_cross_file_lint_output, result_for_file};
use std::fs;

#[test]
fn cross_file_nuxt_named_page_layout_owns_its_page_injections() {
    let dir = tempfile::tempdir().unwrap();
    let app_root = dir.path().join("app");
    let app = app_root.join("app.vue");
    let default_layout = app_root.join("layouts/default.vue");
    let bare_layout = app_root.join("layouts/bare.vue");
    let page = app_root.join("pages/login.vue");

    fs::create_dir_all(default_layout.parent().unwrap()).unwrap();
    fs::create_dir_all(page.parent().unwrap()).unwrap();
    fs::write(
        &app,
        "<template><NuxtLayout><NuxtPage /></NuxtLayout></template>\n",
    )
    .unwrap();
    fs::write(
        &default_layout,
        "<template><main><slot /></main></template>\n",
    )
    .unwrap();
    fs::write(
        &bare_layout,
        r#"<script setup lang="ts">
import { provide, ref } from 'vue'
import { BusyKey } from '../keys'

provide(BusyKey, ref(false))
</script>

<template><main><slot /></main></template>
"#,
    )
    .unwrap();
    fs::write(
        &page,
        r#"<script setup lang="ts">
import { inject } from 'vue'
import { BusyKey } from '../keys'

definePageMeta({ layout: 'bare' })
const busy = inject(BusyKey)
</script>

<template><p>{{ busy ? 'loading' : 'login' }}</p></template>
"#,
    )
    .unwrap();

    let files = [&app, &default_layout, &bare_layout, &page]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

    for file_name in ["bare.vue", "login.vue"] {
        let result = result_for_file(&output.results, file_name);
        assert!(
            result.diagnostics.is_empty(),
            "{file_name}: {:#?}",
            result.diagnostics
        );
    }
}
