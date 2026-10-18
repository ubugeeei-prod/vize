use serde_json::json;

use super::super::span_features::{
    CONTENT_MAPPER_SPAN_FEATURES_ALL, CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL,
};
use super::super::{ContentMapperSpanKind, protocol_spans};
use super::{VizeMapping, model_triplet};
use crate::virtual_ts::VizeSubSpan;

fn rows(name: &str, literal: &str) -> (vize_l0::String, vize_l0::String, Vec<VizeMapping>) {
    let source = vize_l0::cstr!("defineModel<string>({literal})");
    let generated = vize_l0::cstr!(
        "  \"{name}\": string;\n  /* __vize_model_event */ \"update:{name}\": [value: string];\n"
    );
    let authored = source.find(literal).unwrap() + 1;
    let event = generated.find("\"update:").unwrap();
    let range = authored..authored + name.len();
    let mappings = [
        2..name.len() + 4,
        3..name.len() + 3,
        event..event + name.len() + 9,
        event + 1..event + name.len() + 8,
        event + 8..event + name.len() + 8,
    ]
    .into_iter()
    .map(|gen_range| VizeMapping {
        gen_range,
        src_range: range.clone(),
        sub_spans: vec![],
    })
    .collect();
    (source, generated, mappings)
}

#[test]
fn native_inner_rows_preserve_complete_protocol_model_prop_and_event_packets() {
    for name in ["saveItem", "save-item"] {
        for quote in ['\'', '"', '`'] {
            let literal = vize_l0::cstr!("{quote}{name}{quote}");
            let (source, generated, mappings) = rows(name, &literal);
            let original = source.find(literal.as_str()).unwrap();
            let event = generated.find("\"update:").unwrap();
            let (kind, features) = if quote == '"' {
                (
                    ContentMapperSpanKind::Verbatim,
                    CONTENT_MAPPER_SPAN_FEATURES_ALL,
                )
            } else {
                (
                    ContentMapperSpanKind::Alias,
                    CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL,
                )
            };
            assert_eq!(
                serde_json::to_value(protocol_spans(&source, &generated, &mappings, &[])).unwrap(),
                json!([
                    [
                        2,
                        name.len() + 2,
                        original,
                        name.len() + 2,
                        kind as usize,
                        features
                    ],
                    [
                        event,
                        name.len() + 9,
                        original,
                        name.len() + 2,
                        ContentMapperSpanKind::Alias as usize,
                        CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL
                    ]
                ]),
                "whole quoted protocol packet for {literal}"
            );
        }
    }
}

#[test]
fn quoted_model_restoration_requires_the_exact_retained_event_triplet() {
    let (source, generated, mappings) = rows("saveItem", "'saveItem'");
    let event = &mappings[2..];
    assert!(model_triplet(&source, &generated, &event[0], &event[1], &event[2]).is_some());

    let mut mutations = Vec::new();
    let mut partial_name = event.to_vec();
    partial_name[2].gen_range.start += 1;
    mutations.push(partial_name);
    let mut partial_source = event.to_vec();
    partial_source[2].src_range.end -= 1;
    mutations.push(partial_source);
    let mut quoted_source = event.to_vec();
    for row in &mut quoted_source {
        row.src_range.start -= 1;
        row.src_range.end += 1;
    }
    mutations.push(quoted_source);
    let mut sub_span = event.to_vec();
    let span = VizeSubSpan {
        gen_range: sub_span[1].gen_range.clone(),
        src_range: sub_span[1].src_range.clone(),
    };
    sub_span[1].sub_spans.push(span);
    mutations.push(sub_span);
    let mut foreign_source = event.to_vec();
    for row in &mut foreign_source {
        row.src_range = 0..8;
    }
    mutations.push(foreign_source);
    for rows in mutations {
        assert!(model_triplet(&source, &generated, &rows[0], &rows[1], &rows[2]).is_none());
    }
    for generated in [
        generated.replace("__vize_model_event", "__vize_other_event"),
        generated.replace("update:", "delete:"),
        generated.replace("update:saveItem", "update:otherOne"),
    ] {
        assert!(model_triplet(&source, &generated, &event[0], &event[1], &event[2]).is_none());
    }
    for source in [
        source.replace("'saveItem'", "'saveItem\""),
        source.replace("'saveItem'", "'otherOne'"),
        source.replace("'saveItem'", "'save\\tem'"),
    ] {
        assert!(model_triplet(&source, &generated, &event[0], &event[1], &event[2]).is_none());
    }

    // An unowned quote rewrite retains the ordinary inner verbatim row.
    let spans = protocol_spans(&source, &generated, &mappings[..2], &[]);
    assert_eq!(
        serde_json::to_value(spans).unwrap(),
        json!([[
            3,
            8,
            mappings[0].src_range.start,
            8,
            ContentMapperSpanKind::Verbatim as usize,
            CONTENT_MAPPER_SPAN_FEATURES_ALL
        ]])
    );
}
