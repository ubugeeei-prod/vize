//! Models on dynamically selected components retain assignment scope and
//! props while the selected child switches, mounts, and unmounts.

use super::trace::trace;
use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn view(child: &str, label: &str, value: &str, child_id: u32) -> Value {
    json!({
        "tree":[{"tag":"main","attributes":{"data-id":"root"},"children":[
            {"tag":"button","disabled":false,"attributes":{"data-id":child},"children":[format!("{label}:{value}")]},
            {"tag":"i","attributes":{"data-id":"mirror"},"children":[value]},
        ]}],
        "events":[], "identities":[["root",0],[child,child_id],["mirror",2]],
    })
}

#[test]
fn component_switches_preserve_model_updates_like_official_vapor() {
    for (case, model, prop, expression) in [
        ("default", "v-model", "modelValue", "value"),
        ("named", "v-model:title.trim", "title", "form.title"),
        ("indexed", "v-model", "modelValue", "items[index]"),
        (
            "computed_named",
            "v-model:title.trim",
            "title",
            "form[field]",
        ),
    ] {
        let source = format!(
            r#"<main data-id="root"><component :is="view" {model}="{expression}" /><i data-id="mirror">{{{{ {expression} }}}}</i></main>"#
        );
        let event = format!("update:{prop}");
        let props = if prop == "title" {
            vec![prop, "titleModifiers"]
        } else {
            vec![prop]
        };
        let update = |suffix| {
            if prop == "title" {
                format!("' ' + {prop} + '{suffix} '")
            } else {
                format!("{prop} + '{suffix}'")
            }
        };
        let first_update = update("!");
        let second_update = update("?");
        let first = format!(
            r#"<button data-id="first" @click="send('{event}', {first_update})">First:{{{{ {prop} }}}}</button>"#
        );
        let second = format!(
            r#"<button data-id="second" @click="send('{event}', {second_update})">Second:{{{{ {prop} }}}}</button>"#
        );
        let expected = vec![
            view("first", "First", "A", 1),
            view("first", "First", "A!", 1),
            view("second", "Second", "A!", 3),
            view("second", "Second", "A!?", 3),
            view("second", "Second", "P", 3),
            view("first", "First", "P", 4),
            view("first", "First", "P!", 4),
            json!({"tree":[],"events":[],"identities":[]}),
        ];
        let context = json!({"view":"First", "value":"A", "form":{"title":"A"}, "items":["A"], "index":0, "field":"title"});
        let title = if expression == "form.title" {
            "P"
        } else {
            "untouched"
        };
        let steps = json!([
            {"click":"first"},
            {"patch":{"view":"Second"}},
            {"click":"second"},
            {"patch":{"value":"P", "form":{"title":title,"alternate":"P"},"field":"alternate", "items":["untouched","P"],"index":1}},
            {"patch":{"view":"First"}},
            {"click":"first"},
        ]);
        let allocator = Allocator::new();
        let options = VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let walks = WalkCounts::snapshot();
        let compiled = compile_vapor(&allocator, &source, options.clone());
        let first_code = compile_vapor(&allocator, &first, options.clone());
        let second_code = compile_vapor(&allocator, &second, options);
        for result in [&compiled, &first_code, &second_code] {
            assert_eq!(result.error_messages.len(), 0, "{case}");
        }
        assert_eq!(
            WalkCounts::snapshot().since(walks).total_walks(),
            0,
            "{case}: native L3"
        );
        insta::assert_snapshot!(format!("dynamic_component_model_{case}"), compiled.code);
        let vize = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend":"vapor", "code":compiled.code, "context":context, "steps":steps, "identities":true,
                "components":{
                    "First":{"code":first_code.code,"props":props,"emits":[event]},
                    "Second":{"code":second_code.code,"props":props,"emits":[event]},
                },
            }),
        );
        let upstream = trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({
                "source":source,"context":context,"steps":steps,
                "components":{
                    "First":{"source":first,"props":props,"emits":[event]},
                    "Second":{"source":second,"props":props,"emits":[event]},
                },
            }),
        );
        assert_eq!(vize, expected, "{case}: Vize native L3");
        assert_eq!(upstream, expected, "{case}: official Vapor");
    }
}

#[test]
fn computed_native_model_targets_follow_key_changes_like_official_vapor() {
    for tag in ["input", "textarea"] {
        let close = if tag == "input" { "" } else { "</textarea>" };
        let source = format!(
            r#"<main data-id="root"><{tag} data-id="field" v-model="form[key]">{close}<i data-id="mirror">{{{{ form[key] }}}}</i><b data-id="old">{{{{ form.first }}}}</b></main>"#
        );
        let view = |value: &str, first: &str| {
            let mut field =
                json!({"tag":tag,"attributes":{"data-id":"field"},"children":[],"value":value});
            if tag == "input" {
                field["checked"] = json!(false);
            }
            json!({
                "tree":[{"tag":"main","attributes":{"data-id":"root"},"children":[
                    field,
                    {"tag":"i","attributes":{"data-id":"mirror"},"children":[value]},
                    {"tag":"b","attributes":{"data-id":"old"},"children":[first]},
                ]}],
                "events":[], "identities":[["root",0],["field",1],["mirror",2],["old",3]],
            })
        };
        super::trace::assert_native_upstream_trace(
            &source,
            json!({"key":"first","form":{"first":"A","second":"B"}}),
            json!([
                {"event":"input","selector":tag,"value":"typed"},
                {"patch":{"key":"second"}},
                {"event":"input","selector":tag,"value":"other"},
                {"patch":{"form":{"first":"external","second":"P"}}},
                {"patch":{"key":"first"}},
            ]),
            vec![
                view("A", "A"),
                view("typed", "typed"),
                view("B", "typed"),
                view("other", "typed"),
                view("P", "external"),
                view("external", "external"),
                json!({"tree":[],"events":[],"identities":[]}),
            ],
        );
    }
}

#[test]
fn l3_component_models_match_official_vapor_updates() {
    for (model, prop) in [("v-model", "modelValue"), ("v-model:title.trim", "title")] {
        let source = format!(
            r#"<main data-id="root"><MyComp {model}="value" /><span data-id="mirror">{{{{ value }}}}</span></main>"#
        );
        let event = format!("update:{prop}");
        let child = format!(
            r#"<button data-id="child" @click="send('{event}', {prop} + '!')">{{{{ {prop} }}}}</button>"#
        );
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
        assert!(
            compiled.error_messages.is_empty(),
            "{source}: {:?}",
            compiled.error_messages
        );
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            0,
            "{source}: parent must exercise native L3"
        );
        let child_code = compile_vapor(
            &allocator,
            &child,
            VaporCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(
            child_code.error_messages.is_empty(),
            "{child}: {:?}",
            child_code.error_messages
        );
        let context = json!({"value": "A"});
        let steps = json!([{"click": "child"}, {"patch": {"value": "P"}}]);
        let vize = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend": "vapor",
                "code": compiled.code,
                "context": context,
                "steps": steps,
                "identities": true,
                "components": {"MyComp": {"code": child_code.code, "props": [prop], "emits": [event]}},
            }),
        );
        let upstream = trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({
                "source": source,
                "context": context,
                "steps": steps,
                "components": {"MyComp": {"source": child, "props": [prop], "emits": [event]}},
            }),
        );
        assert_eq!(vize, upstream, "{model}: mounted component model parity");
        assert_eq!(vize.len(), 4);
        for (snapshot, expected) in vize.iter().take(3).zip(["A", "A!", "P"]) {
            let children = snapshot["tree"][0]["children"].as_array().unwrap();
            assert_eq!(children[0]["children"][0], expected, "{model}: child prop");
            assert_eq!(
                children[1]["children"][0], expected,
                "{model}: parent assignment"
            );
        }
        assert_eq!(vize[3]["tree"], json!([]), "{model}: unmount");
    }
}
