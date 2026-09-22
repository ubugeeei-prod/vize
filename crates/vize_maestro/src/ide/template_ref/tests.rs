use super::*;
use tower_lsp::lsp_types::Url;

use crate::server::ServerState;

#[test]
fn static_ref_value_finds_authored_value_only() {
    let source = r#"<template><button data-ref="skip" :ref="dynamic" ref="button" /></template>"#;
    let offset = source.find("button\"").unwrap() + "button".len();
    let value = static_ref_value_at_offset(source, offset).unwrap();

    assert_eq!(value.ref_name, "button");
    assert_eq!(&source[value.start..value.end], "button");
    assert!(static_ref_value_at_offset(source, source.find("dynamic").unwrap()).is_none());
    assert!(static_ref_value_at_offset(source, source.find("skip").unwrap()).is_none());
}

#[test]
fn target_maps_static_ref_to_use_template_ref_binding() {
    let source = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'
const el = useTemplateRef<HTMLButtonElement>('button')
</script>

<template><button ref="button" /></template>
"#;
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);
    let offset = source.rfind("button\"").unwrap() + "button".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let target = target_at_offset(&ctx).unwrap();

    assert_eq!(target.ref_name, "button");
    assert_eq!(target.binding_name, "el");
    assert_eq!(&source[target.binding_start..target.binding_end], "el");
    assert_eq!(&source[target.value_start..target.value_end], "button");
}

#[test]
fn target_rejects_missing_use_template_ref_binding() {
    let source = r#"<script setup lang="ts">
const el = 1
</script>
<template><button ref="button" /></template>
"#;
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);
    let offset = source.rfind("button\"").unwrap() + "button".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert!(target_at_offset(&ctx).is_none());
}

const RESIDENT_REF_SFC: &str = "<script setup lang=\"ts\">\r\n// 雪😀\r\nconst button = useTemplateRef('button')\r\n</script>\r\n<template>雪😀<button ref=\"button\" /></template>";

fn resident_ref_context<'a>(state: &'a ServerState, uri: &'a Url, source: &str) -> IdeContext<'a> {
    let offset = source.rfind("button\"").unwrap() + 1;
    IdeContext::testing(state, uri, offset, source.into())
}

fn resident_ref_responses(ctx: &IdeContext<'_>) -> serde_json::Value {
    serde_json::json!({
        "hover": crate::ide::HoverService::hover(ctx),
        "definition": crate::ide::DefinitionService::definition(ctx),
    })
}

#[test]
fn resident_template_refs_preserve_authored_responses_and_inflight_revisions() {
    use vize_resident::DescriptorStats;

    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Ref.vue").unwrap();
    // Explicit request text wins over a newer document-store snapshot.
    state
        .documents
        .open(uri.clone(), "<template/>".into(), 2, "vue".into());
    let original = resident_ref_context(&state, &uri, RESIDENT_REF_SFC);
    let first = resident_ref_responses(&original);
    assert_eq!(
        first["hover"]["range"],
        serde_json::json!({
            "start": {"line": 4, "character": 26},
            "end": {"line": 4, "character": 32},
        })
    );
    assert!(
        first["hover"]["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("useTemplateRef()")
    );
    assert_eq!(
        first["definition"],
        serde_json::json!({
            "uri": uri.as_str(),
            "range": {
                "start": {"line": 2, "character": 6},
                "end": {"line": 2, "character": 12},
            },
        })
    );
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
    assert_eq!(
        resident_ref_responses(&resident_ref_context(&state, &uri, RESIDENT_REF_SFC)),
        first
    );
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 0
        }
    );

    let edited = RESIDENT_REF_SFC
        .replace("const button", "const renamed")
        .replace("<script setup", "\r\n<script setup");
    let updated = resident_ref_context(&state, &uri, &edited);
    let changed = resident_ref_responses(&updated);
    assert_eq!(
        changed["definition"]["range"],
        serde_json::json!({
            "start": {"line": 3, "character": 6},
            "end": {"line": 3, "character": 13},
        })
    );
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
    let clean = ServerState::new();
    assert_eq!(
        changed,
        resident_ref_responses(&resident_ref_context(&clean, &uri, &edited))
    );
    assert_eq!(resident_ref_responses(&original), first);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 0,
            parses: 0
        }
    );
}

#[test]
fn resident_template_refs_cache_rejection_and_recover_after_reopen() {
    use vize_resident::DescriptorStats;

    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Ref.vue").unwrap();
    let broken = RESIDENT_REF_SFC.replace("</template>", "");
    for parses in [1, 0] {
        let ctx = resident_ref_context(&state, &uri, &broken);
        assert!(target_at_offset(&ctx).is_none());
        assert_eq!(
            resident_ref_responses(&ctx),
            serde_json::json!({"hover": null, "definition": null})
        );
        assert_eq!(
            state.resident.take_stats(),
            DescriptorStats { lookups: 1, parses }
        );
    }
    let fixed = resident_ref_responses(&resident_ref_context(&state, &uri, RESIDENT_REF_SFC));
    assert!(!fixed["definition"].is_null());
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
    state.close_document(&uri);
    let _closed = state.resident.take_stats();
    assert_eq!(
        resident_ref_responses(&resident_ref_context(&state, &uri, RESIDENT_REF_SFC)),
        fixed
    );
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
}
