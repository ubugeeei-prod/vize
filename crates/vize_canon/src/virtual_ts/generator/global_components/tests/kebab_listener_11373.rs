//! #11373: keep a typed registry owner before a listener needs its value.
use std::path::Path;

use crate::generate_vue_content_mapper_transform;
use serde_json::json;

const ORIGINALS: &[(&str, &str, &str)] = &[
    (
        "src/GlobalComponentListener.vue",
        "global_link",
        include_str!(
            "../../../../../../../tests/_fixtures/differential/typechecker/global-component-listener-11373/src/GlobalComponentListener.vue.txt"
        ),
    ),
    (
        "src/PascalListener.vue",
        "GlobalLink",
        include_str!(
            "../../../../../../../tests/_fixtures/differential/typechecker/global-component-listener-11373/src/PascalListener.vue.txt"
        ),
    ),
    (
        "src/NoListener.vue",
        "global_link",
        include_str!(
            "../../../../../../../tests/_fixtures/differential/typechecker/global-component-listener-11373/src/NoListener.vue.txt"
        ),
    ),
];

#[test]
fn original_mapper_outputs_keep_typed_global_declarations_before_values() {
    let mut captured = Vec::new();
    for &(id, reference, source) in ORIGINALS {
        let result = generate_vue_content_mapper_transform(Path::new(id), source);
        let packet = match &result {
            Ok(transform) => serde_json::to_value(transform).unwrap(),
            Err(error) => json!({"error":format!("{error:#?}")}),
        };
        println!(
            "vize-global-listener-original {}",
            json!({"id":id,"source":source,"result":packet})
        );
        captured.push((id, reference, result));
    }
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../../tests/_fixtures/differential/typechecker/global-component-listener-11373/cases.json"
    )).unwrap();
    assert_eq!(pack["historicalIssue"], 11373);
    assert_eq!(pack["cases"][0]["inputs"].as_array().unwrap().len(), 6);
    for (id, reference, result) in captured {
        let transform = result.unwrap();
        assert!(transform.diagnostics.is_empty(), "{id}: {transform:#?}");
        let declaration = format!("declare const {reference}:");
        let declarations = transform
            .text
            .match_indices(&declaration)
            .collect::<Vec<_>>();
        assert_eq!(declarations.len(), 1, "{id}: {}", transform.text);
        let (start, _) = declarations[0];
        let line = transform.text[start..].lines().next().unwrap();
        let expected = if reference == "GlobalLink" {
            "$GlobalComponent<\"GlobalLink\">"
        } else {
            "$GlobalComponent<\"GlobalLink\", \"globalLink\", \"global-link\">"
        };
        assert_eq!(
            line,
            format!("{declaration} $GlobalComponentFallback & {expected};"),
            "{id}"
        );
        let late = format!("const {reference}:");
        let late_bindings = transform
            .text
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with(&late))
            .collect::<Vec<_>>();
        assert_eq!(
            late_bindings,
            Vec::<&str>::new(),
            "{id}: {}",
            transform.text
        );
        let call = format!(", typeof {reference}>)({reference}); const __vize_emit_props_");
        if id == "src/GlobalComponentListener.vue" {
            let call_start = transform
                .text
                .find(&call)
                .expect("original listener inference uses the component value");
            assert!(start < call_start, "{}", transform.text);
        }
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &transform.text, oxc_span::SourceType::tsx())
                .parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{id}: {:?}\n{}",
            parsed.diagnostics,
            transform.text
        );
    }
}

#[test]
fn default_unknown_and_local_component_policies_remain_separate() {
    let unknown = "<unregistered-link to=\"/\" @click=\"() => {}\" />";
    let default = super::generate("", unknown, false);
    let checked = super::generate("", unknown, true);
    let local = super::generate(
        "const GlobalLink = (props: { to: number }) => props.to;",
        "<global-link :to=\"1\" />",
        false,
    );
    for (id, result) in [
        ("unknown-default", &default),
        ("unknown-explicit-check", &checked),
        ("local-binding", &local),
    ] {
        println!("vize-global-listener-policy id={id} result={result:#?}");
    }
    let probes = |code: &str| {
        code.lines()
            .filter(|line| {
                line.starts_with("const { \"unregistered-link\":")
                    || line.starts_with("void __vize_global_component_")
            })
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        probes(&default.code),
        Vec::<String>::new(),
        "{}",
        default.code
    );
    assert_eq!(probes(&checked.code), vec![
        "const { \"unregistered-link\": __vize_global_component_0 } = {} as (\"UnregisteredLink\" extends keyof import('vue').GlobalComponents ? { \"unregistered-link\": unknown } : \"unregisteredLink\" extends keyof import('vue').GlobalComponents ? { \"unregistered-link\": unknown } : \"unregistered-link\" extends keyof import('vue').GlobalComponents ? { \"unregistered-link\": unknown } : {});".to_owned(),
        "void __vize_global_component_0;".to_owned(),
    ], "{}", checked.code);
    let local_registry = local
        .code
        .lines()
        .filter(|line| {
            line.starts_with("declare const global_link:")
                || line.starts_with("type $GlobalComponent<")
        })
        .collect::<Vec<_>>();
    assert_eq!(local_registry, Vec::<&str>::new(), "{}", local.code);
}

#[test]
fn retained_prop_comments_keep_the_complete_module_and_authored_value_map() {
    let mut captured = Vec::new();
    for raw in [
        "value // @ts-ignore",
        "value, other // @ts-ignore",
        "'// @ts-ignore'",
    ] {
        let source = format!(
            "<script setup lang=\"ts\">\nconst value = 1; const other = 2;\nconst Widget = (props: {{ foo: unknown; bar: number }}) => props;\n</script>\n<template><Widget :foo=\"{raw}\" :bar=\"other\" /></template>\n"
        );
        let result = generate_vue_content_mapper_transform(
            Path::new("src/TrailingPropComment.vue"),
            &source,
        );
        let packet = match &result {
            Ok(transform) => serde_json::to_value(transform).unwrap(),
            Err(error) => json!({"error":format!("{error:#?}")}),
        };
        println!(
            "vize-global-listener-prop-comment {}",
            json!({"source":source,"raw":raw,"result":packet})
        );
        captured.push((source, raw, result));
    }
    for (source, raw, result) in captured {
        let transform = result.unwrap();
        assert!(transform.diagnostics.is_empty(), "{transform:#?}");
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &transform.text, oxc_span::SourceType::tsx())
                .parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{:?}\n{}",
            parsed.diagnostics,
            transform.text
        );
        assert_eq!(source.matches(raw).count(), 1);
        let start = source.find(raw).unwrap();
        assert!(
            transform.mappings.iter().any(|mapping| {
                let [generated, generated_len, original, original_len, _, _] = mapping.0;
                original == start
                    && original_len == raw.len()
                    && transform.text.get(generated..generated + generated_len) == Some(raw)
            }),
            "authored value must exclude the generated newline: {transform:#?}"
        );
    }
}
