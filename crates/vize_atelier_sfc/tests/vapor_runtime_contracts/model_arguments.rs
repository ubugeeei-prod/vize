use super::super::{Value, json, trace};
use super::models::{CHILD, child_props, outputs, source};

#[test]
fn compound_model_arguments_keep_derived_keys_and_listener_updates() {
    for (argument, first, second) in [
        (
            "state.field||'first'",
            json!({"field": ""}),
            json!({"field": "second"}),
        ),
        (
            "state.field??'first'",
            json!({"field": null}),
            json!({"field": "second"}),
        ),
        (
            "state.ready&&'second'||'first'",
            json!({"ready": false}),
            json!({"ready": true}),
        ),
        (
            "state.other,state.field",
            json!({"field": "first"}),
            json!({"field": "second"}),
        ),
        (
            "(state.other,state.field)",
            json!({"field": "first"}),
            json!({"field": "second"}),
        ),
    ] {
        let source = source(&format!("v-model:[{argument}].trim=\"state.value\""));
        let mut context = json!({"value": "initial", "other": "untouched"});
        context
            .as_object_mut()
            .unwrap()
            .extend(first.as_object().unwrap().clone());
        let extra = json!({"childSource": CHILD, "context": context,
        "steps": [
            {"click": ".first"},
            {"patch": second, "preserve": [".first", ".second"]},
            {"click": ".first"},
            {"click": ".second"},
            {"patch": first, "preserve": [".first", ".second"]},
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
                "first|untouched",
                "second|untouched",
                "second|untouched",
                "second|untouched",
                "first|untouched"
            ],
            "{argument}"
        );
        for (index, key, value) in [
            (0, "first", "initial"),
            (2, "second", "first"),
            (5, "first", "second"),
        ] {
            let modifiers = format!("{key}Modifiers");
            let expected: Value = json!({key: value, modifiers: {"trim": true}});
            assert_eq!(child_props(&dom, index), expected, "{argument}");
        }
        assert_eq!(trace(&source, "vapor", extra), dom, "{argument}");
    }
}
