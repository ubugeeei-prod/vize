//! Bounded process reuse and atomic, validated provider output persistence.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use super::ProviderOutput;

const CACHE_SCHEMA: u32 = 1;
const MAX_ENTRIES: usize = 32;
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_FILE_BYTES: usize = 4 * 1024 * 1024 + 4096;

#[derive(Default)]
pub struct ProviderCache {
    entries: BTreeMap<String, String>,
    order: VecDeque<String>,
    bytes: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CacheFile {
    schema: u32,
    key: String,
    values: Map<String, Value>,
    result_key: String,
}

impl ProviderCache {
    pub fn get(
        &mut self,
        key: &str,
        plugin: &str,
        provides: &[String],
        dir: Option<&Path>,
    ) -> Option<ProviderOutput> {
        let memory = self.entries.get(key).cloned();
        let json = memory.clone().or_else(|| {
            let at = path(dir?, key)?;
            let mut bytes = Vec::new();
            File::open(at)
                .ok()?
                .take((MAX_FILE_BYTES + 1) as u64)
                .read_to_end(&mut bytes)
                .ok()?;
            (bytes.len() <= MAX_FILE_BYTES)
                .then(|| String::from_utf8(bytes).ok())
                .flatten()
        })?;
        let valid = || {
            let file: CacheFile = serde_json::from_str(&json).ok()?;
            if file.schema != CACHE_SCHEMA || file.key != key {
                return None;
            }
            let encoded = serde_json::to_string(&file.values).ok()?;
            let output = ProviderOutput::audit(plugin, provides, &encoded, &encoded).ok()?;
            (output.result_key == file.result_key).then_some(output)
        };
        let Some(output) = valid() else {
            self.remove(key);
            if let Some(at) = dir.and_then(|dir| path(dir, key)) {
                let _ = fs::remove_file(at);
            }
            return None;
        };
        if memory.is_none() {
            self.insert(key, json.clone());
        }
        if let Some(dir) = dir
            && !path(dir, key).is_some_and(|at| at.is_file())
        {
            write(dir, key, &json);
        }
        Some(output)
    }

    pub fn put(&mut self, key: &str, output: &ProviderOutput, dir: Option<&Path>) {
        let file = CacheFile {
            schema: CACHE_SCHEMA,
            key: key.to_owned(),
            values: output.values.clone(),
            result_key: output.result_key.clone(),
        };
        let Ok(json) = serde_json::to_string(&file) else {
            return;
        };
        if json.len() > MAX_FILE_BYTES {
            return;
        }
        if let Some(dir) = dir {
            write(dir, key, &json);
        }
        self.insert(key, json);
    }

    fn insert(&mut self, key: &str, json: String) {
        if json.len() > MAX_BYTES {
            return;
        }
        self.remove(key);
        while self.entries.len() >= MAX_ENTRIES || self.bytes + json.len() > MAX_BYTES {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(old) = self.entries.remove(&oldest) {
                self.bytes = self.bytes.saturating_sub(old.len());
            }
        }
        self.bytes += json.len();
        self.order.push_back(key.to_owned());
        self.entries.insert(key.to_owned(), json);
    }

    fn remove(&mut self, key: &str) {
        if let Some(old) = self.entries.remove(key) {
            self.bytes = self.bytes.saturating_sub(old.len());
        }
        self.order.retain(|stored| stored != key);
    }
}

fn path(dir: &Path, key: &str) -> Option<PathBuf> {
    let (_, digest) = key.rsplit_once(':')?;
    if digest.is_empty() || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(dir.join(format!("plugin-facts-v{CACHE_SCHEMA}-{digest}.json")))
}

fn write(dir: &Path, key: &str, json: &str) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let Some(at) = path(dir, key) else {
        return;
    };
    if fs::create_dir_all(dir).is_err() {
        return;
    }
    let temporary = at.with_extension(format!(
        "{}.{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
    else {
        return;
    };
    if file.write_all(json.as_bytes()).is_ok() && file.sync_all().is_ok() {
        drop(file);
        if fs::rename(&temporary, at).is_ok() {
            return;
        }
    }
    let _ = fs::remove_file(temporary);
}

pub fn cache() -> &'static Mutex<ProviderCache> {
    static CACHE: OnceLock<Mutex<ProviderCache>> = OnceLock::new();
    CACHE.get_or_init(Mutex::default)
}

#[cfg(test)]
mod tests {
    //! Real disk round trips, corrupt entries and bounded process storage.
    use super::super::ProviderOutput;
    use super::{MAX_BYTES, MAX_ENTRIES, ProviderCache, path};

    fn output() -> (Vec<String>, ProviderOutput) {
        let groups = vec!["tokens/colors".to_owned()];
        let json = r#"{"tokens/colors":[[0,"red"]]}"#;
        let output = ProviderOutput::audit("tokens", &groups, json, json).unwrap();
        (groups, output)
    }

    #[test]
    fn persisted_outputs_round_trip_through_a_fresh_cache_and_validate_declarations() {
        let dir = tempfile::tempdir().unwrap();
        let (groups, output) = output();
        let key = "plugin-result:012345abcdef";
        ProviderCache::default().put(key, &output, Some(dir.path()));
        let mut fresh = ProviderCache::default();
        let hit = fresh.get(key, "tokens", &groups, Some(dir.path())).unwrap();
        assert_eq!(
            (hit.values, hit.result_key),
            (output.values, output.result_key)
        );
        assert!(
            fresh
                .get(
                    key,
                    "tokens",
                    &["tokens/other".to_owned()],
                    Some(dir.path())
                )
                .is_none()
        );
        assert!(!path(dir.path(), key).unwrap().exists());
    }

    #[test]
    fn malformed_stale_mismatched_or_tampered_entries_are_cold() {
        let dir = tempfile::tempdir().unwrap();
        let (groups, _) = output();
        let key = "plugin-result:012345abcdef";
        for json in [
            "{broken",
            r#"{"schema":2,"key":"plugin-result:012345abcdef","values":{"tokens/colors":[]},"result_key":"stale"}"#,
            r#"{"schema":1,"key":"wrong","values":{"tokens/colors":[]},"result_key":"stale"}"#,
            r#"{"schema":1,"key":"plugin-result:012345abcdef","values":{"tokens/colors":[[0,"blue"]]},"result_key":"tampered"}"#,
        ] {
            let at = path(dir.path(), key).unwrap();
            std::fs::write(&at, json).unwrap();
            assert!(
                ProviderCache::default()
                    .get(key, "tokens", &groups, Some(dir.path()))
                    .is_none()
            );
            assert!(!at.exists());
        }
    }

    #[test]
    fn process_storage_is_bounded_and_evicts_the_oldest_output() {
        let (groups, output) = output();
        let mut cache = ProviderCache::default();
        for index in 0..MAX_ENTRIES + 1 {
            cache.put(&format!("plugin-result:{index:x}"), &output, None);
        }
        assert_eq!(cache.entries.len(), MAX_ENTRIES);
        assert!(cache.bytes <= MAX_BYTES);
        assert!(
            cache
                .get("plugin-result:0", "tokens", &groups, None)
                .is_none()
        );
        assert!(
            cache
                .get("plugin-result:20", "tokens", &groups, None)
                .is_some()
        );
        for index in 100..110 {
            cache.insert(&format!("large:{index:x}"), "x".repeat(4 * 1024 * 1024));
        }
        assert_eq!(cache.entries.len(), 4);
        assert_eq!(cache.bytes, MAX_BYTES);
    }

    #[test]
    fn disk_paths_never_include_plugin_paths_or_arbitrary_key_tails() {
        let dir = tempfile::tempdir().unwrap();
        assert!(path(dir.path(), "plugin-result:../../outside").is_none());
        assert!(path(dir.path(), "plugin-result:").is_none());
        assert_eq!(
            path(dir.path(), "user/../../name:0123abcdef")
                .unwrap()
                .parent(),
            Some(dir.path())
        );
    }

    fn migration_key() -> String {
        use super::super::super::batch::PluginSpec;
        super::super::super::plugin_cache::content_key_for_build(
            "<template>é</template>\n",
            "Migration.vue",
            &PluginSpec {
                name: "hash-migration-provider",
                version: "1",
                fingerprint: "fixed-code",
                visit: None,
                demands: &[],
            },
            &[],
            "hash-migration-control",
        )
        .unwrap()
    }

    #[test]
    fn old_provider_keys_and_result_fingerprints_cannot_be_reused() {
        let dir = tempfile::tempdir().unwrap();
        let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/hash-migration-v1.json"
        )))
        .unwrap();
        assert_eq!(fixture["state"], "captured-old-source");
        let old = fixture["provider"]
            .as_str()
            .expect("actual old provider key");
        let old_result = fixture["providerResult"]
            .as_str()
            .expect("actual old result key");
        let current = migration_key();
        let (groups, output) = output();
        assert!(old.starts_with("s0.v1:"));
        assert!(current.starts_with("l0.v2:"));
        assert_ne!(output.result_key, old_result);
        let old_output = ProviderOutput {
            values: output.values.clone(),
            result_key: old_result.to_owned(),
        };
        ProviderCache::default().put(old, &old_output, Some(dir.path()));
        assert!(
            ProviderCache::default()
                .get(&current, "tokens", &groups, Some(dir.path()))
                .is_none()
        );
        std::fs::copy(
            path(dir.path(), old).unwrap(),
            path(dir.path(), &current).unwrap(),
        )
        .unwrap();
        assert!(
            ProviderCache::default()
                .get(&current, "tokens", &groups, Some(dir.path()))
                .is_none()
        );
        // A transplanted matching record key must still reject the old result digest.
        ProviderCache::default().put(&current, &old_output, Some(dir.path()));
        assert!(
            ProviderCache::default()
                .get(&current, "tokens", &groups, Some(dir.path()))
                .is_none()
        );
        ProviderCache::default().put(&current, &output, Some(dir.path()));
        let hit = ProviderCache::default()
            .get(&current, "tokens", &groups, Some(dir.path()))
            .unwrap();
        assert_eq!(hit.values, output.values);
        assert_eq!(hit.result_key, output.result_key);
    }

    #[test]
    #[ignore = "explicit old/current producer observation; not acceptance"]
    fn observe_provider_migration_keys() {
        println!(
            "HASH_CACHE_OBSERVATION {}",
            serde_json::json!({"provider":migration_key(),"providerResult":output().1.result_key,"providerValues":output().1.values})
        );
    }
}
