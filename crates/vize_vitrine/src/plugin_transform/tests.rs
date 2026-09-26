use super::*;
use serde_json::{Value, json};

fn identity(name: &str) -> Identity {
    Identity {
        name: name.into(),
        version: "1".into(),
        fingerprint: "fixture-v1".into(),
        cache_inputs: Some(Vec::new()),
    }
}

#[expect(
    clippy::panic_in_result_fn,
    reason = "fixture callback asserts the native input contract"
)]
fn design_system(batch: String) -> Result<String> {
    let batch: Value = serde_json::from_str(&batch).expect("valid batch");
    assert_eq!(batch["schema"], 1);
    assert_eq!(batch["stage"], schema::STAGE);
    let mut edits = Vec::new();
    for node in batch["nodes"].as_array().expect("nodes") {
        for attr in node["attrs"].as_array().expect("attributes") {
            if attr["name"] == "class" && attr["value"] == "legacy-btn" {
                edits.push(json!({"kind":"replace-static-attribute","node":node["id"],"name":"class","value":"ds-button"}));
            }
        }
    }
    Ok(json!({"schema":1,"edits":edits}).to_string())
}

#[test]
fn real_l2_edits_compile_with_original_source_maps_and_hoists() {
    let source = "<main><button class=\"legacy-btn\" :title=\"label\">é {{ label }}</button><span class=\"legacy-btn\">OK</span></main>";
    let plugins = [identity("design-system")];
    for hoist_static in [false, true] {
        let options = CompileOptions {
            source_map: true,
            hoist_static,
            cache: false,
            cache_dir: None,
        };
        let output = compile(source, "src/Design.vue", &plugins, options, |_, batch| {
            design_system(batch)
        })
        .expect("real transform compile");
        let map: Value = serde_json::from_str(output.result.map.as_ref().expect("map").as_str())
            .expect("map JSON");
        assert_eq!(map["sourcesContent"], json!([source]));
        assert_eq!(map["file"], "src/Design.vue");
        assert!(
            output.result.code.contains("ds-button")
                || output.result.preamble.contains("ds-button")
        );
        assert!(!output.result.code.contains("legacy-btn"));
        assert!(!output.result.preamble.contains("legacy-btn"));
        let without_map = compile(
            source,
            "src/Design.vue",
            &plugins,
            CompileOptions {
                source_map: false,
                ..options
            },
            |_, batch| design_system(batch),
        )
        .expect("no map compile");
        assert_eq!(output.result.code, without_map.result.code);
        assert_eq!(output.result.preamble, without_map.result.preamble);
        insta::assert_snapshot!(
            if hoist_static {
                "design_system_hoisted"
            } else {
                "design_system_inline"
            },
            format!(
                "{}\n{}\n{}",
                output.result.preamble,
                output.result.code,
                serde_json::to_string_pretty(&map).expect("JSON")
            )
        );
        let [cost] = output.costs.as_slice() else {
            panic!("one plugin");
        };
        assert_eq!(cost.name, "design-system");
        assert_eq!(cost.nodes, 3);
        assert_eq!(cost.edits, 2);
        assert!(!cost.cached);
        assert!(cost.elapsed_ns >= cost.js_ns);
        assert!(cost.content_key.len() >= 32);
    }
}

#[test]
fn semantic_values_are_decoded_once_and_keep_authored_spans() {
    let source = "<button title=\"old &amp; text\">OK</button>";
    let output = compile(source, "Escaped.vue", &[identity("escape")], CompileOptions { source_map: true, ..Default::default() }, |_, batch| {
        let batch: Value = serde_json::from_str(&batch).expect("batch");
        assert_eq!(batch["nodes"][0]["attrs"][0]["value"], "old & text");
        Ok(json!({"schema":1,"edits":[{"kind":"replace-static-attribute","node":0,"name":"title","value":"new & \"é\" <text>"}]}).to_string())
    }).expect("semantic value");
    insta::assert_snapshot!("semantic_attribute_value", output.result.code);
}

#[test]
fn page_ids_include_bindings_and_conditional_and_loop_regions() {
    let source = "<main @click=\"run\"><button v-if=\"ok\" class=\"legacy-btn\"></button><button v-else class=\"legacy-btn\"></button><i v-for=\"item in items\" class=\"legacy-btn\"></i></main>";
    let output = compile(
        source,
        "Ids.vue",
        &[identity("page-ids")],
        CompileOptions::default(),
        |_, batch| design_system(batch),
    )
    .expect("all region IDs");
    let [cost] = output.costs.as_slice() else {
        panic!("one plugin");
    };
    assert_eq!(cost.edits, 3);
}

#[test]
fn bad_schema_unknown_fields_structural_targets_and_nondeterminism_refuse() {
    for response in [
        "{}",
        r#"{"schema":2,"edits":[]}"#,
        r#"{"schema":1,"edits":[],"extra":true}"#,
        r#"{"schema":1,"edits":[{"kind":"replace-static-attribute","node":0,"name":"class"}]}"#,
        r#"{"schema":1,"edits":[{"kind":"replace-static-attribute","node":0,"name":"key","value":"x"}]}"#,
        r#"{"schema":1,"edits":[{"kind":"replace-static-attribute","node":99,"name":"class","value":"x"}]}"#,
        r#"{"schema":1,"edits":[{"kind":"replace-static-attribute","node":0,"name":"id","value":"x"}]}"#,
    ] {
        assert!(
            compile(
                "<button class=\"legacy-btn\"></button>",
                "Bad.vue",
                &[identity("bad")],
                CompileOptions::default(),
                |_, _| Ok(response.into())
            )
            .is_err(),
            "{response}"
        );
    }
    for source in [
        "<svg class=\"legacy-btn\"></svg>",
        "<Widget class=\"legacy-btn\"/>",
        "<template class=\"legacy-btn\"></template>",
    ] {
        assert!(compile(source, "Bad.vue", &[identity("bad-kind")], CompileOptions::default(), |_, _|
            Ok(json!({"schema":1,"edits":[{"kind":"replace-static-attribute","node":0,"name":"class","value":"x"}]}).to_string())).is_err());
    }
    let mut count = 0;
    let error = compile("<button class=\"legacy-btn\"></button>", "NonDet.vue", &[identity("non-det")], CompileOptions::default(), |_, _| {
        count += 1;
        Ok(json!({"schema":1,"edits":[{"kind":"replace-static-attribute","node":0,"name":"class","value":count.to_string()}]}).to_string())
    }).err().expect("nondeterministic refusal");
    assert!(error.contains("nondeterministic"));
}

#[test]
fn audited_cache_hits_and_declared_identity_changes_are_misses() {
    let source = "<button class=\"legacy-btn\"></button>";
    let mut plugin = identity("cache-test-unique");
    let options = CompileOptions {
        cache: true,
        ..Default::default()
    };
    let mut calls = 0;
    let first = compile(
        source,
        "Cache.vue",
        &[plugin.clone()],
        options,
        |_, batch| {
            calls += 1;
            design_system(batch)
        },
    )
    .expect("first");
    let hit = compile(
        source,
        "Cache.vue",
        &[plugin.clone()],
        options,
        |_, batch| {
            calls += 1;
            design_system(batch)
        },
    )
    .expect("hit");
    assert_eq!(calls, 2);
    assert!(hit.costs.first().expect("cost").cached);
    assert_eq!(first.result.code, hit.result.code);
    let previous = hit.costs.first().expect("cost").content_key.clone();
    plugin.version = "2".into();
    let miss = compile(
        source,
        "Cache.vue",
        &[plugin.clone()],
        options,
        |_, batch| {
            calls += 1;
            design_system(batch)
        },
    )
    .expect("version miss");
    assert_eq!(calls, 4);
    assert_ne!(previous, miss.costs.first().expect("cost").content_key);
    plugin.cache_inputs = Some(vec![("theme".into(), "dark".into())]);
    let miss = compile(
        source,
        "Cache.vue",
        &[plugin.clone()],
        options,
        |_, batch| {
            calls += 1;
            design_system(batch)
        },
    )
    .expect("input miss");
    assert!(!miss.costs.first().expect("cost").cached);
    plugin.cache_inputs = None;
    assert!(
        compile(source, "Cache.vue", &[plugin], options, |_, batch| {
            design_system(batch)
        })
        .is_err()
    );
}

#[test]
fn invalid_late_edit_does_not_mutate_the_native_artifact() {
    let allocator = vize_l0::Allocator::new();
    let (tree, errors) = vize_l1::parse(&allocator, "<button class=\"legacy-btn\"></button>");
    let mut lowered = vize_l1_to_l2::lower(&allocator, &tree, &errors);
    let reply = edits::decode(r#"{"schema":1,"edits":[{"kind":"replace-static-attribute","node":0,"name":"class","value":"changed"},{"kind":"replace-static-attribute","node":0,"name":"missing","value":"x"}]}"#).expect("typed reply");
    let nodes = walk::nodes(&mut lowered.root);
    assert!(edits::validate(&reply, &nodes).is_err());
    assert_eq!(
        walk::nodes(&mut lowered.root)
            .first()
            .expect("node")
            .attrs
            .first()
            .expect("attr")
            .value
            .as_deref(),
        Some("legacy-btn")
    );
}
