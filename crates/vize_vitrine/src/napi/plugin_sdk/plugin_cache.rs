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
use super::error::HostError;

const CACHE_SCHEMA: u32 = 1;

/// An explicitly declared plugin-owned input, such as a rule option or env value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginCacheInput<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

/// A cache opt-in is invalid until the author declares even an empty input
/// list; equal names cannot hide different values by their order.
pub fn validate_cache_inputs(
    plugin: &str,
    declared: bool,
    inputs: &[PluginCacheInput<'_>],
) -> Result<(), HostError> {
    if !declared {
        return Err(HostError::InvalidCacheInputs {
            plugin: plugin.to_owned(),
            detail: "declare cacheInputs, even when it is empty".to_owned(),
        });
    }
    let mut names: Vec<&str> = inputs.iter().map(|input| input.name).collect();
    names.sort_unstable();
    if let Some(name) = names.iter().find(|name| name.is_empty()) {
        return Err(HostError::InvalidCacheInputs {
            plugin: plugin.to_owned(),
            detail: format!("input name `{name}` is empty"),
        });
    }
    if let Some(pair) = names.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(HostError::InvalidCacheInputs {
            plugin: plugin.to_owned(),
            detail: format!("input name `{}` is duplicated", pair[0]),
        });
    }
    Ok(())
}

/// The S0 content key with every non-content batch input declared and folded.
#[must_use]
pub fn content_key(
    source: &str,
    filename: &str,
    spec: &PluginSpec<'_>,
    inputs: &[PluginCacheInput<'_>],
) -> String {
    content_key_for_build(
        source,
        filename,
        spec,
        inputs,
        env!("VIZE_PLUGIN_HOST_BUILD_ID"),
    )
}

/// The build identity is explicit here so a test can pin revision invalidation.
#[must_use]
pub fn content_key_for_build(
    source: &str,
    filename: &str,
    spec: &PluginSpec<'_>,
    inputs: &[PluginCacheInput<'_>],
    build_id: &str,
) -> String {
    let identity = serde_json::json!([spec.name, filename]).to_string();
    let visits = serde_json::to_string(&spec.visit).expect("plugin visit names serialize");
    let demands = serde_json::to_string(spec.demands).expect("plugin demand names serialize");
    let mut inputs = inputs.to_vec();
    inputs.sort_unstable_by(|left, right| left.name.cmp(right.name));
    let inputs = serde_json::to_string(
        &inputs
            .iter()
            .map(|input| (input.name, input.value))
            .collect::<Vec<_>>(),
    )
    .expect("plugin input names and values serialize");
    let toolchain = format!(
        "{}:{BATCH_SCHEMA}:{CACHE_SCHEMA}:{build_id}",
        env!("CARGO_PKG_VERSION"),
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
        .with(AmbientInput::PluginDemands, &demands)
        .with(AmbientInput::PluginInputs, &inputs);
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
