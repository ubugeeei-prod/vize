//! Computed props and handlers update in place and restore earlier sources.

use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

use super::trace::trace;

fn view(value: &str, events: Value) -> Value {
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [
            {"tag": "section", "attributes": {"data-id": "child"}, "children": [
                {"tag": "button", "attributes": {"data-id": "first"}, "children": ["first"], "disabled": false},
                {"tag": "button", "attributes": {"data-id": "second"}, "children": ["second"], "disabled": false},
                {"tag": "b", "attributes": {"data-id": "value"}, "children": [value]},
            ]},
            {"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]},
        ]}],
        "events": events,
        "identities": [["root", 0], ["child", 1], ["first", 2], ["second", 3], ["value", 4], ["tail", 5]],
    })
}

#[test]
fn computed_names_remove_old_handlers_and_restore_static_props() {
    let child = r#"<section data-id="child"><button data-id="first" @click="send('first', primary)">first</button><button data-id="second" @click="send('second', alternate)">second</button><b data-id="value">{{ primary }}|{{ alternate }}</b></section>"#;
    for (component, props, events) in [
        ("Child", "propName", "eventName"),
        ("Child", "keys[index]", "events[index]"),
        ("component :is=\"view\"", "is", "eventName"),
    ] {
        let source = format!(
            r#"<main data-id="root"><{component} alternate="fixed" :[{props}]="value" @[{events}]="record" /><i data-id="tail">tail</i></main>"#
        );
        let context = json!({"view": "Child", "propName": "primary", "is": "primary", "eventName": "first", "keys": ["primary", "alternate"], "events": ["first", "second"], "index": 0, "value": "A"});
        let steps = json!([
            {"click": "first"},
            {"patch": {"propName": "alternate", "is": "alternate", "eventName": "second", "index": 1, "value": "B"}},
            {"click": "first"}, {"click": "second"},
            {"patch": {"value": "C"}}, {"click": "second"},
            {"patch": {"propName": "primary", "is": "primary", "eventName": "first", "index": 0, "value": "D"}},
            {"click": "second"}, {"click": "first"},
        ]);
        let expected = vec![
            view("A|fixed", json!([])),
            view("A|fixed", json!(["A"])),
            view("|B", json!(["A"])),
            view("|B", json!(["A"])),
            view("|B", json!(["A", "B"])),
            view("|C", json!(["A", "B"])),
            view("|C", json!(["A", "B", "C"])),
            view("D|fixed", json!(["A", "B", "C"])),
            view("D|fixed", json!(["A", "B", "C"])),
            view("D|fixed", json!(["A", "B", "C", "D"])),
            json!({"tree": [], "events": ["A", "B", "C", "D"], "identities": []}),
        ];
        let allocator = Allocator::new();
        let options = VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let before = WalkCounts::snapshot();
        let parent = compile_vapor(&allocator, &source, options.clone());
        let child_code = compile_vapor(&allocator, child, options);
        for result in [&parent, &child_code] {
            assert!(
                result.error_messages.is_empty(),
                "{source}: {:?}",
                result.error_messages
            );
        }
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            0,
            "{source}: native S3"
        );
        let vize = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend": "vapor", "code": parent.code, "context": context, "steps": steps, "identities": true,
                "components": {"Child": {"code": child_code.code, "props": ["primary", "alternate"], "emits": ["first", "second"]}},
            }),
        );
        let upstream = trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({
                "source": source, "context": context, "steps": steps,
                "components": {"Child": {"source": child, "props": ["primary", "alternate"], "emits": ["first", "second"]}},
            }),
        );
        assert_eq!(vize, expected, "{source}: native trace");
        assert_eq!(upstream, expected, "{source}: official trace");
    }
}
