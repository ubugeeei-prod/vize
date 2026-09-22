//! `vize lint --format rich` (P4-14a): lint results through the Davinci
//! diagnostic renderer, in the locale `--locale` selects, pinned end to end.
//! Output is piped, so the renderer runs colourless.

use std::{fs, path::Path, process::Command};

const TODO_LIST: &str = r#"<script setup lang="ts">
import { ref } from "vue"

const todos = ref([{ id: 1, title: "Write the talk" }])
</script>

<template>
  <ul>
    <li v-for="todo in todos">{{ todo.title }}</li>
  </ul>
</template>
"#;

fn lint(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("NO_COLOR", "1")
        .args(["lint", "--no-config", "--format", "rich"])
        .args(args)
        .output()
        .expect("vize runs")
}

fn project() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temp project");
    fs::create_dir_all(root.path().join("src")).expect("src dir");
    fs::write(root.path().join("src/TodoList.vue"), TODO_LIST).expect("fixture");
    root
}

#[test]
fn rich_output_renders_the_excerpt_in_english_by_default() {
    let root = project();
    let output = lint(root.path(), &["--help-level", "short", "src/TodoList.vue"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        std::str::from_utf8(&output.stdout).expect("UTF-8 stdout"),
        "error[vue/require-v-for-key]: Elements in iteration expect to have 'v-bind:key' directives. Element: <li>
 --> src/TodoList.vue:9:9
  |
9 |     <li v-for=\"todo in todos\">{{ todo.title }}</li>
  |         ^^^^^^^^^^^^^^^^^^^^^
  |
  = help: Why: The :key attribute helps Vue's virtual DOM efficiently track and update list items.

1 error and 0 warnings in 1 file
"
    );
}

#[test]
fn rich_output_speaks_the_requested_locale_end_to_end() {
    let root = project();
    let output = lint(root.path(), &["--locale", "ja", "src/TodoList.vue"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        std::str::from_utf8(&output.stdout).expect("UTF-8 stdout"),
        "エラー[vue/require-v-for-key]: `v-for` で繰り返す <li> には `v-bind:key` が必要です
 --> src/TodoList.vue:9:9
  |
9 |     <li v-for=\"todo in todos\">{{ todo.title }}</li>
  |         ^^^^^^^^^^^^^^^^^^^^^
  |
  = ヒント: 各項目を一意に識別できる値を :key に指定してください

1 ファイルを検査し、エラー 1 件、警告 0 件が見つかりました
"
    );
}

#[test]
fn an_unknown_locale_is_a_usage_error() {
    let root = project();
    let output = lint(root.path(), &["--locale", "fr", "src/TodoList.vue"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        std::str::from_utf8(&output.stdout).expect("UTF-8 stdout"),
        ""
    );
    assert_eq!(
        std::str::from_utf8(&output.stderr).expect("UTF-8 stderr"),
        "Unknown locale 'fr'. Expected one of: en, ja, zh\n"
    );
}
