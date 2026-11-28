//! Package-branch fixtures shared by editor fallback and publication tests.
#![expect(
    clippy::disallowed_types,
    reason = "native LSP fixture APIs retain std String and shared Arc bridge handles"
)]

use crate::server::ServerState;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

pub(crate) struct Workspace {
    pub(crate) directory: tempfile::TempDir,
    pub(crate) uri: Url,
}

impl Workspace {
    pub(crate) fn new(custom: bool) -> Self {
        let directory = super::tempdir().expect("physical fixture directory");
        let root = directory.path();
        write(root, "package.json", r#"{"private":true,"type":"module"}"#);
        write(
            root,
            "tsconfig.json",
            &serde_json::to_string(&serde_json::json!({
                "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext",
                    "moduleResolution": "Bundler", "skipLibCheck": true, "noEmit": true,
                    "customConditions": if custom { vec!["custom"] } else { vec![] } },
                "include": ["src/**/*"]
            }))
            .expect("fixture compiler options"),
        );
        write(
            root,
            "src/vue.d.ts",
            r#"declare module "vue" {
  export type DefineComponent<P = any, _B = any, _D = any> = { new(): { $props: P } };
  export interface Ref<T = unknown> { value: T }
  export interface ShallowRef<T = unknown> extends Ref<T> {}
}
"#,
        );
        write(
            root,
            "node_modules/@scope/ui/package.json",
            r#"{
  "name":"@scope/ui", "type":"module", "exports": {
    "./Alpha.vue": "./src/Alpha.vue", "./Bravo.vue": "./src/Bravo.vue",
    "./*.vue": { "custom":"./src/Custom.vue", "types":"./src/Level11.vue", "default":"./src/Wrong.vue" }
  }
}"#,
        );
        for (name, ty, generic) in [
            ("Level11", "number", true),
            ("Custom", "string", false),
            ("Wrong", "boolean", false),
            ("Alpha", "number", true),
            ("Bravo", "boolean", true),
        ] {
            let parameters = if generic {
                " generic=\"T extends { id: number }\""
            } else {
                ""
            };
            let items = if generic {
                " items?: readonly T[];"
            } else {
                ""
            };
            let source = vize_l0::cstr!(
                "<script setup lang=\"ts\"{parameters}>\ndefineProps<{{ depth: {ty};{items} }}>();\n</script>\n<template><div /></template>\n"
            );
            write(
                root,
                vize_l0::cstr!("node_modules/@scope/ui/src/{name}.vue").as_str(),
                &source,
            );
        }
        let uri = Url::from_file_path(root.join("src/App.vue")).expect("source URI");
        Self { directory, uri }
    }

    pub(crate) fn target(&self, name: &str) -> PathBuf {
        self.directory
            .path()
            .join(vize_l0::cstr!("node_modules/@scope/ui/src/{name}.vue").as_str())
            .canonicalize()
            .expect("target exists")
    }

    pub(crate) fn source(name: &str) -> String {
        vize_l0::cstr!(
            "<script setup lang=\"ts\">\nconst unicode = '雪🌸';\nimport Child from '@scope/ui/{name}.vue';\n</script>\n<template><Child :depth=\"1\" /></template>\n"
        ).into()
    }

    pub(crate) fn open(&self, state: &ServerState, source: &str, version: i32) {
        if state.get_workspace_root().as_deref() != Some(self.directory.path()) {
            state.set_workspace_root(self.directory.path().to_path_buf());
        }
        write(self.directory.path(), "src/App.vue", source);
        state
            .documents
            .open(self.uri.clone(), source.into(), version, "vue".into());
        state.update_virtual_docs(&self.uri, source);
    }

    pub(crate) async fn bridge(&self, runtime: PathBuf) -> Arc<CorsaBridge> {
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(runtime),
            working_dir: Some(self.directory.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("real native provider starts");
        bridge
    }
}

fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("parent exists");
    fs::write(path, source).expect("fixture source written");
}
