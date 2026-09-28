//! Atomic persistent storage of audited edits. Invalid files are cache misses.
use super::schema::Reply;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: u32,
    key: String,
    reply: Reply,
}
static NEXT: AtomicU64 = AtomicU64::new(0);

fn path(dir: &Path, key: &str) -> PathBuf {
    // Keys have a stable structured prefix. Only its native hash is used in
    // filenames; no plugin-owned text can introduce path components.
    let safe: String = key.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    dir.join("s2-transforms-v1").join(format!("{safe}.json"))
}

pub(super) fn get(dir: &Path, key: &str) -> Option<Reply> {
    let mut file = fs::File::open(path(dir, key)).ok()?;
    if file.metadata().ok()?.len() > 1_048_576 {
        return None;
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 1_048_576 {
        return None;
    }
    let record: Record = serde_json::from_slice(&bytes).ok()?;
    (record.schema == 1 && record.key == key && record.reply.schema == 1).then_some(record.reply)
}

pub(super) fn put(dir: &Path, key: &str, reply: &Reply) {
    let Ok(bytes) = serde_json::to_vec(&Record {
        schema: 1,
        key: key.into(),
        reply: reply.clone(),
    }) else {
        return;
    };
    if bytes.len() > 1_048_576 {
        return;
    }
    let destination = path(dir, key);
    let Some(parent) = destination.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .and_then(|mut file| {
            file.write_all(&bytes)?;
            file.sync_all()
        });
    if written.is_ok() {
        let _ = fs::rename(&temporary, destination);
    }
    let _ = fs::remove_file(temporary);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disk_records_refuse_torn_wrong_key_and_oversize_data() {
        let dir = tempfile::tempdir().expect("directory");
        let reply = Reply {
            schema: 1,
            edits: Vec::new(),
        };
        put(dir.path(), "key1", &reply);
        assert_eq!(get(dir.path(), "key1"), Some(reply));
        fs::write(path(dir.path(), "key1"), b"{torn").expect("corruption");
        assert_eq!(get(dir.path(), "key1"), None);
        put(
            dir.path(),
            "key1",
            &Reply {
                schema: 1,
                edits: Vec::new(),
            },
        );
        fs::copy(path(dir.path(), "key1"), path(dir.path(), "key2")).expect("wrong-key record");
        assert_eq!(get(dir.path(), "key2"), None);
        fs::write(path(dir.path(), "key1"), vec![b' '; 1_048_577]).expect("oversize");
        assert_eq!(get(dir.path(), "key1"), None);
    }
}

#[cfg(test)]
mod migration {
    use super::super::{cache, schema::Identity};
    use super::*;

    fn current_key() -> String {
        cache::key(
            "<button class=\"legacy\">é</button>",
            "Migration.vue",
            "fixed-batch",
            &Identity {
                name: "hash-migration-transform".into(),
                version: "1".into(),
                fingerprint: "fixed-code".into(),
                cache_inputs: Some(Vec::new()),
            },
            "fixed-config",
        )
        .unwrap()
    }

    #[test]
    fn old_transform_records_miss_even_at_the_current_key_path() {
        let dir = tempfile::tempdir().unwrap();
        let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/hash-migration-v1.json"
        )))
        .unwrap();
        assert_eq!(fixture["state"], "captured-old-source");
        let old = fixture["transform"]
            .as_str()
            .expect("actual old transform key");
        let current = current_key();
        assert_eq!(old, "s0.v1:e99ab135a693b1e38e520b7918806d3b");
        let (identity, digest) = current.split_once(':').expect("complete current key");
        assert_eq!(identity, "l0.v2");
        assert_eq!(digest.len(), 32);
        let parsed_digest = u128::from_str_radix(digest, 16).expect("128-bit key digest");
        assert_eq!(digest, format!("{parsed_digest:032x}"));
        let reply = Reply {
            schema: 1,
            edits: Vec::new(),
        };
        put(dir.path(), old, &reply);
        assert_eq!(cache::get(&current, Some(dir.path())), None);
        std::fs::copy(path(dir.path(), old), path(dir.path(), &current)).unwrap();
        assert_eq!(cache::get(&current, Some(dir.path())), None);
        put(dir.path(), &current, &reply);
        assert_eq!(cache::get(&current, Some(dir.path())), Some(reply.clone()));
        assert_eq!(get(dir.path(), &current), Some(reply));
    }

    #[test]
    #[ignore = "explicit old/current producer observation; not acceptance"]
    fn observe_transform_migration_key() {
        println!(
            "HASH_CACHE_OBSERVATION {}",
            serde_json::json!({"transform":current_key()})
        );
    }
}
