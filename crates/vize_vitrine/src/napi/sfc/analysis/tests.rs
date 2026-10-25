use super::{AnalysisCache, SfcAnalyzeOptions, analyze};
use serde_json::{Value, json};
use std::sync::Mutex;

fn metadata(script: &str) -> Value {
    let source = format!("<script setup lang=\"ts\">{script}</script><template><div /></template>");
    let result = analyze(
        &source,
        &SfcAnalyzeOptions::default(),
        &Mutex::new(AnalysisCache::default()),
    );
    assert!(result.is_ok(), "{result:?}");
    result.unwrap_or(Value::Null)
}

#[test]
fn runtime_constructors_defaults_arrays_functions_and_required_are_complete() {
    let result = metadata(
        r#"defineProps({
      label: { type: String, default: "true" },
      code: { type: String, default: "007" },
      count: { type: Number, required: true, default: -2 },
      enabled: { type: Boolean, default: false },
      items: { type: Array, default: () => [1, "日本語"] },
      data: { type: Object, default: () => ({ nested: { flag: true } }) },
      callback: { type: Function, default: () => 42 },
      opaque: { type: Object, default: () => readState() },
      optional: String,
    }); defineEmits(["change", "update:modelValue"]);"#,
    );
    assert_eq!(result["emits"], json!(["change", "update:modelValue"]));
    let props = result["props"].as_array().cloned().unwrap_or_default();
    assert_eq!(props.len(), 9);
    let prop = |name: &str| {
        props
            .iter()
            .find(|prop| prop["name"] == name)
            .cloned()
            .unwrap_or(Value::Null)
    };
    assert_eq!(prop("label")["default_value"], json!("true"));
    assert_eq!(prop("code")["default_value"], json!("007"));
    assert_eq!(prop("count")["default_value"], json!(-2.0));
    assert_eq!(prop("count")["required"], json!(true));
    assert_eq!(prop("enabled")["default_value"], json!(false));
    assert_eq!(prop("items")["default_value"], json!([1.0, "日本語"]));
    assert_eq!(
        prop("data")["default_value"],
        json!({"nested":{"flag":true}})
    );
    for name in ["callback", "opaque"] {
        assert_eq!(prop(name)["unsupported_default"], json!(true));
        assert!(prop(name).get("default_value").is_none());
        assert!(prop(name)["default_source"].is_string());
    }
    assert_eq!(prop("optional")["type"], json!("string"));
    assert_eq!(prop("optional")["required"], json!(false));
    assert!(prop("optional").get("default_value").is_none());
}

#[test]
fn typed_defaults_runtime_array_and_multiple_blocks_share_script_analysis() {
    let typed = metadata(
        "withDefaults(defineProps<{ label?: string; count: number; items?: string[] }>(), { label: 'Authored', count: 3, items: () => ['a'] }); defineEmits<{ change: [value: string] }>();",
    );
    assert_eq!(typed["props"][0]["default_value"], json!("Authored"));
    assert_eq!(typed["props"][1]["required"], json!(true));
    assert_eq!(typed["props"][2]["default_value"], json!(["a"]));
    assert_eq!(typed["emits"], json!(["change"]));
    assert_eq!(
        metadata("defineProps(['label', 'count']);")["props"],
        json!([
            {"name":"label","type":"unknown","required":false},
            {"name":"count","type":"unknown","required":false}
        ])
    );
    let result = analyze(
        "<script lang=\"ts\">type Label = 'brand' | 'neutral'</script><script setup lang=\"ts\">defineProps<{ tone: Label }>()</script>",
        &SfcAnalyzeOptions::default(),
        &Mutex::new(AnalysisCache::default()),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        result.unwrap_or(Value::Null)["props"][0]["name"],
        json!("tone")
    );
}

#[test]
fn profiling_counts_cold_warm_source_edits_bypass_and_errors_without_stale_values() {
    let cache = Mutex::new(AnalysisCache::default());
    let options = SfcAnalyzeOptions {
        filename: Some("Probe.vue".into()),
        profile: Some(true),
        cache: None,
    };
    let source = "<script setup>defineProps({ label: { type: String, default: 'one' } })</script>";
    let cold = analyze(source, &options, &cache).unwrap_or(Value::Null);
    assert_eq!(cold["profile"]["cacheHit"], json!(false));
    for key in [
        "descriptorParseCalls",
        "analyzeCalls",
        "defaultParseCalls",
        "serializeCalls",
    ] {
        assert_eq!(cold["profile"][key], json!(1));
    }
    let warm = analyze(source, &options, &cache).unwrap_or(Value::Null);
    assert_eq!(warm["props"], cold["props"]);
    assert_eq!(warm["profile"]["cacheHit"], json!(true));
    for key in [
        "descriptorParseCalls",
        "analyzeCalls",
        "defaultParseCalls",
        "serializeCalls",
    ] {
        assert_eq!(warm["profile"][key], json!(0));
    }
    let edited =
        analyze(&source.replace("'one'", "'two'"), &options, &cache).unwrap_or(Value::Null);
    assert_eq!(edited["profile"]["cacheHit"], json!(false));
    assert_eq!(edited["props"][0]["default_value"], json!("two"));
    let bypass = SfcAnalyzeOptions {
        cache: Some(false),
        ..options
    };
    assert_eq!(
        analyze(source, &bypass, &cache).unwrap_or(Value::Null)["profile"]["cacheHit"],
        json!(false)
    );
    assert!(analyze("<script setup>broken", &bypass, &cache).is_err());
    assert!(
        metadata("defineProps<{ label?: string }>()")
            .get("profile")
            .is_none()
    );
}
