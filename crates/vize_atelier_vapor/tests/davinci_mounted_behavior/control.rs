use super::*;

#[test]
fn mounted_nested_event_paths_deliver_once_per_activation() {
    let sources = [
        (
            r#"<section><button v-for="item in items" :key="item" @click="save">{{ item }}</button></section>"#,
            json!({"items": ["first"]}),
            json!([{"activate": "button"}, {"patch": {"items": ["second"]}}, {"activate": "button"}]),
            ["first", "first", "second", "second"],
        ),
        (
            r#"<section><slot name="body"><button @click="save">fallback</button></slot></section>"#,
            json!({}),
            json!([{"activate": "button"}, {"patch": {}}, {"activate": "button"}]),
            ["fallback"; 4],
        ),
    ];
    for (source, context, steps, labels) in sources {
        let mut expected = vec![];
        for (label, events) in labels.into_iter().zip([
            json!([]),
            json!(["save"]),
            json!(["save"]),
            json!(["save", "save"]),
        ]) {
            expected.push(json!({
                "tree": [{
                    "tag": "section", "attributes": {},
                    "children": [{"tag": "button", "attributes": {}, "children": [label], "disabled": false}]
                }],
                "events": events
            }));
        }
        expected.push(json!({"tree": [], "events": ["save", "save"]}));
        assert_eq!(assert_backends(source, context, steps), json!(expected));
    }
}

#[test]
fn mounted_conditional_button_rebinds_after_remount_beside_slot_fallback() {
    let trace = assert_backends(
        r#"<section><button v-if="ready" :disabled="locked" @click="save">{{ label }}</button><slot name="body"><span v-text="fallback"></span></slot></section>"#,
        json!({"ready": true, "locked": true, "label": "Save", "fallback": "fallback"}),
        json!([
            {"activate": "button"},
            {"patch": {"locked": false}},
            {"activate": "button"},
            {"patch": {"ready": false}},
            {"patch": {"ready": true, "label": "Again"}},
            {"activate": "button"},
            {"patch": {"ready": false}}
        ]),
    );
    let view = |ready: bool, locked: bool, label: &str, events: Value| {
        let mut children = vec![];
        if ready {
            children.push(json!({
                "tag": "button",
                "attributes": if locked { json!({"disabled": ""}) } else { json!({}) },
                "children": [label],
                "disabled": locked
            }));
        }
        children.push(json!({"tag": "span", "attributes": {}, "children": ["fallback"]}));
        json!({"tree": [{"tag": "section", "attributes": {}, "children": children}], "events": events})
    };
    assert_eq!(
        trace,
        json!([
            view(true, true, "Save", json!([])),
            view(true, true, "Save", json!([])),
            view(true, false, "Save", json!([])),
            view(true, false, "Save", json!(["save"])),
            view(false, false, "", json!(["save"])),
            view(true, false, "Again", json!(["save"])),
            view(true, false, "Again", json!(["save", "save"])),
            view(false, false, "", json!(["save", "save"])),
            {"tree": [], "events": ["save", "save"]}
        ])
    );
}
