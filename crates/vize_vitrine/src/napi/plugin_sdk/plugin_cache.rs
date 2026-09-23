//! P5-13: one manifest-checked content key for in-process and disk reuse.
//!
//! The full SFC is the conservative S0 key. Cached diagnostics contain
//! file-absolute spans, so a shift above the template must invalidate them.
//! The plugin's other batch inputs are the exactly declared P5-1b manifest.

#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use vize_davinci::key::{AmbientInput, CachedArtifact, KeyManifest, source_block_key};

use super::batch::{BATCH_SCHEMA, PluginDiagnostic, PluginSpec};

const CACHE_SCHEMA: u32 = 1;

/// The S0 content key with every non-content batch input declared and folded.
#[must_use]
pub fn content_key(source: &str, filename: &str, spec: &PluginSpec<'_>) -> String {
    let identity = serde_json::json!([spec.name, filename]).to_string();
    let visits = serde_json::to_string(&spec.visit).expect("plugin visit names serialize");
    let demands = serde_json::to_string(spec.demands).expect("plugin demand names serialize");
    let toolchain = format!(
        "{}:{BATCH_SCHEMA}:{CACHE_SCHEMA}",
        env!("CARGO_PKG_VERSION")
    );
    let features = format!(
        "legacy={};glyph={}",
        cfg!(feature = "legacy"),
        cfg!(feature = "glyph")
    );
    let manifest = KeyManifest::new()
        .with(AmbientInput::ToolchainVersion, &toolchain)
        .with(AmbientInput::FeatureFlags, &features)
        .with(AmbientInput::PluginIdentity, &identity)
        .with(AmbientInput::PluginVersion, spec.version)
        .with(AmbientInput::PluginCode, spec.fingerprint)
        .with(AmbientInput::PluginVisits, &visits)
        .with(AmbientInput::PluginDemands, &demands);
    source_block_key("plugin-document", &[], source)
        .with_manifest(CachedArtifact::PluginResult, &manifest)
        .expect("the plugin result manifest declares every input")
        .to_string()
}

#[derive(Default)]
pub struct PluginCache {
    entries: HashMap<String, Vec<PluginDiagnostic>>,
}

#[derive(Serialize, Deserialize)]
struct CacheFile {
    schema: u32,
    key: String,
    diagnostics: Vec<PluginDiagnostic>,
}

impl PluginCache {
    /// Read a result from memory or an optional cross-process disk store.
    /// A missing, torn or stale disk entry is a miss, never a diagnostic.
    pub fn get(&mut self, key: &str, dir: Option<&Path>) -> Option<Vec<PluginDiagnostic>> {
        if let Some(found) = self.entries.get(key) {
            let found = found.clone();
            if let Some(dir) = dir {
                if !path(dir, key).is_file() {
                    self.write(key, &found, dir);
                }
            }
            return Some(found);
        }
        let dir = dir?;
        let at = path(dir, key);
        let bytes = fs::read(&at).ok()?;
        let Ok(file) = serde_json::from_slice::<CacheFile>(&bytes) else {
            let _ = fs::remove_file(at);
            return None;
        };
        if file.schema != CACHE_SCHEMA || file.key != key {
            let _ = fs::remove_file(at);
            return None;
        }
        self.entries
            .insert(key.to_owned(), file.diagnostics.clone());
        Some(file.diagnostics)
    }

    /// Store the exact mapped diagnostics under the same key on both tiers.
    pub fn put(&mut self, key: &str, diagnostics: Vec<PluginDiagnostic>, dir: Option<&Path>) {
        if let Some(dir) = dir {
            self.write(key, &diagnostics, dir);
        }
        self.entries.insert(key.to_owned(), diagnostics);
    }

    fn write(&self, key: &str, diagnostics: &[PluginDiagnostic], dir: &Path) {
        if fs::create_dir_all(dir).is_err() {
            return;
        }
        let file = CacheFile {
            schema: CACHE_SCHEMA,
            key: key.to_owned(),
            diagnostics: diagnostics.to_vec(),
        };
        let Ok(bytes) = serde_json::to_vec(&file) else {
            return;
        };
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let at = path(dir, key);
        let temporary = at.with_extension(format!(
            "{}.{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let Ok(mut writer) = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        else {
            return;
        };
        if writer.write_all(&bytes).is_ok() && writer.sync_all().is_ok() {
            drop(writer);
            if fs::rename(&temporary, &at).is_ok() {
                return;
            }
        }
        let _ = fs::remove_file(temporary);
    }
}

fn path(dir: &Path, key: &str) -> PathBuf {
    // `key` comes from ArtifactKey's own display implementation. Only its
    // digest enters the filename, never a plugin- or user-supplied path.
    let digest = key
        .rsplit_once(':')
        .expect("an artifact key has a digest")
        .1;
    dir.join(format!("plugin-result-v{CACHE_SCHEMA}-{digest}.json"))
}

pub fn cache() -> &'static Mutex<PluginCache> {
    static CACHE: OnceLock<Mutex<PluginCache>> = OnceLock::new();
    CACHE.get_or_init(Mutex::default)
}
