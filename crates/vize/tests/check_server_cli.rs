#![cfg(unix)]

use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::Value;
use vize_s0::corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable};

#[path = "support/vue_stub.rs"]
mod vue_stub;

const APP: &str = r#"<script setup lang="ts">
import Child from './Child.vue'
const count = 1
</script>

<template>
  <Child data-label="😀" :count="count" />
</template>
"#;
const NUMERIC_CHILD: &str = r#"<script setup lang="ts">
defineProps<{ count: number }>()
</script>
"#;
const STRING_CHILD: &str = r#"<script setup lang="ts">
defineProps<{ count: string }>()
</script>
"#;

struct CheckServer {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: std::io::BufReader<ChildStdout>,
}

impl CheckServer {
    fn spawn(project_root: &Path, corsa_path: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vize"))
            .args([
                "check-server",
                "--corsa-path",
                corsa_path.to_str().unwrap(),
                "--working-dir",
                project_root.to_str().unwrap(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = std::io::BufReader::new(child.stdout.take().unwrap());
        Self {
            child: Some(child),
            stdin: Some(stdin),
            stdout,
        }
    }

    fn check(&mut self, id: u64, uri: &Path, content: &str) -> Value {
        self.request(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "check",
            "params": { "uri": uri, "content": content }
        }))
    }

    fn request(&mut self, request: Value) -> Value {
        let stdin = self.stdin.as_mut().unwrap();
        writeln!(stdin, "{request}").unwrap();
        stdin.flush().unwrap();
        let mut response = String::new();
        self.stdout.read_line(&mut response).unwrap();
        serde_json::from_str(&response).unwrap()
    }

    fn shutdown(mut self) {
        let response = self.request(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "shutdown"
        }));
        assert_eq!(response["result"]["status"], "shutdown");
        self.stdin.take();
        assert!(self.child.take().unwrap().wait().unwrap().success());
    }
}

impl Drop for CheckServer {
    fn drop(&mut self) {
        self.stdin.take();
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[test]
fn check_server_maps_dependency_patch_diagnostics_to_the_parent_sfc() {
    let Some(corsa_path) = resolve_corsa_executable(CorsaResolveRequest {
        explicit_path: None,
        project_root: Some(workspace_root()),
    })
    .ok() else {
        return;
    };
    let project_root = create_project();
    let app_path = project_root.join("src/App.vue");
    let child_path = project_root.join("src/Child.vue");
    let mut server = CheckServer::spawn(&project_root, &corsa_path);

    let clean = server.check(1, &app_path, APP);
    assert_eq!(clean["result"]["diagnostics"], serde_json::json!([]));

    std::fs::write(&child_path, STRING_CHILD).unwrap();
    let broken = server.check(2, &app_path, APP);
    assert_eq!(broken["result"]["errorCount"], 1, "{broken:#}");
    assert_eq!(
        broken["result"]["diagnostics"],
        serde_json::json!([{
            "message": "Type 'number' is not assignable to type 'string'.",
            "severity": "error",
            "line": 7,
            "column": 27,
            "code": "TS2322"
        }])
    );

    std::fs::write(&child_path, NUMERIC_CHILD).unwrap();
    let repaired = server.check(3, &app_path, APP);
    assert_eq!(repaired["result"]["diagnostics"], serde_json::json!([]));

    server.shutdown();
    std::fs::remove_dir_all(project_root).unwrap();
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn create_project() -> PathBuf {
    let project_root = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("check-server-patch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    vue_stub::install_vue_jsx_type_stub(&project_root);
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
    std::fs::write(project_root.join("src/App.vue"), APP).unwrap();
    std::fs::write(project_root.join("src/Child.vue"), NUMERIC_CHILD).unwrap();
    project_root
}
