//! #10894: drive rejected keys, cancellation, bubbling and repeated component emits.

use serde_json::json;

use super::runtime::{Child, mounted_trace_with_components};

const BACKENDS: [&str; 3] = ["vdom", "vapor", "vapor-legacy"];

#[test]
fn component_event_modifiers_cancel_submit_filter_keys_and_stop_propagation() {
    let child: Child<'_> = ("Box", "<form><slot /></form>", &[], &[]);
    for backend in BACKENDS {
        let trace = mounted_trace_with_components(
            backend,
            r#"<div @keydown="saveParent"><Box @submit.prevent="record($event.defaultPrevented)" @keydown.enter.stop.prevent="record($event.key)" /></div>"#,
            &[child],
            json!({}),
            json!([
                {"event": "submit", "selector": "form"},
                {"event": "keydown", "selector": "form", "key": "a"},
                {"event": "keydown", "selector": "form", "key": "Enter"},
                {"event": "keydown", "selector": "form", "key": "Escape"}
            ]),
            false,
        );
        for (index, expected) in [
            json!([]),
            json!([true]),
            json!([true, "saveParent"]),
            json!([true, "saveParent", "Enter"]),
            json!([true, "saveParent", "Enter", "saveParent"]),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(trace[index]["events"], expected, "{backend} step {index}");
        }
    }
}

#[test]
fn component_event_modifiers_deliver_emitted_and_fallthrough_once_listeners_once() {
    for (name, template, emits, event) in [
        (
            "Pinger",
            r#"<button @click="$emit('ping')">ping</button>"#,
            &["ping"][..],
            "ping",
        ),
        ("Box", "<button>click</button>", &[][..], "click"),
    ] {
        for backend in BACKENDS {
            let options = if name == "Box" {
                ".capture.passive"
            } else {
                ""
            };
            let source = format!(r#"<{name} @{event}.once{options}="save" />"#);
            let trace = mounted_trace_with_components(
                backend,
                &source,
                &[(name, template, &[], emits)],
                json!({}),
                json!([
                    {"event": "click", "selector": "button"},
                    {"event": "click", "selector": "button"}
                ]),
                false,
            );
            assert_eq!(trace[1]["events"], json!(["save"]), "{backend}: {name}");
            assert_eq!(trace[2]["events"], json!(["save"]), "{backend}: {name}");
        }
    }
}

#[test]
fn component_event_modifiers_rebind_dynamic_names_and_observe_cancellation() {
    for backend in BACKENDS {
        let trace = mounted_trace_with_components(
            backend,
            r#"<Box @[event].enter.prevent="record($event.defaultPrevented)" />"#,
            &[("Box", "<input />", &[], &[])],
            json!({"event": "keydown"}),
            json!([
                {"event": "keydown", "selector": "input", "key": "a", "observeEvent": true},
                {"event": "keydown", "selector": "input", "key": "Enter", "observeEvent": true},
                {"patch": {"event": "keyup"}},
                {"event": "keydown", "selector": "input", "key": "Enter", "observeEvent": true},
                {"event": "keyup", "selector": "input", "key": "Escape", "observeEvent": true},
                {"event": "keyup", "selector": "input", "key": "Enter", "observeEvent": true}
            ]),
            false,
        );
        let accepted = json!({"dispatched": false, "defaultPrevented": true});
        let rejected = json!({"dispatched": true, "defaultPrevented": false});
        assert_eq!(trace[1]["events"], json!([rejected]), "{backend}");
        assert_eq!(
            trace[2]["events"],
            json!([rejected, true, accepted]),
            "{backend}"
        );
        assert_eq!(
            trace[6]["events"],
            json!([rejected, true, accepted, rejected, rejected, true, accepted]),
            "{backend}"
        );
    }
}

#[test]
fn component_event_modifiers_cancel_default_actions_without_a_handler() {
    for backend in BACKENDS {
        let trace = mounted_trace_with_components(
            backend,
            r#"<Box @submit.prevent />"#,
            &[("Box", "<form></form>", &[], &[])],
            json!({}),
            json!([{"event": "submit", "selector": "form", "observeEvent": true}]),
            false,
        );
        assert_eq!(
            trace[1]["events"],
            json!([{"dispatched": false, "defaultPrevented": true}]),
            "{backend}"
        );
    }
}

#[test]
fn component_event_modifiers_keep_each_handler_on_a_shared_event_key() {
    for backend in BACKENDS {
        let trace = mounted_trace_with_components(
            backend,
            r#"<Box @keydown.enter.prevent="record('enter')" @keydown.esc.prevent="record('esc')" />"#,
            &[("Box", "<input />", &[], &[])],
            json!({}),
            json!([
                {"event": "keydown", "selector": "input", "key": "a"},
                {"event": "keydown", "selector": "input", "key": "Enter"},
                {"event": "keydown", "selector": "input", "key": "Escape"},
                {"event": "keydown", "selector": "input", "key": "Enter"}
            ]),
            false,
        );
        assert_eq!(trace[1]["events"], json!([]), "{backend}");
        assert_eq!(trace[2]["events"], json!(["enter"]), "{backend}");
        assert_eq!(trace[3]["events"], json!(["enter", "esc"]), "{backend}");
        assert_eq!(
            trace[4]["events"],
            json!(["enter", "esc", "enter"]),
            "{backend}"
        );
    }
}
