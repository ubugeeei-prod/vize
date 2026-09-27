use super::*;

fn compiled() -> CompileResult {
    CompileResult {
        code: "const answer=42;".to_owned(),
        preamble: String::new(),
        ast: serde_json::json!({}),
        map: None,
        helpers: vec![],
        templates: None,
    }
}

#[test]
fn fresh_disk_entries_repeat_output_validation() {
    let directory = tempfile::tempdir().unwrap();
    let compiled = compiled();
    let spec = PluginSpec {
        name: "fresh-disk",
        version: "1",
        fingerprint: "code",
        family: "output",
        inputs: Some(&[]),
    };
    let key = content_key(&compiled, &spec, "config", true).unwrap();
    disk::write(
        directory.path(),
        &key,
        r#"[{"placement":"replace","comment":"forged"}]"#,
    );
    assert!(get(&key, Some(directory.path()), &compiled, "output").is_none());
    assert!(!disk::exists(directory.path(), &key));
    disk::write(
        directory.path(),
        &key,
        r#"[{"placement":"prepend","comment":"license"}]"#,
    );
    let hit = get(&key, Some(directory.path()), &compiled, "output").unwrap();
    assert_eq!(hit.code, "/* license */\nconst answer=42;");
}

#[test]
fn corrupt_mismatched_and_oversized_entries_are_misses() {
    let directory = tempfile::tempdir().unwrap();
    let key = "key:abc123";
    let path = directory.path().join("plugin-output-v1-abc123.json");
    for content in [
        "{torn".to_owned(),
        serde_json::json!({"schema":2,"key":key,"response":"[]"}).to_string(),
        serde_json::json!({"schema":1,"key":"different","response":"[]"}).to_string(),
        "x".repeat(3 * 1024 * 1024),
    ] {
        std::fs::write(&path, content).unwrap();
        assert!(disk::read(directory.path(), key).is_none());
        assert!(!path.exists());
    }
}

#[test]
fn memory_hit_materializes_a_new_disk_directory_without_js() {
    let directory = tempfile::tempdir().unwrap();
    let compiled = compiled();
    let spec = PluginSpec {
        name: "disk-materialization",
        version: "1",
        fingerprint: "code",
        family: "output",
        inputs: Some(&[]),
    };
    let key = content_key(&compiled, &spec, "config", true).unwrap();
    put(key.clone(), compiled.clone(), "[]".to_owned(), None);
    assert!(get(&key, Some(directory.path()), &compiled, "output").is_some());
    assert_eq!(disk::read(directory.path(), &key), Some("[]".to_owned()));
}

fn migration_key() -> String {
    content_key_for_build(
        &compiled(),
        &PluginSpec {
            name: "hash-migration-output",
            version: "1",
            fingerprint: "fixed-code",
            family: "output",
            inputs: Some(&[]),
        },
        "fixed-config",
        true,
        "hash-migration-control",
    )
    .unwrap()
}

#[test]
fn old_output_records_miss_and_relocated_records_do_not_rewrite_current_output() {
    let directory = tempfile::tempdir().unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/hash-migration-v1.json"
    )))
    .unwrap();
    assert_eq!(fixture["state"], "captured-old-source");
    let old = fixture["output"].as_str().expect("actual old output key");
    let current = migration_key();
    assert!(old.starts_with("s0.v1:"));
    assert!(current.starts_with("l0.v2:"));
    let response = r#"[{"placement":"prepend","comment":"migration license"}]"#;
    disk::write(directory.path(), old, response);
    let prior = compiled();
    assert!(get(&current, Some(directory.path()), &prior, "output").is_none());
    let filename = |key: &str| {
        directory.path().join(format!(
            "plugin-output-v1-{}.json",
            key.rsplit_once(':').unwrap().1
        ))
    };
    std::fs::copy(filename(old), filename(&current)).unwrap();
    assert!(get(&current, Some(directory.path()), &prior, "output").is_none());
    disk::write(directory.path(), &current, response);
    let fresh = get(&current, Some(directory.path()), &prior, "output").unwrap();
    let direct = super::super::rewrite::apply(&prior, "output", response)
        .unwrap()
        .0;
    assert_eq!(
        serde_json::to_vec(&fresh).unwrap(),
        serde_json::to_vec(&direct).unwrap()
    );
    assert_eq!(fresh.code, "/* migration license */\nconst answer=42;");
    // A second fresh disk read repeats the actual validator without cached ASTs.
    let (repeated, _) = super::super::rewrite::apply(
        &prior,
        "output",
        &disk::read(directory.path(), &current).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_vec(&repeated).unwrap(),
        serde_json::to_vec(&direct).unwrap()
    );
}

#[test]
#[ignore = "explicit old/current producer observation; not acceptance"]
fn observe_output_migration_key() {
    println!(
        "HASH_CACHE_OBSERVATION {}",
        serde_json::json!({"output":migration_key()})
    );
}
