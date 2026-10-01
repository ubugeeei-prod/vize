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
                "{{\"name\":{},\"template\":{},\"vize\":{},\"data\":{},\"observeInput\":true,\"compareCheckedWithClient\":{}}}",
                json(&format!("{}-{path}", fixture.name)),
                json(&template),
                json(&code),
                fixture.data,
                fixture.props.contains("true-value")
            ));
            expected.push(fixture.checked);
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
            result["vize"]["inputChecked"]
                .as_bool()
                .expect("input checked state"),
            checked,
            "{}: {html}",
            result["name"]
        );
    }
}

#[test]
fn select_options_use_the_first_authored_value() {
    let descriptor = vize_atelier_sfc::parse_sfc(
        include_str!("../fixtures/select-value-order.vue"),
        vize_atelier_sfc::SfcParseOptions::default(),
    )
    .expect("select value-order SFC");
    let template = &descriptor.template.expect("select template").content;
    let code = compile(template);
    #[cfg(feature = "legacy-differential")]
    assert_eq!(
        code,
        vize_atelier_ssr::differential::with_legacy_lane(|| compile(template)),
        "select value order: native and legacy model lowering"
    );
    let mut cases = Vec::new();
    let expected = [[true, false], [false, true], [false, false], [true, true]];
    for model in [
        json("bound"),
        json("static"),
        json("other"),
        "[\"bound\",\"static\"]".into(),
    ] {
        cases.push(format!(
            "{{\"name\":{},\"template\":{},\"vize\":{},\"data\":{{\"model\":{model},\"bound\":\"bound\"}}}}",
            json(&model), json(template), json(&code)
        ));
    }
    let rendered = render_cases(&format!(
        "{{\"check\":true,\"cases\":[{}]}}",
        cases.join(",")
    ));
    assert_eq!(rendered.lines().count(), expected.len());
    for (line, selected) in rendered.lines().zip(expected) {
        let result: Value = serde_json::from_str(line).expect("rendered select JSON");
        let html = result["vize"]["html"]
            .as_str()
            .expect("rendered select HTML");
        let actual = html
            .split("<option ")
            .skip(1)
            .map(|tail| {
                tail.split('>')
                    .next()
                    .expect("option attributes")
                    .contains("selected")
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, selected, "{}: {html}", result["name"]);
    }
}
