//! Whole reported input and warning-positive execution/provenance controls.

use super::parse_script_setup;
use crate::reactivity::ReactivityLossKind;

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint-regressions/",
    "computed-local-reactivity-8756/ComputedLocal.vue.txt"
));
const CONTROLS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint-regressions/",
    "computed-local-reactivity-8756/controls.json"
));

fn snapshot_and_spread_count(sfc: &str) -> usize {
    let (_, script) = sfc.split_once("<script setup lang=\"ts\">").unwrap();
    let (script, _) = script.split_once("</script>").unwrap();
    parse_script_setup(script)
        .reactivity
        .losses()
        .iter()
        .filter(|loss| {
            matches!(
                loss.kind,
                ReactivityLossKind::RefValueExtract { .. }
                    | ReactivityLossKind::ReactiveSpread { .. }
            )
        })
        .count()
}

#[test]
fn original_computed_map_reads_current_ref_and_local_field() {
    assert_eq!(snapshot_and_spread_count(ORIGINAL), 0);
}

#[test]
fn immediate_reactive_maps_preserve_async_deferred_custom_and_snapshot_boundaries() {
    let cases: serde_json::Value = serde_json::from_str(CONTROLS).unwrap();
    let cases = cases.as_array().unwrap();
    assert_eq!(cases.len(), 35);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let expected = case["expectedSnapshotAndSpreadWarnings"].as_u64().unwrap() as usize;
        assert_eq!(
            snapshot_and_spread_count(source),
            expected,
            "{}",
            case["id"]
        );
    }
}

#[test]
fn map_authority_does_not_change_await_detection_or_public_scope_payload() {
    let source = "import { computed, ref } from 'vue';\nconst held = ref(0);\n\
        const shown = computed(() => [1].map(() => held.value));\n";
    let result = parse_script_setup(source);
    assert!(result.reexecuted_callbacks.is_some());
    assert!(
        result
            .reexecuted_callbacks
            .as_ref()
            .unwrap()
            .frames
            .is_empty()
    );
    let rendered = format!("{:?}", result.into_croquis());
    assert!(!rendered.contains("ReexecutedCallbacks"));
    let no_map = parse_script_setup("const shown = computed(() => 1);");
    assert!(no_map.reexecuted_callbacks.is_none());
    let top_level = parse_script_setup(&format!("{source}\nawait ready();"));
    assert!(top_level.scopes.iter().any(|scope| {
        matches!(scope.data(), crate::scope::ScopeData::ScriptSetup(data) if data.is_async)
    }));
    let nested = parse_script_setup(&format!(
        "{source}\nasync function later() {{ await ready(); }}"
    ));
    assert!(nested.scopes.iter().any(|scope| {
        matches!(scope.data(), crate::scope::ScopeData::ScriptSetup(data) if !data.is_async)
    }));
}
