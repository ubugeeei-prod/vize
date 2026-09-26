//! Reactive model argument names update keys, listeners, and modifier props.

use super::trace::trace;
use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn view(title: &str, label: &str, modifier: &str, value: &str) -> Value {
    let label_text = if label.is_empty() {
        vec![]
    } else {
        vec![label]
    };
    json!({
        "tree":[{"tag":"main","attributes":{"data-id":"root"},"children":[
            {"tag":"section","attributes":{"data-id":"child"},"children":[
                {"tag":"button","disabled":false,"attributes":{"data-id":"title"},"children":[title]},
                {"tag":"button","disabled":false,"attributes":{"data-id":"label"},"children":label_text},
                {"tag":"i","attributes":{"data-id":"modifier"},"children":[modifier]},
            ]},
            {"tag":"b","attributes":{"data-id":"mirror"},"children":[value]},
        ]}],
        "events":[],"identities":[["root",0],["child",1],["title",2],["label",3],["modifier",4],["mirror",5]],
    })
}

#[test]
fn model_names_replace_old_listeners_and_restore_static_props_like_official_vapor() {
    let child = r#"<section data-id="child"><button data-id="title" @click="send('update:title', ' T! ')">{{ title }}</button><button data-id="label" @click="send('update:label', ' L! ')">{{ label }}</button><i data-id="modifier">{{ titleModifiers?.trim ? 'title' : labelModifiers?.trim ? 'label' : '-' }}</i></section>"#;
    let expected = vec![
        view("A", "", "title", "A"),
        view("T!", "", "title", "T!"),
        view("static", "T!", "label", "T!"),
        view("static", "T!", "label", "T!"),
        view("static", "L!", "label", "L!"),
        view("static", "external", "label", "external"),
        view("external", "", "title", "external"),
        view("external", "", "title", "external"),
        view("T!", "", "title", "T!"),
        json!({"tree":[],"events":[],"identities":[]}),
    ];
    for (case, name, owner) in [
        ("reference", "name", "Child"),
        ("indexed", "names[index]", "Child"),
        ("call", "name.toLowerCase()", "Child"),
        ("dynamic", "name", "component :is=\"view\""),
    ] {
        let source = format!(
            r#"<main data-id="root"><{owner} title="static" v-model:[{name}].trim="value" /><b data-id="mirror">{{{{ value }}}}</b></main>"#
        );
        let context =
            json!({"name":"title","names":["title","label"],"index":0,"view":"Child","value":"A"});
        let steps = json!([
            {"click":"title"},
            {"patch":{"name":"label","index":1}},
            {"click":"title"},
            {"click":"label"},
            {"patch":{"value":"external"}},
            {"patch":{"name":"title","index":0}},
            {"click":"label"},
            {"click":"title"},
        ]);
        let allocator = Allocator::new();
        let options = VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let before = WalkCounts::snapshot();
        let compiled = compile_vapor(&allocator, &source, options.clone());
        let child_code = compile_vapor(&allocator, child, options);
        assert!(
            compiled.error_messages.is_empty(),
            "{case}: {:?}",
            compiled.error_messages
        );
        assert!(
            child_code.error_messages.is_empty(),
            "{case}: {:?}",
            child_code.error_messages
        );
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            0,
            "{case}: native S3"
        );
        insta::assert_snapshot!(format!("component_model_name_{case}"), compiled.code);
        let props = ["title", "label", "titleModifiers", "labelModifiers"];
        let emits = ["update:title", "update:label"];
        let native = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend":"vapor","code":compiled.code,"context":context,"steps":steps,"identities":true,
                "components":{"Child":{"code":child_code.code,"props":props,"emits":emits}},
            }),
        );
        let official = trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({
                "source":source,"context":context,"steps":steps,
                "components":{"Child":{"source":child,"props":props,"emits":emits}},
            }),
        );
        assert_eq!(native, expected, "{case}: native runtime");
        assert_eq!(official, expected, "{case}: official runtime");
    }
}
