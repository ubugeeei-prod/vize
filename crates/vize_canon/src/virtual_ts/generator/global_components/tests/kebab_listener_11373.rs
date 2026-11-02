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
        assert!(line.contains(expected), "{id}: {line}");
        assert!(
            !transform
                .text
                .contains(&format!("const {reference}: any = undefined as any")),
            "{id}: {}",
            transform.text
        );
        let call = format!("_emit_resolver)({reference})");
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
    assert!(
        !default.code.contains("__vize_global_component_"),
        "{}",
        default.code
    );
    assert!(
        checked.code.contains("__vize_global_component_"),
        "{}",
        checked.code
    );
    assert!(
        !local.code.contains("declare const global_link:"),
        "{}",
        local.code
    );
    assert!(!local.code.contains("$GlobalComponent<"), "{}", local.code);
}
