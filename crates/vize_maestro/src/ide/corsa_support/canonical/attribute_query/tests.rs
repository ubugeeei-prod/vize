//! Actual original generator positions must retain semantic query ownership.

use tower_lsp::lsp_types::Url;

use super::component_attribute_position;
use crate::ide::corsa_support::canonical_dependency_tests::host_document;
use crate::ide::{IdeContext, position_to_offset};
use crate::server::ServerState;

const APP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/rename-source-authority/8009/App.vue.txt"
));
#[test]
fn original_key_cannot_query_a_coarse_element_and_real_props_keep_producer_keys() {
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "<!-- 😀 unsaved -->\n"] {
            let source = vize_l0::cstr!("{prefix}{APP}").replace('\n', newline);
            let uri = Url::parse("file:///workspace/App.vue").unwrap();
            let state = ServerState::new();
            state
                .documents
                .open(uri.clone(), source.clone(), 1, "vue".into());
            let document = host_document(&uri, &source);
            let key = source.find(":key=\"`entry-${i}`\"").unwrap() + 2;
            let ctx = IdeContext::new(&state, &uri, key).unwrap();
            assert_eq!(component_attribute_position(&ctx, &document), Some(None));
            for (authored, generated) in [(":item-kind", "itemKind"), (":label", "label")] {
                let offset = source.find(authored).unwrap() + 1;
                let ctx = IdeContext::new(&state, &uri, offset).unwrap();
                let (line, character) = component_attribute_position(&ctx, &document)
                    .expect("component attribute route")
                    .expect("whole existing producer prop endpoint");
                let native =
                    position_to_offset(&document.virtual_result.code, line, character).unwrap();
                assert_eq!(
                    document
                        .virtual_result
                        .code
                        .get(native..native + generated.len()),
                    Some(generated)
                );
            }
        }
    }
}

#[test]
fn copied_query_requires_the_complete_owner_bytes_and_unique_native_identity() {
    use vize_canon::virtual_ts::VizeMapping;
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let mut document = host_document(&uri, APP);
    let result = &mut document.virtual_result;
    result.import_source_map = Default::default();
    result.code = "other".into();
    result.source_mappings = vec![VizeMapping {
        gen_range: 0..5,
        src_range: 0..5,
        sub_spans: vec![],
    }];
    assert_eq!(super::copied_position("value", result, 2), None);
    result.code = "value".into();
    assert_eq!(super::copied_position("value", result, 2), Some((0, 2)));
    result.source_mappings[0].src_range = 0..6;
    result.source_mappings[0].gen_range = 0..6;
    assert_eq!(super::copied_position("value", result, 2), None);
    result.code = "valuevalue".into();
    result.source_mappings = vec![
        VizeMapping {
            gen_range: 0..5,
            src_range: 0..5,
            sub_spans: vec![],
        },
        VizeMapping {
            gen_range: 5..10,
            src_range: 0..5,
            sub_spans: vec![],
        },
    ];
    assert_eq!(super::copied_position("value", result, 2), None);
}

#[test]
fn ordinary_values_events_models_and_native_dom_leave_this_attribute_guard() {
    let source = "<script setup lang=\"ts\">import Child from './Child.vue'; const value = 1;</script><template><Child :label=\"value\" @change=\"value\" v-model=\"value\" v-model:title=\"value\"/><button style=\"color: red\" /></template>";
    let uri = Url::parse("file:///workspace/Parent.vue").unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let document = host_document(&uri, source);
    for (prefix, within) in [
        (":label=\"value", 8),
        ("@change", 2),
        ("v-model=", 2),
        ("v-model:title", 9),
        ("style=", 2),
    ] {
        let ctx = IdeContext::new(&state, &uri, source.find(prefix).unwrap() + within).unwrap();
        assert_eq!(component_attribute_position(&ctx, &document), None);
    }
}
