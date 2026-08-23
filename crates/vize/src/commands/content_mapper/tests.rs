use serde_json::{Value, json};

use super::test_support::{
    exchange, frames, initialize_request, open_project_request, transform_request,
};

#[test]
fn accepts_legacy_initialize_version_without_echoing_it() {
    let input = frames(&[
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": 1,
                "positionEncodings": ["utf-16", "utf-8"],
                "locale": "en-US"
            }
        }),
        open_project_request(2, Value::Null, json!({})),
        transform_request(
            3,
            "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template>{{ count }}</template>\n",
        ),
    ]);
    let responses = exchange(&input);

    assert!(responses[0]["result"].get("protocolVersion").is_none());
    assert_eq!(responses[0]["result"]["positionEncoding"], "utf-8");
    assert_eq!(responses[0]["result"]["diagnosticSource"], "vize");
    assert!(responses[2]["result"].get("scriptKind").is_none());
    assert_eq!(responses[2]["result"]["extension"], ".tsx");
    assert!(
        responses[2]["result"]["text"]
            .as_str()
            .unwrap()
            .contains("const count = 1")
    );
    assert!(
        !responses[2]["result"]["mappings"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn negotiates_utf8_without_legacy_protocol_version() {
    let input = frames(&[json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "positionEncodings": ["utf-16", "utf-8"],
            "locale": "en-US"
        }
    })]);
    let responses = exchange(&input);

    assert!(responses[0]["result"].get("protocolVersion").is_none());
    assert_eq!(responses[0]["result"]["positionEncoding"], "utf-8");
    assert_eq!(responses[0]["result"]["diagnosticSource"], "vize");
}

#[test]
fn parse_errors_are_successful_transform_results() {
    let input = frames(&[
        initialize_request(),
        open_project_request(2, Value::Null, json!({})),
        transform_request(3, "<template><div></template>"),
    ]);
    let responses = exchange(&input);

    assert!(responses[2].get("error").is_none());
    assert!(
        !responses[2]["result"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(responses[2]["result"]["diagnostics"][0]["code"], 100_002);
}

#[test]
fn rejects_transform_before_initialize() {
    let input = frames(&[json!({
        "jsonrpc": "2.0",
        "id": "early",
        "method": "transform",
        "params": {
            "fileName": "App.vue",
            "content": "<template />",
            "projectHandle": "p1"
        }
    })]);
    let responses = exchange(&input);

    assert_eq!(responses[0]["id"], "early");
    assert_eq!(responses[0]["error"]["code"], -32002);
}

#[test]
fn transform_honors_options_api_project_option() {
    let source = r#"<script lang="ts">
export default { data() { return { count: 1 } } }
</script>
<template>{{ count }}</template>
"#;
    let enabled = frames(&[
        initialize_request(),
        open_project_request(2, json!({ "optionsApi": true }), json!({})),
        transform_request(3, source),
    ]);
    let disabled = frames(&[
        initialize_request(),
        open_project_request(2, json!({ "optionsApi": false }), json!({})),
        transform_request(3, source),
    ]);
    let enabled_responses = exchange(&enabled);
    let disabled_responses = exchange(&disabled);

    assert!(
        enabled_responses[2]["result"]["text"]
            .as_str()
            .unwrap()
            .contains("__VizeOptionsBinding")
    );
    assert!(
        !disabled_responses[2]["result"]["text"]
            .as_str()
            .unwrap()
            .contains("__VizeOptionsBinding")
    );
}

#[test]
fn transform_honors_no_unused_locals_compiler_option() {
    let source = r#"<script setup lang="ts">
const used = 1
const unused = 2
</script>
<template>{{ used }}</template>
"#;
    let preserving = frames(&[
        initialize_request(),
        open_project_request(2, Value::Null, json!({ "noUnusedLocals": true })),
        transform_request(3, source),
    ]);
    let suppressing = frames(&[
        initialize_request(),
        open_project_request(2, Value::Null, json!({ "noUnusedLocals": false })),
        transform_request(3, source),
    ]);
    let preserving = exchange(&preserving);
    let suppressing = exchange(&suppressing);
    let preserving = preserving[2]["result"]["text"].as_str().unwrap();
    let suppressing = suppressing[2]["result"]["text"].as_str().unwrap();

    assert!(preserving.contains("void used;"), "{preserving}");
    assert!(!preserving.contains("void unused;"), "{preserving}");
    assert!(suppressing.contains("void used;"), "{suppressing}");
    assert!(suppressing.contains("void unused;"), "{suppressing}");
}

#[test]
fn transform_exposes_template_diagnostic_directives() {
    let source = r#"<script setup lang="ts">
const count = 1
</script>
<template>
  <!-- @vue-expect-error -->
  {{ count.bad }}
</template>
"#;
    let input = frames(&[
        initialize_request(),
        open_project_request(2, Value::Null, json!({})),
        transform_request(3, source),
    ]);
    let responses = exchange(&input);
    let directives = &responses[2]["result"]["diagnosticDirectives"];

    assert_eq!(
        directives["unusedExpectDirectiveDiagnostics"],
        json!([{ "code": 4, "messageText": "Unused '@vue-expect-error' directive" }]),
        "{}",
        responses[2]
    );
    let tuple = directives["directives"][0].as_array().unwrap();
    assert_eq!(tuple.len(), 6, "{}", responses[2]);
    assert_eq!(tuple[4], 1, "expect policy: {}", responses[2]);
}

#[test]
fn transform_exposes_semantic_links() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue'
const count = ref(1)
</script>
<template>{{ count }}</template>
"#;
    let input = frames(&[
        initialize_request(),
        open_project_request(2, Value::Null, json!({})),
        transform_request(3, source),
    ]);
    let responses = exchange(&input);

    assert!(
        !responses[2]["result"]["semanticLinks"]
            .as_array()
            .unwrap()
            .is_empty(),
        "{}",
        responses[2]
    );
}

#[test]
fn transform_defaults_options_api_on_for_absent_null_and_empty_options() {
    let source = r#"<script lang="ts">
export default { data() { return { count: 1 } } }
</script>
<template>{{ count }}</template>
"#;
    let without_options = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "openProject",
        "params": {
            "configFileName": "/project/tsconfig.json",
            "projectHandle": super::test_support::PROJECT_HANDLE,
            "compilerOptions": {}
        }
    });
    for open in [
        without_options,
        open_project_request(2, Value::Null, json!({})),
        open_project_request(2, json!({}), json!({})),
    ] {
        let input = frames(&[initialize_request(), open, transform_request(3, source)]);
        let responses = exchange(&input);

        assert!(
            responses[2]["result"]["text"]
                .as_str()
                .unwrap()
                .contains("__VizeOptionsBinding"),
            "expected default-on Options API output: {}",
            responses[2]
        );
    }
}
