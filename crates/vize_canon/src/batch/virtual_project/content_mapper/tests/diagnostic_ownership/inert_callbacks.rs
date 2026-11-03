//! Exact inert event-navigation rows preserve every mapped contextual owner.

use super::{FORM, added_echoes, packet, rows};
use serde_json::Value;

pub(super) fn form_events_and_callback_inference_keep_one_binding_ownership_role() {
    for newline in ["\n", "\r\n"] {
        for callback in ["", " :validate-form-field-on=\"(field) => !!field\""] {
            let source = FORM
                .replace(
                    ":schema=\"schema\"",
                    &format!(":schema=\"schema\"{callback}"),
                )
                .replace('\n', newline);
            let before = packet("Form.art.vue", &source, false);
            let after = packet("Form.art.vue", &source, true);
            let echoes = added_echoes(&before, &after);
            assert!(!echoes.is_empty());
            let fields = [
                [
                    "initial-state",
                    "{ role: 'ok', tags: [] }",
                    "schema",
                    "schema",
                ],
                [
                    "initial-state",
                    "{ role: null, tags: [] }",
                    "schema",
                    "schema",
                ],
                [
                    "initial-state",
                    "{ role: 'ok', tags: [] }",
                    "schema",
                    "schema",
                ],
            ];
            let mut expected = Vec::new();
            for field in fields {
                expected.extend(field);
                if !callback.is_empty() {
                    expected.extend(["validate-form-field-on", "(field) => !!field"]);
                }
            }
            if callback.is_empty() {
                expected.extend([
                    "{ role: 'ok', tags: [] }",
                    "schema",
                    "{ role: null, tags: [] }",
                    "schema",
                    "{ role: 'ok', tags: [] }",
                    "schema",
                ]);
            } else {
                // A callback's canonical packet now precedes the ordinary
                // source expressions at that exact root usage. Keep both
                // resolver roles and both bare values for every variant.
                for field @ [_, value, _, schema] in fields {
                    expected.extend(field);
                    expected.extend(field);
                    expected.extend([value, schema]);
                }
            }
            // The event-handler emit-argument inference phase follows the
            // authored root stream, still retaining every sibling binding.
            expected.extend(fields.iter().flatten().copied());
            let actual = echoes
                .into_iter()
                .map(|[original, length, _, _, _, _]| {
                    source
                        .get(original..original + length)
                        .expect("exact authored binding")
                })
                .collect::<Vec<_>>();
            assert_eq!(
                actual, expected,
                "the complete ownership vector includes ordinary siblings and the explicitly unmapped callback navigation copies"
            );
        }
    }
}

pub(super) fn inline_callback_copies_stay_ignored_and_never_hide_the_canonical_check() {
    // #10739: the event-navigation copy had an `any` context and no ignore, while
    // an ignored inference copy authorized hiding the contextual assignment.
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "<!-- 日本 🧭 -->"] {
            for callback in ["(field) => !!field", "(field) => field.missing"] {
                let source = format!("{prefix}<script setup lang=\"ts\">\nimport Comp from './Comp.vue'\nconst noop = () => {{}}\n</script>\n<template>\n  <Comp :pick=\"{callback}\" @save=\"noop\" />\n</template>\n")
                    .replace('\n', newline);
                let before = packet("src/Min.vue", &source, false);
                let packet = packet("src/Min.vue", &source, true);
                let echoes = added_echoes(&before, &packet);
                assert_callback_navigation_echoes(&packet, &source, callback, &echoes);
                let text = packet["text"].as_str().expect("complete generated source");
                let copies = text
                    .match_indices(callback)
                    .map(|(at, _)| {
                        let line_start = text[..at].rfind('\n').map_or(0, |end| end + 1);
                        let previous = text[..line_start.saturating_sub(1)]
                            .rsplit('\n')
                            .next()
                            .unwrap_or_default();
                        if previous.trim_start().starts_with("// @ts-ignore") {
                            "ignored"
                        } else {
                            text[line_start..at].trim_start()
                        }
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    copies,
                    [
                        "ignored",
                        "ignored",
                        "const __vize_prop_check_0_pick: $ResolvedProp<typeof __vize_resolved_Comp_props_0, typeof __vize_selected_Comp_props_0, 'pick', $CallableProp<$PropValue<__Comp_ValueProps_0, 'pick', __Comp_FallthroughValue_0<'pick'>>>> = ",
                        "ignored",
                    ],
                    "{text}"
                );
            }
        }
    }
}

pub(super) fn proved_expected_callback_geometry_never_hides_its_contextual_body() {
    let source = "<script setup lang=\"ts\">import Comp from './Comp.vue'; const noop = () => {};</script><template><!-- @vue-expect-error --><Comp :id=\"{ a: 1 }\" :pick=\"(field) => !!field\" @save=\"noop\" /></template>";
    let mut before = packet("Callback.vue", source, false);
    let mut after = packet("Callback.vue", source, true);
    let table = after
        .as_object_mut()
        .unwrap()
        .remove("diagnosticDirectives")
        .unwrap();
    before
        .as_object_mut()
        .unwrap()
        .remove("diagnosticDirectives");
    assert_eq!(
        after, before,
        "all text, mappings, editor features, links and authored diagnostics remain exact"
    );
    let callback = source.find("(field) => !!field").unwrap();
    let rows = table["directives"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| serde_json::from_value::<[usize; 6]>(row.clone()).unwrap())
        .collect::<Vec<_>>();
    let expects = rows.iter().filter(|row| row[4] == 1).collect::<Vec<_>>();
    assert_eq!(expects.len(), 1, "unused EXPECT remains one aggregate");
    let prop = source.find(":pick=").unwrap() + 1;
    let callback_echoes = rows
        .iter()
        .filter(|row| row[4] == 0 && [prop, callback].contains(&row[0]))
        .copied()
        .collect::<Vec<_>>();
    assert_callback_navigation_echoes(&after, source, "(field) => !!field", &callback_echoes);
    let [_, _, start, end, _, _] = *expects[0];
    let body_rows = after["mappings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row[2] == callback && row[3] == "(field) => !!field".len())
        .collect::<Vec<_>>();
    assert_eq!(
        body_rows.len(),
        2,
        "one inference copy and one authored contextual check"
    );
    for row in body_rows {
        let generated = row[0].as_u64().unwrap() as usize;
        let length = row[1].as_u64().unwrap() as usize;
        let editor_features = row[5].as_u64().unwrap();
        assert_eq!(
            start <= generated && generated + length <= end,
            editor_features != 0,
            "only the actual authored body belongs to EXPECT"
        );
    }
}

fn assert_callback_navigation_echoes(
    packet: &Value,
    source: &str,
    callback: &str,
    actual: &[[usize; 6]],
) {
    let text = packet["text"].as_str().expect("complete virtual text");
    let navigation = text
        .find("const __vize_events_resolved_0 =")
        .expect("event navigation phase");
    let key = navigation
        + text[navigation..]
            .find("\"pick\"")
            .expect("literal navigation key");
    let value = key + "\"pick\": ".len();
    let prop = source.find(":pick=").expect("authored directive") + 1;
    let authored = source.find(callback).expect("authored callback");
    assert_eq!(
        actual,
        [
            [prop, "pick".len(), key, key + "\"pick\"".len(), 0, 0],
            [
                authored,
                callback.len(),
                value,
                value + callback.len(),
                0,
                0
            ],
        ]
    );
    let mappings = rows(&packet["mappings"]);
    for [_, _, start, end, _, _] in actual {
        assert!(
            mappings
                .iter()
                .all(|[at, length, ..]| at + length <= *start || *end <= *at),
            "only unmapped navigation copies may be ignored; every contextual mapping remains visible"
        );
    }
}
