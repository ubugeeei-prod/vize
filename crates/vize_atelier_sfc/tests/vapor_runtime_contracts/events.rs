use super::super::{json, trace};
use super::models::{CHILD, child_props, outputs, source};

#[test]
fn nested_component_prop_and_event_arguments_share_the_selected_key() {
    for (bind, on) in [(":", "@"), ("v-bind:", "v-on:")] {
        let source = source(&format!(
            r#"{bind}[state.keys['names]'][state.indices[state.index]]]="state.value" {on}[state.keys['events]'][state.indices[state.index]]]="state.value=$event.trim()""#
        ));
        let extra = json!({"childSource": CHILD,
        "context": {"keys": {"names]": ["first", "second"], "events]": ["update:first", "update:second"]},
            "indices": [0, 1], "index": 0, "value": "initial", "other": "untouched"},
        "steps": [
            {"click": ".first"},
            {"patch": {"index": 1, "value": "external"}, "preserve": [".first", ".second"]},
            {"click": ".first"},
            {"click": ".second"},
            {"patch": {"index": 0}},
            {"click": ".second"},
            {"click": ".first"},
            {"patch": {"keys": {"names]": ["second", "first"], "events]": ["update:second", "update:first"]}}, "preserve": [".first", ".second"]},
            {"click": ".first"},
            {"click": ".second"}
        ]});
        let dom = trace(&source, "vdom", extra.clone());
        assert_eq!(
            outputs(&dom),
            [
                "initial|untouched",
                "first|untouched",
                "external|untouched",
                "external|untouched",
                "second|untouched",
                "second|untouched",
                "second|untouched",
                "first|untouched",
                "first|untouched",
                "first|untouched",
                "second|untouched"
            ]
        );
        assert_eq!(child_props(&dom, 0), json!({"first": "initial"}));
        assert_eq!(child_props(&dom, 2), json!({"second": "external"}));
        assert_eq!(child_props(&dom, 8), json!({"second": "first"}));
        assert_eq!(trace(&source, "vapor", extra), dom, "{bind} / {on}");
    }
}

#[test]
fn conditional_event_arguments_use_the_condition_not_an_unrelated_event_field() {
    let source =
        source(r#"@[state.index?'update:second':'update:first']="state.value=$event.trim()""#);
    let extra = json!({"childSource": CHILD,
    "context": {"index": 0, "event": "update:second", "value": "initial", "other": "untouched"},
    "steps": [
        {"click": ".first"},
        {"patch": {"event": "update:first"}},
        {"patch": {"index": 1, "value": "external"}, "preserve": [".first", ".second"]},
        {"click": ".first"},
        {"click": ".second"},
        {"patch": {"event": "update:second"}},
        {"patch": {"index": 0}},
        {"click": ".second"},
        {"click": ".first"}
    ]});
    let dom = trace(&source, "vdom", extra.clone());
    assert_eq!(
        outputs(&dom),
        [
            "initial|untouched",
            "first|untouched",
            "first|untouched",
            "external|untouched",
            "external|untouched",
            "second|untouched",
            "second|untouched",
            "second|untouched",
            "second|untouched",
            "first|untouched"
        ]
    );
    assert_eq!(trace(&source, "vapor", extra), dom);
}

#[test]
fn component_event_arguments_change_reactively_without_stale_listeners() {
    for (argument, prefix) in [("state.event", ""), ("state.event.slice(1)", "_")] {
        for handler in [
            "state.value=$event.trim()",
            "value=>{state.value=value.trim()}",
        ] {
            let source = source(&format!("@[{argument}]=\"{handler}\""));
            let extra = json!({"childSource": CHILD,
            "context": {"event": format!("{prefix}update:first"), "value": "initial", "other": "untouched"},
            "steps": [
                {"click": ".first"},
                {"patch": {"event": format!("{prefix}update:second"), "value": "external"}, "preserve": [".first", ".second"]},
                {"click": ".first"},
                {"click": ".second"},
                {"patch": {"event": prefix}},
                {"click": ".second"},
                {"patch": {"event": format!("{prefix}update:first")}},
                {"click": ".first"}
            ]});
            let dom = trace(&source, "vdom", extra.clone());
            assert_eq!(
                outputs(&dom),
                [
                    "initial|untouched",
                    "first|untouched",
                    "external|untouched",
                    "external|untouched",
                    "second|untouched",
                    "second|untouched",
                    "second|untouched",
                    "second|untouched",
                    "first|untouched"
                ]
            );
            assert_eq!(trace(&source, "vapor", extra), dom, "{argument}: {handler}");
        }
    }
}

#[test]
fn component_event_arguments_and_handlers_keep_loop_scope_after_reordering() {
    let source = r#"<script setup>import Child from './Child.vue'; import { state } from './fixture';</script><template><section><Child v-for="item in state.items" :key="item.id" :data-id="item.id" @[item.event]="item.value=$event.trim()"/><output>{{ JSON.stringify(state.items) }}</output></section></template>"#;
    let extra = json!({"childSource": CHILD,
    "context": {"items": [{"id": "a", "event": "update:first", "value": "A"}, {"id": "b", "event": "update:second", "value": "B"}]},
    "steps": [
        {"click": "[data-id=a] .first"},
        {"click": "[data-id=b] .second"},
        {"patch": {"items": [{"id": "b", "event": "update:first", "value": "B2"}, {"id": "a", "event": "update:second", "value": "A2"}]}, "preserve": ["[data-id=a]", "[data-id=b]"]},
        {"click": "[data-id=b] .second"},
        {"click": "[data-id=a] .first"},
        {"click": "[data-id=b] .first"},
        {"click": "[data-id=a] .second"}
    ]});
    let dom = trace(source, "vdom", extra.clone());
    let final_items = dom[7]["tree"][0]["children"][2]["children"][0]
        .as_str()
        .unwrap();
    assert_eq!(
        serde_json::from_str::<super::super::Value>(final_items).unwrap(),
        json!([
            {"id": "b", "event": "update:first", "value": "first"},
            {"id": "a", "event": "update:second", "value": "second"}
        ])
    );
    assert_eq!(trace(source, "vapor", extra), dom);
}
