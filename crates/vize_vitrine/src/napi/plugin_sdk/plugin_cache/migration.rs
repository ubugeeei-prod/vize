//! The real diagnostic store must not accept a v1 record under a v2 key.
use super::super::batch::PluginSpec;
use super::*;
use serde_json::Value;

fn current_key() -> String {
    content_key_for_build(
        "<template>é</template>\n",
        "Migration.vue",
        &PluginSpec {
            name: "hash-migration-diagnostic",
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

fn old_key() -> String {
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/hash-migration-v1.json"
    )))
    .unwrap();
    assert_eq!(fixture["state"], "captured-old-source");
    fixture["diagnostic"]
        .as_str()
        .expect("actual old producer required")
        .to_owned()
}

#[test]
fn old_diagnostic_records_are_cold_even_when_moved_to_current_path() {
    let dir = tempfile::tempdir().unwrap();
    let old = old_key();
    let current = current_key();
    assert!(old.starts_with("s0.v1:"));
    assert!(current.starts_with("l0.v2:"));
    assert_ne!(old, current);
    PluginCache::default().put(&old, Vec::new(), Some(dir.path()));
    assert_eq!(PluginCache::default().get(&current, Some(dir.path())), None);
    std::fs::copy(
        path(dir.path(), &old).unwrap(),
        path(dir.path(), &current).unwrap(),
    )
    .unwrap();
    assert_eq!(PluginCache::default().get(&current, Some(dir.path())), None);
    assert!(!path(dir.path(), &current).unwrap().exists());
    PluginCache::default().put(&current, Vec::new(), Some(dir.path()));
    assert_eq!(
        PluginCache::default().get(&current, Some(dir.path())),
        Some(Vec::new())
    );
}

#[test]
#[ignore = "explicit old/current producer observation; not acceptance"]
fn observe_diagnostic_migration_key() {
    println!(
        "HASH_CACHE_OBSERVATION {}",
        serde_json::json!({"diagnostic":current_key()})
    );
}
