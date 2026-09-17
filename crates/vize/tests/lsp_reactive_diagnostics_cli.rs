use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

fn diagnostic(source: &str, needle: &str, token: &str, code: u32, message: &str) -> Value {
    let mut start = position(source, needle);
    start["character"] = json!(
        start["character"].as_u64().unwrap()
            + needle[..needle.find(token).unwrap()].encode_utf16().count() as u64
    );
    let end = json!({
        "line": start["line"],
        "character": start["character"].as_u64().unwrap() + token.encode_utf16().count() as u64
    });
    json!({
        "code": code,
        "message": message,
        "range": { "start": start, "end": end },
        "severity": 1,
        "source": "vize/types"
    })
}

#[test]
fn ordinary_values_and_user_ref_classes_keep_original_diagnostics() {
    let declarations = r#"const number = 1 as number;
const text = 'hello' as string;
const node = document.createElement('div');
class Ref<T> { constructor(public value: T) {} }
const custom = new Ref(1);"#;
    for newline in ["\n", "\r\n"] {
        let valid = format!(
            "<script setup lang=\"ts\">\n{declarations}\n</script>\n<template>{{{{ '\u{1f600}' }}}}{{{{ number.toFixed() }}}}{{{{ text.toUpperCase() }}}}{{{{ node.tagName }}}}{{{{ custom.value }}}}</template>\n"
        ).replace('\n', newline);
        let mut fixture = Fixture::new_with_vue(&valid);
        assert_eq!(fixture.open(&valid), json!([]));
        for (index, (from, to, token, code, message)) in [
            (
                "number.toFixed()",
                "number.toUpperCase()",
                "toUpperCase",
                2339,
                "Property 'toUpperCase' does not exist on type 'number'.",
            ),
            (
                "text.toUpperCase()",
                "text.toFixed()",
                "toFixed",
                2551,
                "Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?",
            ),
            (
                "node.tagName",
                "node.toFixed()",
                "toFixed",
                2339,
                "Property 'toFixed' does not exist on type 'HTMLDivElement'.",
            ),
            (
                "custom.value",
                "custom.toFixed()",
                "toFixed",
                2339,
                "Property 'toFixed' does not exist on type 'Ref<number>'.",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let broken = valid.replace(from, to);
            assert_eq!(
                fixture.change(&broken, (index * 2 + 2) as i64),
                json!([diagnostic(&broken, to, token, code, message)])
            );
            assert_eq!(fixture.change(&valid, (index * 2 + 3) as i64), json!([]));
        }
        fixture.shutdown();
    }
}

#[test]
fn actual_vue_refs_preserve_script_errors_and_template_unwrapping() {
    for (factory, initializer, ty) in [
        ("ref", "ref(1)", "Ref<number, number>"),
        ("computed", "computed(() => 1)", "ComputedRef<number>"),
        ("shallowRef", "shallowRef(1)", "ShallowRef<number, number>"),
    ] {
        for newline in ["\n", "\r\n"] {
            let valid = format!(
                "<script setup lang=\"ts\">\nimport {{ {factory} }} from 'vue';\nconst count = {initializer};\ncount.value.toFixed();\n</script>\n<template>{{{{ '\u{1f600}' }}}}{{{{ count.toFixed() }}}}</template>\n"
            ).replace('\n', newline);
            let mut fixture = Fixture::new_with_vue(&valid);
            assert_eq!(fixture.open(&valid), json!([]), "{factory}");

            let script_error = valid.replace("count.value.toFixed()", "count.toFixed()");
            assert_eq!(
                fixture.change(&script_error, 2),
                json!([diagnostic(
                    &script_error,
                    "count.toFixed()",
                    "toFixed",
                    2339,
                    &format!("Property 'toFixed' does not exist on type '{ty}'.")
                )]),
                "{factory}"
            );
            assert_eq!(fixture.change(&valid, 3), json!([]));

            let template_error =
                valid.replace("{{ count.toFixed() }}", "{{ count.toUpperCase() }}");
            assert_eq!(
                fixture.change(&template_error, 4),
                json!([diagnostic(
                    &template_error,
                    "count.toUpperCase()",
                    "toUpperCase",
                    2339,
                    "Property 'toUpperCase' does not exist on type 'number'."
                )])
            );
            assert_eq!(fixture.change(&valid, 5), json!([]));

            let explicit_unwrap = valid.replace("{{ count.toFixed() }}", "{{ count.value }}");
            assert_eq!(
                fixture.change(&explicit_unwrap, 6),
                json!([diagnostic(
                    &explicit_unwrap,
                    "count.value }}",
                    "value",
                    2551,
                    "Property 'value' does not exist on type 'number'. Did you mean 'valueOf'?"
                )])
            );
            assert_eq!(fixture.change(&valid, 7), json!([]));
            fixture.shutdown();
        }
    }
}

#[test]
fn ref_named_assignment_errors_do_not_claim_vue_identity() {
    for (declaration, value_type) in [
        (
            "class Ref<T> { constructor(public value: T) {} }\nconst count = new Ref(1);",
            "Ref<number>",
        ),
        (
            "import { ref } from 'vue';\nconst count = ref(1);",
            "Ref<number, number>",
        ),
    ] {
        let valid = format!(
            "<script setup lang=\"ts\">\n{declaration}\nconst target: number = count.value;\n</script>\n<template>{{{{ target }}}}</template>\n"
        );
        let mut fixture = Fixture::new_with_vue(&valid);
        assert_eq!(fixture.open(&valid), json!([]));
        let broken = valid.replace("= count.value;", "= count;");
        assert_eq!(
            fixture.change(&broken, 2),
            json!([diagnostic(
                &broken,
                "target",
                "target",
                2322,
                &format!("Type '{value_type}' is not assignable to type 'number'.")
            )])
        );
        assert_eq!(fixture.change(&valid, 3), json!([]));
        fixture.shutdown();
    }
}
