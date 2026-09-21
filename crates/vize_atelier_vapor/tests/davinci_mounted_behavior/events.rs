//! Native S3 events must preserve delivery, modifier composition and options.

use serde_json::json;

#[test]
fn native_event_modifiers_compose_key_filters_with_bubbling_guards() {
    let source =
        r#"<div @keydown="saveParent"><button @keydown.enter.stop="save">Key</button></div>"#;
    let trace = crate::assert_backends(
        source,
        json!({}),
        json!([
            {"event": "keydown", "selector": "button", "key": "Escape"},
            {"event": "keydown", "selector": "button", "key": "Enter"},
            {"event": "keydown", "selector": "button", "key": "Tab"}
        ]),
    );
    // Non-matching keys still bubble to the parent. Enter calls only the child.
    for (index, events) in [
        json!([]),
        json!(["saveParent"]),
        json!(["saveParent", "save"]),
        json!(["saveParent", "save", "saveParent"]),
        json!(["saveParent", "save", "saveParent"]),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(trace[index]["events"], events);
    }
}

#[test]
fn native_events_distinguish_keyboard_aliases_from_mouse_buttons() {
    for (directive, rejected, accepted) in [
        (
            "keydown.a",
            json!({"event": "keydown", "key": "b"}),
            json!({"event": "keydown", "key": "a"}),
        ),
        (
            "keydown.left",
            json!({"event": "keydown", "key": "ArrowRight"}),
            json!({"event": "keydown", "key": "ArrowLeft"}),
        ),
        (
            "click.right",
            json!({"event": "contextmenu", "button": 0}),
            json!({"event": "contextmenu", "button": 2}),
        ),
        (
            "click.middle",
            json!({"event": "mouseup", "button": 0}),
            json!({"event": "mouseup", "button": 1}),
        ),
    ] {
        let source = vize_carton::cstr!(r#"<button @{directive}="save">Action</button>"#);
        let steps = [rejected, accepted].map(|mut step| {
            step["selector"] = json!("button");
            step
        });
        let trace = crate::assert_backends(&source, json!({}), json!(steps));
        assert_eq!(trace[1]["events"], json!([]), "{directive}");
        assert_eq!(trace[2]["events"], json!(["save"]), "{directive}");
    }
}

#[test]
fn native_events_support_non_delegated_names_and_combined_options() {
    for (directive, event, bubbles, count) in [
        ("focus", "focus", false, 2),
        ("mouseenter", "mouseenter", false, 2),
        ("pointerenter", "pointerenter", false, 2),
        ("scroll", "scroll", false, 2),
        ("custom-event", "custom-event", true, 2),
        ("click.once.capture", "click", true, 1),
        ("click.once.capture.passive", "click", true, 1),
    ] {
        let source = vize_carton::cstr!(r#"<button @{directive}="save">Action</button>"#);
        let trace = crate::assert_backends(
            &source,
            json!({}),
            json!([
                {"event": event, "selector": "button", "bubbles": bubbles},
                {"event": event, "selector": "button", "bubbles": bubbles}
            ]),
        );
        assert_eq!(trace[0]["events"], json!([]), "{directive}");
        assert_eq!(trace[1]["events"], json!(["save"]), "{directive}");
        assert_eq!(
            trace[2]["events"],
            json!(vec!["save"; count]),
            "{directive}"
        );
        assert_eq!(trace[3]["tree"], json!([]), "{directive}");
    }
}
