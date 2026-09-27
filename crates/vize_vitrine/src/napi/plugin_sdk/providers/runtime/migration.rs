//! Exercise the actual provider dependency assembler and the diagnostic store.
use super::super::super::plugin_cache::PluginCache;
use super::*;
use serde_json::Value;

#[test]
fn canonical_provider_stamps_invalidate_rules_without_changing_fact_payloads() {
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/hash-migration-v1.json"
    )))
    .unwrap();
    assert_eq!(fixture["state"], "captured-old-source");
    let groups = vec!["tokens/colors".to_owned()];
    let json = r#"{"tokens/colors":[[0,"red"]]}"#;
    let output = ProviderOutput::audit("tokens", &groups, json, json).unwrap();
    let old_provider = fixture["provider"].as_str().expect("old provider capture");
    let old_result = fixture["providerResult"]
        .as_str()
        .expect("old result capture");
    let provider = PluginSpec {
        name: "hash-migration-provider",
        version: "1",
        fingerprint: "fixed-code",
        visit: None,
        demands: &[],
    };
    let current_provider = super::super::super::plugin_cache::content_key_for_build(
        "<template>é</template>\n",
        "Migration.vue",
        &provider,
        &[],
        "hash-migration-control",
    )
    .unwrap();
    let assemble = |provider_key: &str, result_key: &str| {
        let mut host = ProviderHost::new(Vec::new()).unwrap();
        host.results.insert(
            0,
            Provided {
                values: output.values.clone(),
                stamp: serde_json::to_string(&(provider_key, result_key)).unwrap(),
            },
        );
        host.declared(&groups)
    };
    let (old_facts, old_inputs) = assemble(old_provider, old_result);
    let (current_facts, current_inputs) = assemble(&current_provider, &output.result_key);
    assert_eq!(old_facts, current_facts);
    assert_eq!(current_facts, output.values);
    assert_ne!(old_inputs, current_inputs);
    let consumer = PluginSpec {
        name: "hash-migration-consumer",
        version: "1",
        fingerprint: "fixed-code",
        visit: None,
        demands: &groups,
    };
    let key = |inputs: &[(String, String)]| {
        let inputs: Vec<_> = inputs
            .iter()
            .map(|(name, value)| PluginCacheInput { name, value })
            .collect();
        content_key(
            "<template>é</template>\n",
            "Migration.vue",
            &consumer,
            &inputs,
        )
        .unwrap()
    };
    let old_stamp_key = key(&old_inputs);
    let current_stamp_key = key(&current_inputs);
    assert_ne!(old_stamp_key, current_stamp_key);
    let dir = tempfile::tempdir().unwrap();
    PluginCache::default().put(&old_stamp_key, Vec::new(), Some(dir.path()));
    assert!(
        PluginCache::default()
            .get(&current_stamp_key, Some(dir.path()))
            .is_none()
    );
    PluginCache::default().put(&current_stamp_key, Vec::new(), Some(dir.path()));
    assert_eq!(
        PluginCache::default().get(&current_stamp_key, Some(dir.path())),
        Some(Vec::new())
    );
}
