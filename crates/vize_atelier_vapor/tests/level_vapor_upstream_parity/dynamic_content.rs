//! Computed slot content keeps names reactive and parameters in slot scope.

use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

use super::trace::trace;

fn element(tag: &str, id: &str, children: Value) -> Value {
    json!({"tag": tag, "attributes": {"data-id": id}, "children": children})
}

fn view(first: Value, second: Value, ids: Value) -> Value {
    json!({
        "tree": [element("main", "root", json!([
            element("section", "child", json!([first, second])),
            element("i", "tail", json!(["tail"])),
        ]))],
        "events": [], "identities": ids,
    })
}

#[test]
fn names_switch_scoped_content_and_fallbacks_like_official_vapor() {
    let content = |text: &str| element("b", "content", json!([text]));
    let one = || element("em", "one-fallback", json!(["one"]));
    let two = || element("em", "two-fallback", json!(["two"]));
    let expected = vec![
        view(
            content("A:P0"),
            two(),
            json!([
                ["root", 0],
                ["child", 1],
                ["content", 2],
                ["two-fallback", 3],
                ["tail", 4]
            ]),
        ),
        view(
            content("B:P1"),
            two(),
            json!([
                ["root", 0],
                ["child", 1],
                ["content", 2],
                ["two-fallback", 3],
                ["tail", 4]
            ]),
        ),
        view(
            one(),
            content("B:P1"),
            json!([
                ["root", 0],
                ["child", 1],
                ["one-fallback", 5],
                ["content", 6],
                ["tail", 4]
            ]),
        ),
        view(
            one(),
            two(),
            json!([
                ["root", 0],
                ["child", 1],
                ["one-fallback", 5],
                ["two-fallback", 7],
                ["tail", 4]
            ]),
        ),
        view(
            content("C:P2"),
            two(),
            json!([
                ["root", 0],
                ["child", 1],
                ["content", 8],
                ["two-fallback", 7],
                ["tail", 4]
            ]),
        ),
        json!({"tree":[],"events":[],"identities":[]}),
    ];
    let child = r#"<section data-id="child"><slot name="one" :value="value"><em data-id="one-fallback">one</em></slot><slot name="two" :value="value"><em data-id="two-fallback">two</em></slot></section>"#;
    let context = json!({"selected":"one", "label":"A", "childValue":"P0", "names":{"one":"one", "two":"two", "absent":"absent"}});
    let steps = json!([
        {"patch":{"label":"B", "childValue":"P1"}},
        {"patch":{"selected":"two"}},
        {"patch":{"selected":"absent"}},
        {"patch":{"selected":"one", "label":"C", "childValue":"P2"}},
    ]);
    for (case, name, own) in [
        ("template_reference", "selected", false),
        ("template_member", "names[selected]", false),
        ("template_call", "selected.toLowerCase()", false),
        ("component_reference", "selected", true),
        ("component_member", "names[selected]", true),
    ] {
        let slot_body = r#"<b data-id="content">{{ label }}:{{ value }}</b>"#;
        let component = if own {
            format!(
                r#"<MyComp :value="childValue" v-slot:[{name}]="{{ value }}">{slot_body}</MyComp>"#
            )
        } else {
            format!(
                r#"<MyComp :value="childValue"><template #[{name}]="{{ value }}">{slot_body}</template></MyComp>"#
            )
        };
        let source =
            format!(r#"<main data-id="root">{component}<i data-id="tail">tail</i></main>"#);
        let allocator = Allocator::new();
        let before = WalkCounts::snapshot();
        let options = VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let compiled = compile_vapor(&allocator, &source, options.clone());
        let compiled_child = compile_vapor(&allocator, child, options);
        assert_eq!(compiled.error_messages.len(), 0, "{case}");
        assert_eq!(compiled_child.error_messages.len(), 0, "{case} child");
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            0,
            "{case}: native L3"
        );
        insta::assert_snapshot!(format!("dynamic_content_{case}"), compiled.code);
        // A dependency change that computes the same name must preserve the
        // supplied slot's node, not rebuild it through a new function identity.
        let steps = if name == "selected.toLowerCase()" {
            let mut steps = steps.as_array().unwrap().clone();
            steps.insert(0, json!({"patch":{"selected":"ONE"}}));
            Value::Array(steps)
        } else {
            steps.clone()
        };
        let mut expected = expected.clone();
        if name == "selected.toLowerCase()" {
            expected.insert(1, expected[0].clone());
        }
        let vize = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend":"vapor", "code":compiled.code, "context":context, "steps":steps, "identities":true,
                "components":{"MyComp":{"code":compiled_child.code, "props":["value"]}},
            }),
        );
        let upstream = trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({
                "source":source, "context":context, "steps":steps,
                "components":{"MyComp":{"source":child, "props":["value"]}},
            }),
        );
        assert_eq!(vize, expected, "{case}: Vize native L3");
        assert_eq!(upstream, expected, "{case}: official Vapor");
    }
}
