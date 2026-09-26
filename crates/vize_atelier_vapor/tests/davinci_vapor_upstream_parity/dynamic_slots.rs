//! Computed outlet names switch supplied slots and fallback lifetimes through
//! the native L3 path and the pinned official compiler under the same runtime.

use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

use super::trace::trace;

fn view(content: Value, tail: &str, identities: Value) -> Value {
    json!({
        "tree": [{
            "tag": "main",
            "attributes": {"data-id": "root"},
            "children": [content, {
                "tag": "i", "attributes": {"data-id": "tail"}, "children": [tail],
            }],
        }],
        "events": [],
        "identities": identities,
    })
}

fn fallback(text: &str) -> Value {
    json!({"tag": "b", "attributes": {"data-id": "fallback"}, "children": [text]})
}

#[test]
fn computed_names_match_official_slot_updates_and_fallback_lifetimes() {
    let expected = vec![
        view(json!("A"), "T0", json!([["root", 0], ["tail", 1]])),
        view(json!("B"), "T1", json!([["root", 0], ["tail", 1]])),
        view(json!("second"), "T1", json!([["root", 0], ["tail", 1]])),
        view(
            fallback("F1"),
            "T1",
            json!([["root", 0], ["fallback", 2], ["tail", 1]]),
        ),
        view(
            fallback("F2"),
            "T1",
            json!([["root", 0], ["fallback", 2], ["tail", 1]]),
        ),
        view(json!("C"), "T1", json!([["root", 0], ["tail", 1]])),
        view(
            fallback("F2"),
            "T1",
            json!([["root", 0], ["fallback", 3], ["tail", 1]]),
        ),
        json!({"tree": [], "events": [], "identities": []}),
    ];
    let context = json!({
        "selected": "one", "value": "A", "fallback": "F0", "tail": "T0",
        "names": {"one": "slot-one", "two": "slot-two", "absent": "slot-absent"},
    });
    let steps = json!([
        {"patch": {"value": "B", "tail": "T1"}},
        {"patch": {"selected": "two"}},
        {"patch": {"selected": "absent", "fallback": "F1"}},
        {"patch": {"fallback": "F2"}},
        {"patch": {"selected": "one", "value": "C"}},
        {"patch": {"selected": "absent"}},
    ]);
    for (case, name, first, second) in [
        ("reference", "selected", "one", "two"),
        ("member", "names[selected]", "slot-one", "slot-two"),
        ("compound", "'slot-' + selected", "slot-one", "slot-two"),
        ("call", "selected.toLowerCase()", "one", "two"),
    ] {
        let source = r#"<main data-id="root"><slot :name="NAME" :value="value"><b data-id="fallback">{{ fallback }}</b></slot><i data-id="tail">{{ tail }}</i></main>"#.replace("NAME", name);
        let allocator = Allocator::new();
        let before = WalkCounts::snapshot();
        let compiled = compile_vapor(
            &allocator,
            &source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert_eq!(compiled.error_messages.len(), 0, "{case}");
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            0,
            "{case}: native L3"
        );
        insta::assert_snapshot!(format!("dynamic_outlet_{case}"), compiled.code);
        let mut slots = serde_json::Map::new();
        slots.insert(first.to_owned(), json!({"prop": "value"}));
        slots.insert(second.to_owned(), json!({"text": "second"}));
        let slots = Value::Object(slots);
        let vize = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend": "vapor", "code": compiled.code, "context": context,
                "steps": steps, "identities": true, "slots": slots,
            }),
        );
        let upstream = trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({
                "source": source, "context": context, "steps": steps, "slots": slots,
            }),
        );
        assert_eq!(vize, expected, "{case}: Vize L3");
        assert_eq!(upstream, expected, "{case}: official Vapor");
    }
}
