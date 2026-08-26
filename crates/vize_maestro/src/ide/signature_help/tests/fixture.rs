use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use crate::server::ServerState;

pub(super) struct Fixture {
    pub(super) root: tempfile::TempDir,
    corsa_path: PathBuf,
    #[cfg(unix)]
    protocol_trace_dir: Option<PathBuf>,
}

impl Fixture {
    pub(super) fn new(corsa_path: &Path) -> Self {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("src")).unwrap();
        write_project(root.path());
        Self {
            root,
            corsa_path: corsa_path.to_path_buf(),
            #[cfg(unix)]
            protocol_trace_dir: None,
        }
    }

    #[cfg(unix)]
    pub(super) fn new_traced(corsa_path: &Path) -> Self {
        let mut fixture = Self::new(corsa_path);
        let (wrapper, trace_dir, shutdown_gate) =
            crate::ide::rename::corsa_session_tests::harness::protocol::traced_corsa_executable(
                fixture.root.path(),
                corsa_path,
                false,
            )
            .expect("prepare ART signature protocol trace");
        assert!(shutdown_gate.is_none());
        fixture.corsa_path = wrapper;
        fixture.protocol_trace_dir = Some(trace_dir);
        fixture
    }

    #[cfg(not(unix))]
    pub(super) fn new_traced(corsa_path: &Path) -> Self {
        Self::new(corsa_path)
    }

    #[cfg(unix)]
    pub(super) fn describe_server_frames(&self) -> String {
        self.protocol_trace_dir.as_ref().map_or_else(
            || "server frames were not traced".to_owned(),
            |trace_dir| {
                crate::ide::rename::corsa_session_tests::harness::evidence::describe_server_frames_in(
                    trace_dir,
                )
            },
        )
    }

    #[cfg(not(unix))]
    pub(super) fn describe_server_frames(&self) -> String {
        "server frames require the Unix byte proxy".to_owned()
    }

    pub(super) fn vue(&self, name: &str, source: &str) -> (ServerState, Url) {
        self.open(name, source, "vue")
    }

    pub(super) fn tsx(&self, name: &str, source: &str) -> (ServerState, Url) {
        self.open(name, source, "typescriptreact")
    }

    fn open(&self, name: &str, source: &str, language_id: &str) -> (ServerState, Url) {
        let path = self.root.path().join("src").join(name);
        std::fs::write(&path, source).unwrap();
        let uri = Url::from_file_path(path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(self.root.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, language_id.to_string());
        state.update_virtual_docs(&uri, source);
        (state, uri)
    }

    pub(super) fn bridge(&self) -> Arc<CorsaBridge> {
        Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(self.corsa_path.clone()),
            working_dir: Some(self.root.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }))
    }
}

fn write_project(root: &Path) {
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    let vue = root.join("node_modules/vue");
    std::fs::create_dir_all(&vue).unwrap();
    std::fs::write(
        vue.join("package.json"),
        r#"{"name":"vue","version":"3.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        vue.join("index.d.ts"),
        r#"export type DefineComponent<P = any, _B = any, _D = any> = { new(): { $props: P } };
export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
export interface Ref<T = unknown, _Raw = T> { value: T }
export interface ShallowRef<T = unknown> extends Ref<T> {}
"#,
    )
    .unwrap();
}

pub(super) fn resolve_tsgo_binary() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    [
        workspace_root.parent()?.join("corsa-bind/.cache/tsgo"),
        workspace_root
            .parent()?
            .join("corsa-bind/ref/corsa-upstream/.cache/tsgo"),
        workspace_root.join("node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
    .or_else(|| vize_s0::corsa_resolver::discover_corsa_in_ancestors(workspace_root))
}
