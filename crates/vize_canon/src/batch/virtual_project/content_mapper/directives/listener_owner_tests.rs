#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::path::Path;

use super::super::protocol::{ContentMapperDiagnosticDirective, DIRECTIVE_POLICY_IGNORE};
use super::super::{ContentMapperTransform, generate_vue_content_mapper_transform};

fn transform(source: &str) -> ContentMapperTransform {
    generate_vue_content_mapper_transform(Path::new("/project/App.vue"), source).unwrap()
}

fn covered(rows: &[ContentMapperDiagnosticDirective], position: usize) -> bool {
    rows.iter().any(|row| {
        let [_, _, start, end, _, _] = row.0;
        start <= position && position < end
    })
}

fn owner_rows(result: &ContentMapperTransform) -> &[ContentMapperDiagnosticDirective] {
    result
        .diagnostic_directives
        .as_ref()
        .map_or(&[], |table| table.directives.as_slice())
}

fn mapped_positions(result: &ContentMapperTransform, original: usize) -> Vec<usize> {
    result
        .mappings
        .iter()
        .filter_map(
            |super::super::protocol::ContentMapperSpan(
                [generated, length, start, original_length, _, _],
            )| {
                (*start <= original && original < start + original_length)
                    .then_some(*generated + (*length / 2))
            },
        )
        .collect()
}

#[test]
fn listener_detection_requires_the_mapping_own_attribute_boundary() {
    for (source, needle, expected) in [
        ("<Child @click = \" handler\" />", "handler", true),
        ("<Child v-on:click = \"handler\" />", "handler", true),
        ("<Child :on-click = \"handler\" />", "handler", false),
        (
            "<Child @click=\"first\" :value=\"second\" />",
            "second",
            false,
        ),
        ("<Child title=\"@click\" />", "@click", false),
        ("<Child @click handler />", "click", false),
    ] {
        let start = source.rfind(needle).unwrap();
        assert_eq!(
            super::authored_component_listener(source, &(start..start + needle.len())),
            expected,
            "{source}"
        );
    }
}

#[test]
fn listener_literals_and_ordinary_props_choose_inverse_owners() {
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "日本🧭"] {
            for event in [
                "@click=\"handler\"",
                "v-on:click=\"handler\"",
                "@click = \"handler\"",
                "v-on:click = \" handler\"",
            ] {
                let source = format!(
                    "<script setup lang=\"ts\">import Child from './Child.vue'; const count = 'bad'; const handler = (event: MouseEvent) => event.button;</script>{newline}<template>{prefix}<Child :count=\"count\" {event} /></template>"
                );
                let result = transform(&source);
                let rows = owner_rows(&result);
                let generated = result.text.as_str();
                let count_check = generated
                    .find("const __vize_prop_check_0_count")
                    .expect("ordinary per-prop check");
                let count_literal = generated
                    .find("\"count\": count")
                    .expect("ordinary whole-props literal");
                let listener_check = generated
                    .find("const __vize_prop_check_0_onClick")
                    .expect("listener per-prop check");
                let listener_literals = generated
                    .match_indices("\"onClick\": handler")
                    .map(|(start, _)| start)
                    .collect::<Vec<_>>();

                assert!(covered(rows, count_check), "{event}: {rows:?}");
                assert!(!covered(rows, count_literal), "{event}: {rows:?}");
                assert!(!covered(rows, listener_check), "{event}: {rows:?}");
                assert!(!listener_literals.is_empty(), "{generated}");
                assert!(
                    listener_literals
                        .iter()
                        .all(|position| covered(rows, *position)),
                    "{event}: {rows:?}\n{generated}"
                );
                assert!(rows.iter().all(|row| row.0[4] == DIRECTIVE_POLICY_IGNORE));
            }
        }
    }
}

#[test]
fn explicit_on_click_props_never_use_listener_ownership() {
    for attribute in [":on-click", ":onClick"] {
        let source = format!(
            "<script setup lang=\"ts\">import Child from './Child.vue'; const handler = 'bad';</script><template><Child {attribute}=\"handler\" /></template>"
        );
        let result = transform(&source);
        let rows = owner_rows(&result);
        let generated = result.text.as_str();
        let check = generated
            .find("const __vize_prop_check_0_")
            .expect("explicit prop check");
        let literal = generated
            .find("\"onClick\": handler")
            .expect("explicit prop literal");

        assert!(covered(rows, check), "{attribute}: {rows:?}");
        assert!(!covered(rows, literal), "{attribute}: {rows:?}");
    }
}

#[test]
fn inline_listener_callback_keeps_one_uncovered_body_owner() {
    let source = "<script setup lang=\"ts\">import Child from './Child.vue';</script><template><Child @click=\"(event) => event.missing\" /></template>";
    let result = transform(source);
    let rows = owner_rows(&result);
    let original = source.find("event.missing").expect("callback body");
    let positions = mapped_positions(&result, original);

    assert!(!positions.is_empty(), "callback body must stay mapped");
    assert!(
        positions.iter().any(|position| !covered(rows, *position)),
        "callback-body diagnostics must retain an active owner: {rows:?}"
    );
}

#[test]
fn authored_directives_own_sibling_prop_and_listener_errors_without_synthetic_overlap() {
    for token in ["@vue-ignore", "@vue-expect-error"] {
        let source = format!(
            "<script setup lang=\"ts\">import Child from './Child.vue'; const wrong = 'bad'; const handler = (event: string) => event.length;</script><template><!-- {token} --><Child :count=\"wrong\" @click=\"handler\" /></template>"
        );
        let result = transform(&source);
        let rows = owner_rows(&result);
        let generated = result.text.as_str();
        let targets = [
            "const __vize_prop_check_0_count",
            "\"count\": wrong",
            "const __vize_prop_check_0_onClick",
            "\"onClick\": handler",
        ]
        .map(|needle| generated.find(needle).unwrap());

        for target in targets {
            let owners = rows
                .iter()
                .filter(|row| {
                    let [_, _, start, end, _, _] = row.0;
                    start <= target && target < end
                })
                .collect::<Vec<_>>();
            assert!(!owners.is_empty(), "{token} must own {target}: {rows:?}");
            assert!(
                owners.iter().all(|row| {
                    let [start, length, _, _, _, _] = row.0;
                    &source[start..start + length] == token
                }),
                "synthetic ownership must not overlap {token}: {owners:?}"
            );
        }
    }
}
