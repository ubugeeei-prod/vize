//! Bounded in-process cache of audited, validated edits, never generated ASTs.
use super::{
    Result,
    schema::{Identity, Reply, SCHEMA, STAGE},
};
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use vize_l0::key::{AmbientInput, CachedArtifact, KeyManifest, source_block_key};

#[derive(Default)]
struct Cache {
    entries: BTreeMap<String, Reply>,
    order: VecDeque<String>,
    bytes: usize,
}
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

pub(super) fn key(
    source: &str,
    filename: &str,
    batch: &str,
    plugin: &Identity,
    options: &str,
) -> Result<String> {
    let mut inputs = plugin.cache_inputs.clone().unwrap_or_default();
    inputs.sort();
    if inputs.iter().any(|(name, _)| name.is_empty())
        || inputs
            .windows(2)
            .any(|p| matches!(p, [(left, _), (right, _)] if left == right))
    {
        return Err("transform cacheInputs names must be nonempty and unique".into());
    }
    let inputs = serde_json::to_string(&inputs).map_err(|e| e.to_string())?;
    let identity =
        serde_json::to_string(&(plugin.name.as_str(), filename)).map_err(|e| e.to_string())?;
    let build = format!(
        "{}:{SCHEMA}:{STAGE}:{}",
        env!("CARGO_PKG_VERSION"),
        env!("VIZE_PLUGIN_HOST_BUILD_ID")
    );
    let manifest = KeyManifest::new()
        .with(AmbientInput::ToolchainVersion, &build)
        .with(AmbientInput::FeatureFlags, options)
        .with(AmbientInput::PluginIdentity, &identity)
        .with(AmbientInput::PluginVersion, &plugin.version)
        .with(AmbientInput::PluginCode, &plugin.fingerprint)
        .with(AmbientInput::PluginVisits, batch)
        .with(AmbientInput::PluginDemands, "[]")
        .with(AmbientInput::PluginInputs, &inputs);
    source_block_key("plugin-transform", &[], source)
        .with_manifest(CachedArtifact::PluginResult, &manifest)
        .map(|key| key.to_string())
        .map_err(|e| format!("unkeyable transform: {e:?}"))
}

pub(super) fn get(key: &str, dir: Option<&Path>) -> Option<Reply> {
    let found = CACHE
        .get_or_init(|| Mutex::new(Cache::default()))
        .lock()
        .ok()?
        .entries
        .get(key)
        .cloned();
    if let Some(reply) = found {
        if let Some(dir) = dir {
            super::disk::put(dir, key, &reply);
        }
        return Some(reply);
    }
    let reply = super::disk::get(dir?, key)?;
    Some(reply)
}

pub(super) fn put(key: String, reply: Reply, dir: Option<&Path>) {
    if let Some(dir) = dir {
        super::disk::put(dir, &key, &reply);
    }
    let Ok(mut cache) = CACHE.get_or_init(|| Mutex::new(Cache::default())).lock() else {
        return;
    };
    if cache.entries.contains_key(&key) {
        return;
    }
    let size = size(&reply);
    if size > 1_048_576 {
        return;
    }
    while cache.entries.len() >= 64 || cache.bytes + size > 1_048_576 {
        let Some(old) = cache.order.pop_front() else {
            break;
        };
        if let Some(reply) = cache.entries.remove(&old) {
            cache.bytes -= self::size(&reply);
        }
    }
    cache.bytes += size;
    cache.order.push_back(key.clone());
    cache.entries.insert(key, reply);
}

fn size(reply: &Reply) -> usize {
    serde_json::to_vec(reply).map_or(1_048_577, |bytes| bytes.len())
}
