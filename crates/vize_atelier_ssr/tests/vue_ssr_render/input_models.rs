//! Input-model output is checked through the production Vue server renderer,
//! across fallthrough roots, inline elements, and explicit merged props.

use super::{json, render_cases};
use serde::Deserialize;
use serde_json::Value;
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_l0::Allocator;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    name: String,
    props: String,
    data: Value,
    checked: bool,
    #[serde(default)]
    inline_checked: Option<bool>,
}

fn compile(template: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_ssr_with_options(&allocator, template, SsrCompilerOptions::default());
    assert!(errors.is_empty(), "{template}: {errors:?}");
    format!("{}{}", result.preamble, result.code)
}

#[test]
fn input_models_match_vue_in_all_attribute_emission_paths() {
    let fixtures: Vec<Fixture> = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/ssr-input-models.json"
    ))
    .expect("input model corpus");
    let mut cases = Vec::new();
    let mut expected = Vec::new();
    for fixture in fixtures {
        for path in ["root", "inline", "spread"] {
            let spread = if path == "spread" {
                " v-bind=\"{}\""
            } else {
                ""
            };
            let input = format!("<input {} v-model=\"model\"{spread}>", fixture.props);
            let template = if path == "root" {
                input
            } else {
                format!("<div>{input}</div>")
            };
            let code = compile(&template);
            #[cfg(feature = "legacy-differential")]
            assert_eq!(
                code,
                vize_atelier_ssr::differential::with_legacy_lane(|| compile(&template)),
                "{}-{path}: native and legacy model lowering",
                fixture.name
            );
            cases.push(format!(
                "{{\"name\":{},\"template\":{},\"vize\":{},\"data\":{},\"compareCheckedWithClient\":{}}}",
                json(&format!("{}-{path}", fixture.name)),
                json(&template),
                json(&code),
                fixture.data,
                fixture.props.contains("true-value")
            ));
            expected.push(if path == "inline" {
                fixture.inline_checked.unwrap_or(fixture.checked)
            } else {
                fixture.checked
            });
        }
    }
    let rendered = render_cases(&format!(
        "{{\"check\":true,\"cases\":[{}]}}",
        cases.join(",")
    ));
    assert_eq!(rendered.lines().count(), expected.len());
    for (line, checked) in rendered.lines().zip(expected) {
        let result: Value = serde_json::from_str(line).expect("rendered model JSON");
        let html = result["vize"]["html"]
            .as_str()
            .expect("rendered model HTML");
        assert_eq!(
            html.contains(" checked"),
            checked,
            "{}: {html}",
            result["name"]
        );
    }
}
