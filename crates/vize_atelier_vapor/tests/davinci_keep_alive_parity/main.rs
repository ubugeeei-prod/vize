//! Isolated native, retained and official KeepAlive runtime comparisons.
//! Retained compiles cannot change another binary's process-global walk probes.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "tests assert by panicking and compare std-string fixtures"
)]

//! Cache reactivation, eviction, filters, updates and event continuity.

use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

mod trace;

use trace::trace;

fn view(child: &str, label: &str, id: u32, events: Value) -> Value {
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [
            {"tag": "button", "attributes": {"data-id": child}, "children": [label], "disabled": false},
            {"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]},
        ]}],
        "events": events,
        "identities": [["root", 0], [child, id], ["tail", 2]],
    })
}

#[test]
fn cache_policy_reuses_or_evicts_the_same_components_as_official_vapor() {
    for (props, first_again, second_again) in [
        ("", 1, 3),
        ("max=\"1\"", 4, 5),
        ("include=\"First\"", 1, 4),
        ("exclude=\"First\"", 4, 3),
        (
            ":include=\"names\" :exclude=\"excluded\" :max=\"limit\"",
            1,
            4,
        ),
    ] {
        let source = format!(
            r#"<main data-id="root"><KeepAlive {props}><component :is="view" :label="label" @send="record" /></KeepAlive><i data-id="tail">tail</i></main>"#
        );
        let first = r#"<button data-id="first" @click="send('send', label)">{{ label }}</button>"#;
        let second =
            r#"<button data-id="second" @click="send('send', label)">{{ label }}</button>"#;
        let context =
            json!({"view": "First", "label": "A", "names": ["First"], "excluded": [], "limit": 2});
        let steps = json!([
            {"click": "first"}, {"patch": {"view": "Second", "label": "B"}},
            {"click": "second"}, {"patch": {"view": "First", "label": "C"}},
            {"click": "first"}, {"patch": {"view": "Second", "label": "D"}},
            {"click": "second"},
        ]);
        let expected = vec![
            view("first", "A", 1, json!([])),
            view("first", "A", 1, json!(["A"])),
            view("second", "B", 3, json!(["A"])),
            view("second", "B", 3, json!(["A", "B"])),
            view("first", "C", first_again, json!(["A", "B"])),
            view("first", "C", first_again, json!(["A", "B", "C"])),
            view("second", "D", second_again, json!(["A", "B", "C"])),
            view("second", "D", second_again, json!(["A", "B", "C", "D"])),
            json!({"tree": [], "events": ["A", "B", "C", "D"], "identities": []}),
        ];
        let allocator = Allocator::new();
        let options = VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let before = WalkCounts::snapshot();
        let parent = compile_vapor(&allocator, &source, options.clone());
        let first_code = compile_vapor(&allocator, first, options.clone());
        let second_code = compile_vapor(&allocator, second, options);
        for result in [&parent, &first_code, &second_code] {
            assert!(
                result.error_messages.is_empty(),
                "{props}: {:?}",
                result.error_messages
            );
        }
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            0,
            "{props}: native S3"
        );
        let vize = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend": "vapor", "code": parent.code, "context": context, "steps": steps, "identities": true,
                "components": {"First": {"code": first_code.code, "props": ["label"], "emits": ["send"]}, "Second": {"code": second_code.code, "props": ["label"], "emits": ["send"]}},
            }),
        );
        let retained = compile_vapor(
            &allocator,
            &source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                davinci_retained_lane: true,
                ..Default::default()
            },
        );
        assert!(
            retained.error_messages.is_empty(),
            "{props}: retained compile"
        );
        let retained_trace = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend": "vapor", "code": retained.code, "context": context, "steps": steps, "identities": true,
                "components": {"First": {"code": first_code.code, "props": ["label"], "emits": ["send"]}, "Second": {"code": second_code.code, "props": ["label"], "emits": ["send"]}},
            }),
        );
        let upstream = trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({
                "source": source, "context": context, "steps": steps,
                "components": {"First": {"source": first, "props": ["label"], "emits": ["send"]}, "Second": {"source": second, "props": ["label"], "emits": ["send"]}},
            }),
        );
        assert_eq!(vize, expected, "{props}: native trace");
        assert_eq!(retained_trace, expected, "{props}: retained trace");
        assert_eq!(upstream, expected, "{props}: official trace");
    }
}
