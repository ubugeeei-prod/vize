//! Retain exact quoted definition endpoints in the protocol consumer only.

use super::span_features::content_mapper_span_features;
use super::{ContentMapperSpanKind, SpanCandidate, VizeMapping, candidate};
use std::ops::Range;
use vize_l0::SmallVec;

pub(super) fn candidates(
    source: &str,
    generated: &str,
    mappings: &[VizeMapping],
) -> Vec<SpanCandidate> {
    let mut result = Vec::new();
    let model_names: SmallVec<[Range<usize>; 4]> = mappings
        .windows(3)
        .filter_map(|rows| {
            let [whole, interior, name] = rows else {
                return None;
            };
            model_triplet(source, generated, whole, interior, name).map(|_| whole.src_range.clone())
        })
        .collect();
    let mut mappings = mappings.iter().peekable();
    while let Some(mapping) = mappings.next() {
        let mut following = mappings.clone();
        if let (Some(interior), Some(name)) = (following.next(), following.next())
            && let Some(quoted) = model_triplet(source, generated, mapping, interior, name)
        {
            result.push(quoted);
            let _ = mappings.next();
            let _ = mappings.next();
            continue;
        }
        if let Some(next) = mappings.peek()
            && model_names.contains(&mapping.src_range)
            && let Some(quoted) = model_prop_pair(source, generated, mapping, next)
        {
            result.push(quoted);
            let _ = mappings.next();
            continue;
        }
        if let Some(next) = mappings.peek()
            && let Some(quoted) = quoted_pair(source, generated, mapping, next)
        {
            result.push(quoted);
            let _ = mappings.next();
            continue;
        }
        if mapping.sub_spans.is_empty() {
            result.extend(candidate(
                source,
                generated,
                mapping.gen_range.clone(),
                mapping.src_range.clone(),
            ));
        } else {
            result.extend(mapping.sub_spans.iter().filter_map(|span| {
                candidate(
                    source,
                    generated,
                    span.gen_range.clone(),
                    span.src_range.clone(),
                )
            }));
        }
    }
    result
}

fn model_prop_pair(
    source: &str,
    generated: &str,
    whole: &VizeMapping,
    interior: &VizeMapping,
) -> Option<SpanCandidate> {
    if !whole.sub_spans.is_empty()
        || !interior.sub_spans.is_empty()
        || whole.src_range != interior.src_range
        || whole.gen_range.start.checked_add(1)? != interior.gen_range.start
        || whole.gen_range.end.checked_sub(1)? != interior.gen_range.end
    {
        return None;
    }
    let text = generated.get(whole.gen_range.clone())?;
    let authored = source.get(whole.src_range.clone())?;
    if !text.starts_with('"')
        || !text.ends_with('"')
        || text.get(1..text.len().checked_sub(1)?)? != authored
    {
        return None;
    }
    let original = whole.src_range.start.checked_sub(1)?..whole.src_range.end.checked_add(1)?;
    let candidate = candidate(source, generated, whole.gen_range.clone(), original.clone())?;
    (candidate.generated == whole.gen_range && candidate.original == original).then_some(candidate)
}

/// The native model producer retains its inner name at three event endpoints.
/// Protocol-v1 consumers keep the original complete quoted alias instead;
/// an interior alias would split otherwise exact public model-prop rows.
fn model_triplet(
    source: &str,
    generated: &str,
    whole: &VizeMapping,
    interior: &VizeMapping,
    name: &VizeMapping,
) -> Option<SpanCandidate> {
    if !whole.sub_spans.is_empty()
        || !interior.sub_spans.is_empty()
        || !name.sub_spans.is_empty()
        || whole.src_range.is_empty()
        || whole.src_range != interior.src_range
        || whole.src_range != name.src_range
        || whole.gen_range.start.checked_add(1)? != interior.gen_range.start
        || whole.gen_range.end.checked_sub(1)? != interior.gen_range.end
        || interior.gen_range.start.checked_add("update:".len())? != name.gen_range.start
        || interior.gen_range.end != name.gen_range.end
        || !generated
            .get(..whole.gen_range.start)?
            .ends_with("  /* __vize_model_event */ ")
    {
        return None;
    }
    let text = generated.get(whole.gen_range.clone())?;
    if !text.starts_with('"') || !text.ends_with('"') {
        return None;
    }
    let generated_name = generated
        .get(interior.gen_range.clone())?
        .strip_prefix("update:")?;
    let authored = source.get(whole.src_range.clone())?;
    if authored.is_empty() || authored.contains('\\') || generated_name != authored {
        return None;
    }
    let original = whole.src_range.start.checked_sub(1)?..whole.src_range.end.checked_add(1)?;
    let quoted = source.get(original.clone())?;
    if !matches!(quoted.as_bytes(), [open @ (b'\'' | b'"' | b'`'), .., close] if open == close)
        || quoted.get(1..quoted.len().checked_sub(1)?)? != authored
    {
        return None;
    }
    let candidate = candidate(source, generated, whole.gen_range.clone(), original.clone())?;
    (candidate.generated == whole.gen_range && candidate.original == original).then_some(candidate)
}

fn quoted_pair(
    source: &str,
    generated: &str,
    whole: &VizeMapping,
    interior: &VizeMapping,
) -> Option<SpanCandidate> {
    if !whole.sub_spans.is_empty()
        || !interior.sub_spans.is_empty()
        || whole.src_range.is_empty()
        || whole.src_range != interior.src_range
        || whole.gen_range.start.checked_add(1)? != interior.gen_range.start
        || whole.gen_range.end.checked_sub(1)? != interior.gen_range.end
    {
        return None;
    }
    let text = generated.get(whole.gen_range.clone())?;
    let quote = *text.as_bytes().first()?;
    if !matches!(quote, b'\'' | b'"') || text.as_bytes().last() != Some(&quote) {
        return None;
    }
    let authored = source.get(whole.src_range.clone())?;
    if authored.contains('\\') || text.get(1..text.len().checked_sub(1)?)? != authored {
        return None;
    }
    let original = whole.src_range.start.checked_sub(1)?..whole.src_range.end.checked_add(1)?;
    if source.get(original.clone())? != text {
        return None;
    }
    // VizeMapping has no feature field. Preserve the computed protocol
    // metadata of both ordinary candidates and the full replacement.
    let whole_candidate = candidate(
        source,
        generated,
        whole.gen_range.clone(),
        whole.src_range.clone(),
    )?;
    let interior_candidate = candidate(
        source,
        generated,
        interior.gen_range.clone(),
        interior.src_range.clone(),
    )?;
    if whole_candidate.generated != interior.gen_range
        || interior_candidate.generated != interior.gen_range
        || whole_candidate.kind != ContentMapperSpanKind::Verbatim
        || interior_candidate.kind != ContentMapperSpanKind::Verbatim
    {
        return None;
    }
    let features = content_mapper_span_features(
        generated,
        whole_candidate.generated.start,
        whole_candidate.kind,
    );
    if features
        != content_mapper_span_features(
            generated,
            interior_candidate.generated.start,
            interior_candidate.kind,
        )
        || features
            != content_mapper_span_features(
                generated,
                whole.gen_range.start,
                ContentMapperSpanKind::Verbatim,
            )
    {
        return None;
    }
    Some(SpanCandidate {
        generated: whole.gen_range.clone(),
        original,
        kind: ContentMapperSpanKind::Verbatim,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::{Value, json};

    use super::super::{generate_vue_content_mapper_transform, protocol_spans};
    use crate::virtual_ts::{VizeMapping, VizeSubSpan};

    const ORIGINAL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/",
        "content-mapper-quoted-event-definition/input.vue.txt"
    ));
    const CASES: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/",
        "content-mapper-quoted-event-definition/cases.json"
    ));

    #[test]
    #[expect(clippy::string_slice, reason = "tests assert by panicking")]
    fn original_call_signature_definition_keeps_the_complete_quoted_span() {
        let original_fixture = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../vize/tests/fixtures/content_mapper_project/src/CallSignatureChild.vue"
        ));
        assert_eq!(
            ORIGINAL, original_fixture,
            "original input bytes stay intact"
        );
        let corpus: Value = serde_json::from_str(CASES).unwrap();
        let expected = &corpus["original"];
        let result =
            generate_vue_content_mapper_transform(Path::new("CallSignatureChild.vue"), ORIGINAL)
                .expect("original transform");
        assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
        let authored_map = result.text.find("type __VizeAuthoredEventMap").unwrap();
        let literal = expected["literal"].as_str().unwrap();
        let generated = authored_map + result.text[authored_map..].find(literal).unwrap();
        let original_start = expected["sourceRange"][0].as_u64().unwrap() as usize;
        let original_end = expected["sourceRange"][1].as_u64().unwrap() as usize;
        assert_eq!(&ORIGINAL[original_start..original_end], literal);
        let endpoint: Vec<_> = result
            .mappings
            .iter()
            .filter(|span| span.0[0] == generated)
            .collect();
        assert_eq!(
            serde_json::to_value(endpoint).unwrap(),
            json!([[
                generated,
                expected["spanLength"],
                original_start,
                original_end - original_start,
                expected["kind"],
                expected["features"]
            ]]),
            "complete original quoted definition endpoint"
        );
        assert!(
            result
                .mappings
                .windows(2)
                .all(|spans| { spans[0].0[0] + spans[0].0[1] <= spans[1].0[0] }),
            "protocol generated spans must be disjoint"
        );
    }

    #[test]
    fn quoted_pairs_and_refusals_preserve_all_authored_protocol_results() {
        let corpus: Value = serde_json::from_str(CASES).unwrap();
        let cases = corpus["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 16);
        let mut actual = Vec::new();
        let mut expected = Vec::new();
        for case in cases {
            let mappings: Vec<_> = case["mappings"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| VizeMapping {
                    gen_range: range(row, "generated"),
                    src_range: range(row, "original"),
                    sub_spans: row
                        .get("subSpans")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .map(|span| VizeSubSpan {
                            gen_range: range(span, "generated"),
                            src_range: range(span, "original"),
                        })
                        .collect(),
                })
                .collect();
            let spans = protocol_spans(
                case["source"].as_str().unwrap(),
                case["generated"].as_str().unwrap(),
                &mappings,
                &[],
            );
            actual.push(json!({"name":case["name"],"spans":spans}));
            expected.push(json!({"name":case["name"],"spans":case["expected"]}));
        }
        assert_eq!(actual, expected, "all complete quoted-span corpus packets");
    }

    fn range(value: &Value, key: &str) -> std::ops::Range<usize> {
        value[key][0].as_u64().unwrap() as usize..value[key][1].as_u64().unwrap() as usize
    }
}

#[cfg(test)]
mod model_tests {
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
                    serde_json::to_value(protocol_spans(&source, &generated, &mappings, &[]))
                        .unwrap(),
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

        // An unowned quote rewrite retains the ordinary inner verbatim row;
        // its generated quotes only carry diagnostics to the same name.
        let spans = protocol_spans(&source, &generated, &mappings[..2], &[]);
        let name = mappings[0].src_range.start;
        let atom = ContentMapperSpanKind::Atom as usize;
        assert_eq!(
            serde_json::to_value(spans).unwrap(),
            json!([
                [2, 1, name, 8, atom, 0],
                [
                    3,
                    8,
                    name,
                    8,
                    ContentMapperSpanKind::Verbatim as usize,
                    CONTENT_MAPPER_SPAN_FEATURES_ALL
                ],
                [11, 1, name, 8, atom, 0]
            ])
        );
    }
}
