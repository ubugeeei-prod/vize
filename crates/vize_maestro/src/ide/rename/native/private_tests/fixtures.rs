//! Authored fixtures and full native transaction inputs.

use crate::server::ServerState;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tower_lsp::lsp_types::Url;
use vize_l0::cstr;

pub(super) const LOCAL: &str = "<script setup lang=\"ts\">import { ref } from 'vue';\nconst message = ref('hello');\nconst broken: string = 42; void broken;</script>\n<template>🚀 {{ message }} {{ message }}</template>";
const CHILD: &str = "<script setup lang=\"ts\">defineProps<{ message: string }>();</script><template>{{ message }}</template>";

pub(super) fn project(source: &str, filename: &str) -> (tempfile::TempDir, Url) {
    let directory = crate::test_support::tempdir().expect("physical temp project");
    fs::write(directory.path().join(filename), source).expect("authored source");
    fs::write(directory.path().join("Child.vue"), CHILD).expect("child source");
    fs::write(
        directory.path().join("shared.ts"),
        "export const message = 1;",
    )
    .expect("shared module");
    fs::write(
        directory.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["./**/*"]}"#,
    ).expect("authored config");
    let uri = Url::from_file_path(directory.path().join(filename)).expect("authored URI");
    (directory, uri)
}

pub(super) fn opened(root: &Path, uri: &Url, source: &str) -> ServerState {
    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());
    let child_uri = Url::from_file_path(root.join("Child.vue")).expect("child URI");
    for (uri, source) in [(uri, source), (&child_uri, CHILD)] {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        state.update_virtual_docs(uri, source);
    }
    state
}

pub(super) const PUBLIC_LOCAL: &str = "<script setup lang=\"ts\">import Child from './Child.vue'; const message = 'local';</script><template><Child :message=\"message\" /></template>";

pub(super) fn provider() -> Option<PathBuf> {
    let provider = crate::test_support::tsgo();
    if provider.is_none() {
        assert!(
            std::env::var_os("VIZE_TEST_DISABLE_TSGO")
                .is_some_and(|value| !value.is_empty() && value != "false"),
            "native provider is mandatory outside the explicit PR-disabled envelope",
        );
    }
    provider
}

pub(super) fn native_cases() -> [(std::string::String, &'static str, &'static str, bool); 13] {
    [
        (LOCAL.to_string(), "message =", "nextMessage", true),
        (LOCAL.to_string(), "message }}", "nextMessage", true),
        (LOCAL.replace('\n', "\r\n"), "message =", "nextMessage", true),
        ("<script setup lang=\"ts\">import { message } from './shared';</script><template>{{ message }}</template>".into(), "message }", "next", false),
        ("<script setup lang=\"ts\">const { message } = defineProps<{ message: string }>();</script><template>{{ message }}</template>".into(), "message }", "next", false),
        ("<script setup lang=\"ts\">const message = defineModel<string>();</script><template>{{ message }}</template>".into(), "message =", "next", false),
        ("<script setup lang=\"ts\">const message = 1; defineExpose({ message });</script><template>{{ message }}</template>".into(), "message =", "next", false),
        ("<script setup lang=\"ts\">const message = 1; const value = { message };</script><template>{{ message }}</template>".into(), "message =", "next", false),
        ("<script setup lang=\"ts\">const message = 1; const value = { message: 2 }; void value.message;</script><template>{{ message }}</template>".into(), "message;", "next", false),
        ("<script setup lang=\"ts\">const message = 1; const f = (next: number) => message;</script><template>{{ message }}</template>".into(), "message =", "next", false),
        ("<script setup lang=\"ts\">const message = 1;</script><template>{{ message }}</template><style>.x{color:v-bind(message)}</style>".into(), "message =", "next", false),
        (PUBLIC_LOCAL.into(), "message =", "nextMessage", true),
        (PUBLIC_LOCAL.into(), "message\"", "nextMessage", true),
    ]
}

pub(super) fn expected_private_source(source: &str, new_name: &str) -> std::string::String {
    if source == PUBLIC_LOCAL {
        // Only the local declaration and bound expression change. The public
        // prop key must remain byte-exact, unlike a blanket spelling replace.
        source
            .replacen("const message =", &cstr!("const {new_name} ="), 1)
            .replace(":message=\"message\"", &cstr!(":message=\"{new_name}\""))
    } else {
        source.replace("message", new_name)
    }
}
