use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeConfig};
use vize_s0::cstr;

use self::assertions::{assert_exact_event_edit, authored_text, prepare_range, strict_rename};
use super::super::canonical;
use crate::{ide::IdeContext, server::ServerState};

mod assertions;
pub(in crate::ide) mod evidence;
mod generation;
pub(in crate::ide) mod protocol;

const CHILD_SOURCE: &str = r#"<script setup lang="ts">
defineEmits<{ saveItem: [id: string] }>();
</script>
"#;

const PARENT_SOURCE: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
const handleSave = (id: string) => id;
</script>
<template>💥 <Child @save-item="handleSave" /></template>
"#;

pub(super) struct RealCorsaRenameSession {
    root: tempfile::TempDir,
    canonical_root: PathBuf,
    state: ServerState,
    bridge: Arc<CorsaBridge>,
    child_uri: Url,
    parent_uri: Url,
    protocol_trace_dir: PathBuf,
    shutdown_gate: Option<protocol::ShutdownGate>,
    expected_readiness_generations: usize,
    shutdown_complete: bool,
}

impl RealCorsaRenameSession {
    pub(super) fn new(corsa_path: &Path) -> Result<Self, String> {
        Self::new_inner(corsa_path, false)
    }

    pub(super) fn new_with_shutdown_gate(corsa_path: &Path) -> Result<Self, String> {
        Self::new_inner(corsa_path, true)
    }

    fn new_inner(corsa_path: &Path, observe_shutdown: bool) -> Result<Self, String> {
        let root = tempfile::TempDir::new().map_err(|error| error.to_string())?;
        let canonical_root = root.path().canonicalize().map_err(|error| {
            cstr!(
                "canonicalize session root {}: {error}",
                root.path().display()
            )
        })?;
        let src = canonical_root.join("src");
        fs::create_dir_all(&src).map_err(|error| error.to_string())?;
        fs::write(
            canonical_root.join("tsconfig.json"),
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
        .map_err(|error| error.to_string())?;

        let child_path = src.join("Child.vue");
        let parent_path = src.join("Parent.vue");
        fs::write(&child_path, CHILD_SOURCE).map_err(|error| error.to_string())?;
        fs::write(&parent_path, PARENT_SOURCE).map_err(|error| error.to_string())?;
        let child_uri = Url::from_file_path(&child_path)
            .map_err(|()| cstr!("invalid child path {}", child_path.display()))?;
        let parent_uri = Url::from_file_path(&parent_path)
            .map_err(|()| cstr!("invalid parent path {}", parent_path.display()))?;

        let state = ServerState::new();
        state.set_workspace_root(canonical_root.clone());
        for (uri, source) in [(&child_uri, CHILD_SOURCE), (&parent_uri, PARENT_SOURCE)] {
            state
                .documents
                .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
            state.update_virtual_docs(uri, source);
        }

        let (traced_corsa_path, protocol_trace_dir, shutdown_gate) =
            protocol::traced_corsa_executable(&canonical_root, corsa_path, observe_shutdown)?;
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(traced_corsa_path),
            working_dir: Some(canonical_root.clone()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        if let Err(error) = crate::runtime::block_on(bridge.spawn()) {
            // Without this the partially started child would outlive the
            // failed setup: `Self` is never constructed, so `Drop` never runs.
            let _ = crate::runtime::block_on(bridge.shutdown());
            return Err(error.to_string());
        }

        Ok(Self {
            root,
            canonical_root,
            state,
            bridge,
            child_uri,
            parent_uri,
            protocol_trace_dir,
            shutdown_gate,
            expected_readiness_generations: 1,
            shutdown_complete: false,
        })
    }

    pub(super) fn assert_prepare_ranges(&self) -> Result<(), String> {
        let event_start = PARENT_SOURCE
            .find("save-item")
            .ok_or_else(|| "parent event marker missing".to_owned())?;
        for relative in 0.."save-item".len() {
            let ctx = IdeContext::new(&self.state, &self.parent_uri, event_start + relative)
                .ok_or_else(|| cstr!("missing parent context at cursor {relative}"))?;
            let prepared = crate::runtime::block_on(canonical::prepare_strict(
                &ctx,
                Some(self.bridge.as_ref()),
            ))
            .map_err(|error| cstr!("strict prepare failed at cursor {relative}: {error}"))?;
            let canonical::Answer::Available(Some(prepared)) = prepared else {
                return Err(cstr!(
                    "strict prepare returned no canonical answer at cursor {relative}"
                )
                .into());
            };
            let selected = authored_text(PARENT_SOURCE, prepare_range(&prepared));
            if selected != "save-item" {
                return Err(
                    cstr!("strict prepare selected {selected:?} at cursor {relative}").into(),
                );
            }
        }
        Ok(())
    }

    pub(super) fn assert_direct_first_child_rename(&self) -> Result<(), String> {
        let child_start = CHILD_SOURCE
            .find("saveItem")
            .ok_or_else(|| "child event marker missing".to_owned())?;
        let child_ctx = IdeContext::new(&self.state, &self.child_uri, child_start + 2)
            .ok_or_else(|| "missing child context".to_owned())?;
        let (answer, stages) = crate::runtime::block_on(canonical::rename_strict_traced(
            &child_ctx,
            "direct-event",
            Some(self.bridge.as_ref()),
        ));
        let answer = answer.map_err(|error| cstr!("strict rename failed: {error}; {stages:#?}"))?;
        let canonical::Answer::Available(Some(edit)) = answer else {
            return Err(
                cstr!("strict rename returned no canonical edit; stages={stages:#?}").into(),
            );
        };
        if !matches!(
            stages.last(),
            Some(canonical::CanonicalRenameStage::Complete)
        ) {
            return Err(cstr!("rename did not complete; stages={stages:#?}").into());
        }
        assert_exact_event_edit(
            &edit,
            (&self.parent_uri, PARENT_SOURCE, "save-item", "direct-event"),
            (&self.child_uri, CHILD_SOURCE, "saveItem", "directEvent"),
        )
    }

    pub(super) fn assert_parent_and_child_renames(&self) -> Result<(), String> {
        let parent_start = PARENT_SOURCE
            .find("save-item")
            .ok_or_else(|| "parent event marker missing".to_owned())?;
        let parent_ctx = IdeContext::new(&self.state, &self.parent_uri, parent_start + 2)
            .ok_or_else(|| "missing parent context".to_owned())?;
        let parent_edit = strict_rename(&parent_ctx, self.bridge.as_ref(), "nextItem")?;
        assert_exact_event_edit(
            &parent_edit,
            (&self.parent_uri, PARENT_SOURCE, "save-item", "next-item"),
            (&self.child_uri, CHILD_SOURCE, "saveItem", "nextItem"),
        )?;

        let child_start = CHILD_SOURCE
            .find("saveItem")
            .ok_or_else(|| "child event marker missing".to_owned())?;
        let child_ctx = IdeContext::new(&self.state, &self.child_uri, child_start + 2)
            .ok_or_else(|| "missing child context".to_owned())?;
        let child_edit = strict_rename(&child_ctx, self.bridge.as_ref(), "final-event")?;
        assert_exact_event_edit(
            &child_edit,
            (&self.parent_uri, PARENT_SOURCE, "save-item", "final-event"),
            (&self.child_uri, CHILD_SOURCE, "saveItem", "finalEvent"),
        )
    }

    pub(super) fn shutdown(&mut self) -> Result<(), String> {
        let result =
            crate::runtime::block_on(self.bridge.shutdown()).map_err(|error| error.to_string());
        self.shutdown_complete = true;
        result?;
        let canonical_uri = Url::from_file_path(&self.canonical_root)
            .map_err(|()| "canonical root is not a file URI".to_owned())?;
        let logical_uri = Url::from_file_path(self.root.path())
            .map_err(|()| "logical root is not a file URI".to_owned())?;
        protocol::assert_graceful_lsp_lifecycle(
            &self.protocol_trace_dir,
            canonical_uri.as_str(),
            logical_uri.as_str(),
            self.expected_readiness_generations,
        )
    }

    pub(super) fn logical_root(&self) -> &Path {
        self.root.path()
    }

    pub(super) fn arm_shutdown_gate(&mut self) -> Result<protocol::ShutdownGate, String> {
        self.shutdown_gate
            .take()
            .ok_or_else(|| "editor shutdown gate was already armed".to_owned())?
            .arm()
    }

    pub(super) fn assert_root_identity(&self) -> Result<(), String> {
        let state_root = self
            .state
            .get_workspace_root()
            .ok_or_else(|| "session state lost its workspace root".to_owned())?;
        if state_root != self.canonical_root {
            return Err(cstr!(
                "state root {} != canonical root {}",
                state_root.display(),
                self.canonical_root.display()
            )
            .into());
        }
        for uri in [&self.child_uri, &self.parent_uri] {
            let path = uri
                .to_file_path()
                .map_err(|()| cstr!("non-file session URI: {uri}"))?;
            if !path.starts_with(&self.canonical_root) {
                return Err(cstr!(
                    "document {} escaped canonical root {}",
                    path.display(),
                    self.canonical_root.display()
                )
                .into());
            }
        }
        self.assert_platform_root_spelling()
    }

    fn assert_platform_root_spelling(&self) -> Result<(), String> {
        // macOS resolves both `/var` and `/tmp` through `/private`. The default
        // TMPDIR lands under `/var/folders`, but the repository's own Nix dev
        // shell points TMPDIR at `/tmp/nix-shell.*`, so accepting only the
        // `/var` spelling fails this test for anyone running it the documented
        // way on macOS.
        #[cfg(target_os = "macos")]
        if self.root.path() != self.canonical_root
            && !["/var", "/tmp"].iter().any(|logical_prefix| {
                self.root.path().starts_with(logical_prefix)
                    && self
                        .canonical_root
                        .starts_with(std::path::Path::new("/private").join(&logical_prefix[1..]))
            })
        {
            return Err(cstr!(
                "unexpected macOS temp root spellings: logical={}, canonical={}",
                self.root.path().display(),
                self.canonical_root.display()
            )
            .into());
        }
        #[cfg(target_os = "linux")]
        if self.root.path() != self.canonical_root {
            return Err(cstr!(
                "Linux temp root should already be canonical: logical={}, canonical={}",
                self.root.path().display(),
                self.canonical_root.display()
            )
            .into());
        }
        Ok(())
    }
}

impl Drop for RealCorsaRenameSession {
    fn drop(&mut self) {
        if !self.shutdown_complete {
            let _ = crate::runtime::block_on(self.bridge.shutdown());
            self.shutdown_complete = true;
        }
    }
}
