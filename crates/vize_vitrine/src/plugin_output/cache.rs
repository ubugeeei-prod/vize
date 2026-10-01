//! Bounded cache keyed by the entire preceding output and declared inputs.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "serialized plugin boundary uses standard strings"
)]

use super::PluginSpec;
use crate::CompileResult;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use vize_l0::key::{AmbientInput, CachedArtifact, KeyManifest, source_block_key};
mod disk;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheInput {
    pub name: String,
    pub value: String,
}

pub fn content_key(
    compiled: &CompileResult,
    spec: &PluginSpec<'_>,
    config: &str,
    audit: bool,
) -> Result<String, String> {
    content_key_for_build(
        compiled,
        spec,
        config,
        audit,
        env!("VIZE_PLUGIN_HOST_BUILD_ID"),
    )
}

pub fn content_key_for_build(
    compiled: &CompileResult,
    spec: &PluginSpec<'_>,
    config: &str,
    audit: bool,
    build: &str,
) -> Result<String, String> {
    let mut inputs = spec.inputs.unwrap_or_default().to_vec();
    inputs.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    let input_json = serde_json::to_string(&inputs).map_err(|e| e.to_string())?;
    let prior = serde_json::to_string(compiled).map_err(|e| e.to_string())?;
    let identity = serde_json::json!([spec.name, spec.family]).to_string();
    let toolchain = format!("{}:output@1:{build}", env!("CARGO_PKG_VERSION"));
    let features = format!(
        "legacy={};glyph={};napi={};wasm={};audit={audit};config={config}",
        cfg!(feature = "legacy"),
        cfg!(feature = "glyph"),
        cfg!(feature = "napi"),
        cfg!(feature = "wasm")
    );
    let manifest = KeyManifest::new()
        .with(AmbientInput::ToolchainVersion, &toolchain)
        .with(AmbientInput::FeatureFlags, &features)
        .with(AmbientInput::PluginIdentity, &identity)
        .with(AmbientInput::PluginVersion, spec.version)
        .with(AmbientInput::PluginCode, spec.fingerprint)
        .with(AmbientInput::PluginVisits, "output")
        .with(AmbientInput::PluginDemands, "[]")
        .with(AmbientInput::PluginInputs, &input_json);
    source_block_key("plugin-output", &[], &prior)
        .with_manifest(CachedArtifact::PluginResult, &manifest)
        .map(|key| key.to_string())
        .map_err(|e| format!("{e:?}"))
}

#[derive(Default)]
struct Cache {
    values: BTreeMap<String, (CompileResult, String, usize)>,
    order: VecDeque<String>,
    bytes: usize,
}

fn cache() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(Mutex::default)
}

pub fn get(
    key: &str,
    dir: Option<&Path>,
    compiled: &CompileResult,
    family: &str,
) -> Option<CompileResult> {
    let hit = cache().lock().ok()?.values.get(key).cloned();
    if let Some((result, response, _)) = hit {
        if let Some(dir) = dir
            && !disk::exists(dir, key)
        {
            disk::write(dir, key, &response);
        }
        return Some(result);
    }
    let dir = dir?;
    let response = disk::read(dir, key)?;
    let Ok((result, _)) = super::rewrite::apply(compiled, family, &response) else {
        disk::remove(dir, key);
        return None;
    };
    put(key.to_owned(), result.clone(), response, None);
    Some(result)
}

pub fn put(key: String, result: CompileResult, response: String, dir: Option<&Path>) {
    if let Some(dir) = dir {
        disk::write(dir, &key, &response);
    }
    let Ok(bytes) = serde_json::to_vec(&result) else {
        return;
    };
    let bytes = bytes.len().saturating_add(response.len());
    const LIMIT: usize = 4 * 1024 * 1024;
    if bytes > LIMIT {
        return;
    }
    let Ok(mut cache) = cache().lock() else {
        return;
    };
    if cache.values.contains_key(&key) {
        return;
    }
    while cache.values.len() >= 64 || cache.bytes.saturating_add(bytes) > LIMIT {
        let Some(old) = cache.order.pop_front() else {
            break;
        };
        if let Some((_, _, bytes)) = cache.values.remove(&old) {
            cache.bytes = cache.bytes.saturating_sub(bytes);
        }
    }
    cache.bytes += bytes;
    cache.order.push_back(key.clone());
    cache.values.insert(key, (result, response, bytes));
}
