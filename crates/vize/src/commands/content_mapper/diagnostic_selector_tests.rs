use serde_json::json;

use super::test_support::{
    exchange, frames, initialize_request, open_project_request, transform_request,
};

#[test]
fn loose_component_prop_selectors_are_sent_only_to_negotiated_hosts() {
    let source = r#"<!-- @checkUnknownProps false -->
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :items="[1, 2]" :buffer="200" /></template>
"#;
    let requests = |initialize| {
        frames(&[
            initialize,
            open_project_request(2, json!({}), json!({})),
            transform_request(3, source),
        ])
    };
    let legacy = exchange(&requests(initialize_request()));
    let negotiated = exchange(&requests(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "positionEncodings": ["utf-8"],
            "capabilities": { "diagnosticDirectiveCodeSelectors": true }
        }
    })));

    let legacy = legacy[2]["result"]["diagnosticDirectives"]["directives"]
        .as_array()
        .unwrap();
    assert!(
        legacy
            .iter()
            .all(|tuple| tuple.as_array().unwrap().len() == 5)
    );

    let text = negotiated[2]["result"]["text"].as_str().unwrap();
    let tuples = negotiated[2]["result"]["diagnosticDirectives"]["directives"]
        .as_array()
        .unwrap();
    let selected = tuples
        .iter()
        .filter(|tuple| tuple.as_array().is_some_and(|tuple| tuple.len() == 6))
        .filter(|tuple| tuple[4] == 0 && tuple[5] == 2353)
        .cloned()
        .collect::<Vec<_>>();
    let expected = ["items", "buffer"]
        .map(|attribute| {
            let original = source.find(&format!(":{attribute}=")).unwrap() + 1;
            let generated = text.find(&format!("\"{attribute}\": ")).unwrap();
            json!([
                original,
                attribute.len(),
                generated,
                generated + attribute.len() + 2,
                0,
                2353,
            ])
        })
        .to_vec();
    assert_eq!(selected, expected);
}
