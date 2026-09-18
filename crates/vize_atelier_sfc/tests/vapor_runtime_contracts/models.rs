use super::super::{Value, compile, json, trace};

const CHILD: &str = r#"<script setup>
const props = defineProps(['first', 'second', 'modelValue', 'fixed', 'firstModifiers', 'secondModifiers', 'modelModifiers', 'modelValueModifiers', 'fixedModifiers']);
const emit = defineEmits(['update:first', 'update:second', 'update:modelValue', 'update:fixed']);
const snapshot = () => JSON.stringify({ first: props.first, second: props.second, modelValue: props.modelValue, fixed: props.fixed, firstModifiers: props.firstModifiers, secondModifiers: props.secondModifiers, modelModifiers: props.modelModifiers, modelValueModifiers: props.modelValueModifiers, fixedModifiers: props.fixedModifiers });
</script><template><div><pre>{{ snapshot() }}</pre><button class="first" @click="emit('update:first', ' first ')">first</button><button class="second" @click="emit('update:second', ' second ')">second</button><button class="default" @click="emit('update:modelValue', ' default ')">default</button><button class="fixed" @click="emit('update:fixed', ' 42 ')">fixed</button></div></template>"#;

fn source(model: &str) -> String {
    format!(
        "<script setup>import Child from './Child.vue'; import {{ state }} from './fixture';</script><template><section><Child {model}/><output>{{{{ state.value }}}}|{{{{ state.other }}}}</output></section></template>"
    )
}

fn outputs(trace: &Value) -> Vec<&str> {
    trace
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|snapshot| snapshot["tree"][0]["children"][1]["children"][0].as_str())
        .collect()
}

fn child_props(trace: &Value, index: usize) -> Value {
    serde_json::from_str(
        trace[index]["tree"][0]["children"][0]["children"][0]["children"][0]
            .as_str()
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn dynamic_model_keys_values_listeners_and_modifiers_follow_argument_changes() {
    // Use the equivalent field oracle: VDOM currently omits the parentheses
    // needed around ternary arguments when composing derived keys (#6216).
    let reference = source("v-model:[state.field].trim=\"state.value\"");
    for argument in [
        "state.field",
        "state.field.toString()",
        "state.index?'second':'first'",
    ] {
        let source = source(&format!("v-model:[{argument}].trim=\"state.value\""));
        let extra = json!({"childSource": CHILD,
        "context": {"field": "first", "index": 0, "value": "initial", "other": "untouched"},
        "steps": [
            {"click": ".first"},
            {"patch": {"field": "second", "index": 1, "value": "external"}, "preserve": [".first", ".second"]},
            {"click": ".first"},
            {"click": ".second"},
            {"patch": {"field": "first", "index": 0}},
            {"click": ".second"},
            {"click": ".first"}
        ]});
        let dom = trace(&reference, "vdom", extra.clone());
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
                "first|untouched"
            ],
            "{argument}: {}",
            compile(&source, "vdom")
        );
        assert_eq!(
            child_props(&dom, 0),
            json!({"first": "initial", "firstModifiers": {"trim": true}})
        );
        assert_eq!(
            child_props(&dom, 2),
            json!({"second": "external", "secondModifiers": {"trim": true}})
        );
        assert_eq!(trace(&source, "vapor", extra), dom, "{argument}");
    }
}

#[test]
fn default_named_and_multiple_models_keep_distinct_modifier_contracts() {
    for model in [
        "v-model.trim=\"state.value\"",
        "v-model:modelValue.trim=\"state.value\"",
        "v-model:[state.field].trim=\"state.value\"",
    ] {
        let source = source(&format!("{model} v-model:fixed.number=\"state.other\""));
        let extra = json!({"childSource": CHILD,
            "context": {"field": "modelValue", "value": "initial", "other": "0"},
            "steps": [{"click": ".default"}, {"click": ".fixed"}, {"patch": {"value": "external", "other": 7}}]});
        let dom = trace(&source, "vdom", extra.clone());
        assert_eq!(
            outputs(&dom),
            ["initial|0", "default|0", "default|42", "external|7"]
        );
        let key = if model.contains("[state.field]") {
            "modelValueModifiers"
        } else {
            "modelModifiers"
        };
        assert_eq!(child_props(&dom, 0)[key], json!({"trim": true}));
        assert_eq!(
            child_props(&dom, 0)["fixedModifiers"],
            json!({"number": true})
        );
        assert_eq!(trace(&source, "vapor", extra), dom, "{model}");
    }
}

#[test]
fn model_assignments_resolve_loop_scope_and_computed_targets() {
    let source = r#"<script setup>import Child from './Child.vue'; import { state } from './fixture';</script><template><section><Child v-for="item in state.items" :key="item.id" :data-id="item.id" v-model:[state.field].trim="item[state.target]"/><output>{{ JSON.stringify(state.items) }}</output></section></template>"#;
    let extra = json!({"childSource": CHILD,
    "context": {"field": "first", "target": "value", "items": [{"id": "a", "value": "A"}, {"id": "b", "value": "B"}]},
    "steps": [
        {"click": "[data-id=b] .first"},
        {"patch": {"field": "second", "target": "next", "items": [{"id": "b", "value": "first", "next": "B2"}, {"id": "a", "value": "A", "next": "A2"}]}, "preserve": ["[data-id=a]", "[data-id=b]"]},
        {"click": "[data-id=a] .second"},
        {"click": "[data-id=b] .first"}
    ]});
    let dom = trace(source, "vdom", extra.clone());
    let final_items = dom[4]["tree"][0]["children"][2]["children"][0]
        .as_str()
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(final_items).unwrap(),
        json!([{"id":"b", "value":"first", "next":"B2"}, {"id":"a", "value":"A", "next":"second"}])
    );
    assert_eq!(trace(source, "vapor", extra), dom);
}

#[test]
fn custom_modifiers_are_literal_keys_and_unmodified_models_stay_unmodified() {
    for modifiers in ["", ".trim.foo-bar"] {
        let source = source(&format!("v-model:[state.field]{modifiers}=\"state.value\""));
        let extra = json!({"childSource": CHILD,
            "context": {"field": "first", "value": "initial", "other": ""},
            "steps": [{"click": ".first"}, {"patch": {"field": "second"}}, {"click": ".second"}]});
        let actual = trace(&source, "vapor", extra.clone());
        if modifiers.is_empty() {
            assert_eq!(
                outputs(&actual),
                ["initial|", " first |", " first |", " second |"]
            );
            assert_eq!(child_props(&actual, 0), json!({"first": "initial"}));
            assert_eq!(actual, trace(&source, "vdom", extra));
        } else {
            assert_eq!(
                outputs(&actual),
                ["initial|", "first|", "first|", "second|"]
            );
            assert_eq!(
                child_props(&actual, 2),
                json!({"second": "first", "secondModifiers": {"trim": true, "foo-bar": true}})
            );
        }
    }
}

#[test]
fn dynamic_bound_keys_recompute_without_reordering_spread_precedence() {
    for bindings in [
        "v-bind=\"state.spread\" :[state.field]=\"state.value\" fixed=\"tail\"",
        "fixed=\"head\" :[state.field]=\"state.value\" v-bind=\"state.spread\"",
    ] {
        let source = source(bindings);
        let extra = json!({"childSource": CHILD,
            "context": {"field": "first", "value": "initial", "other": "", "spread": {"first": "spread", "fixed": "spread-fixed"}},
            "steps": [{"patch": {"field": "second", "value": "updated"}}, {"patch": {"spread": {"second": "spread-2"}}}]});
        let dom = trace(&source, "vdom", extra.clone());
        assert_eq!(trace(&source, "vapor", extra), dom, "{bindings}");
    }
}
